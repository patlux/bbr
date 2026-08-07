//! Token-stream deserializer for the serde contract tests: replays the
//! [`Tok`] stream recorded by `super::ser` (and hand-crafted invalid
//! streams) back into the contract types.

use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use serde::de::value::StrDeserializer;
use serde::de::{
    DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};

use super::ser::Tok;

/// Error type of the token deserializer: a message describing the malformed
/// or unexpected token.
#[derive(Debug)]
pub(crate) struct Fail(String);

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Fail {}

impl serde::de::Error for Fail {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Fail(msg.to_string())
    }
}

/// Deserializer that replays a recorded [`Tok`] stream. The cursor position
/// lives in an `Rc<Cell<usize>>` shared by every clone, so the `Seq`/`Map`
/// views can hand an owned `D` clone to nested seeds while still advancing
/// the same stream (serde_core provides no blanket `Deserializer for
/// &mut T`).
#[derive(Clone)]
pub(crate) struct D<'a> {
    toks: &'a [Tok],
    pos: Rc<Cell<usize>>,
}

impl<'a> D<'a> {
    pub(crate) fn new(toks: &'a [Tok]) -> Self {
        D {
            toks,
            pos: Rc::new(Cell::new(0)),
        }
    }

    fn peek(&self) -> Option<Tok> {
        self.toks.get(self.pos.get()).cloned()
    }

    fn next(&self) -> Option<Tok> {
        let tok = self.toks.get(self.pos.get()).cloned();
        if tok.is_some() {
            self.pos.set(self.pos.get() + 1);
        }
        tok
    }

    fn fail<T>(&self, message: impl Into<String>) -> Result<T, Fail> {
        Err(Fail(message.into()))
    }
}

/// Sequence view over the token stream that skips the `Field` markers
/// preceding each value (structs are replayed positionally).
struct Seq<'a> {
    d: D<'a>,
}

impl<'de, 'a> SeqAccess<'de> for Seq<'a> {
    type Error = Fail;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Fail>
    where
        T: DeserializeSeed<'de>,
    {
        loop {
            match self.d.peek() {
                None => return Ok(None),
                Some(Tok::End) => {
                    // Consume the terminator so a nested seq's `End` never
                    // terminates an enclosing seq.
                    self.d.next();
                    return Ok(None);
                }
                Some(Tok::Field(_)) => {
                    self.d.next();
                }
                Some(_) => return seed.deserialize(self.d.clone()).map(Some),
            }
        }
    }
}

/// Map view over the token stream (used only for completeness; the contract
/// types replay positionally).
struct Map<'a> {
    d: D<'a>,
}

impl<'de, 'a> MapAccess<'de> for Map<'a> {
    type Error = Fail;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Fail>
    where
        K: DeserializeSeed<'de>,
    {
        match self.d.next() {
            None => Ok(None),
            Some(Tok::End) => Ok(None),
            Some(Tok::Field(name)) => seed
                .deserialize(StrDeserializer::<Fail>::new(name))
                .map(Some),
            Some(tok) => self.d.fail(format!("expected map key, got {tok:?}")),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Fail>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.d.clone())
    }
}

/// Enum variant access for the token stream.
enum VarAccess<'a> {
    Unit,
    Newtype { d: D<'a> },
    Struct { d: D<'a> },
}

struct EnumAcc<'a> {
    variant: &'static str,
    access: VarAccess<'a>,
}

impl<'de, 'a> EnumAccess<'de> for EnumAcc<'a> {
    type Error = Fail;
    type Variant = VarAccess<'a>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VarAccess<'a>), Fail>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(StrDeserializer::<Fail>::new(self.variant))?;
        Ok((variant, self.access))
    }
}

impl<'de, 'a> VariantAccess<'de> for VarAccess<'a> {
    type Error = Fail;

    fn unit_variant(self) -> Result<(), Fail> {
        match self {
            VarAccess::Unit => Ok(()),
            _ => Err(Fail("expected unit variant".into())),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Fail>
    where
        T: DeserializeSeed<'de>,
    {
        match self {
            VarAccess::Newtype { d } => seed.deserialize(d),
            _ => Err(Fail("expected newtype variant".into())),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self {
            VarAccess::Newtype { d } => visitor.visit_seq(Seq { d }),
            VarAccess::Struct { d } => visitor.visit_seq(Seq { d }),
            VarAccess::Unit => Err(Fail("expected tuple variant".into())),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self {
            VarAccess::Struct { d } => {
                // Same terminator rule as `deserialize_struct`: consume the
                // trailing `End` after the declared fields.
                let value = visitor.visit_seq(Seq { d: d.clone() })?;
                if matches!(d.peek(), Some(Tok::End)) {
                    d.next();
                }
                Ok(value)
            }
            _ => Err(Fail("expected struct variant".into())),
        }
    }
}

impl<'de, 'a> Deserializer<'de> for D<'a> {
    type Error = Fail;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.peek() {
            Some(Tok::Bool(_)) => self.deserialize_bool(visitor),
            Some(Tok::U(_)) => self.deserialize_u64(visitor),
            Some(Tok::Str(_)) => self.deserialize_str(visitor),
            Some(Tok::None) => self.deserialize_option(visitor),
            Some(tok) => self.fail(format!("deserialize_any: unexpected token {tok:?}")),
            None => self.fail("deserialize_any: token stream exhausted"),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.next() {
            Some(Tok::Bool(v)) => visitor.visit_bool(v),
            Some(tok) => self.fail(format!("expected bool, got {tok:?}")),
            None => self.fail("expected bool, token stream exhausted"),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.next() {
            Some(Tok::U(v)) => visitor.visit_u64(v),
            Some(tok) => self.fail(format!("expected u64, got {tok:?}")),
            None => self.fail("expected u64, token stream exhausted"),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.fail("floats are not part of the auth-challenge wire shape")
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.fail("floats are not part of the auth-challenge wire shape")
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.next() {
            Some(Tok::Str(s)) => visitor.visit_str(&s),
            Some(tok) => self.fail(format!("expected string, got {tok:?}")),
            None => self.fail("expected string, token stream exhausted"),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.next() {
            Some(Tok::Str(s)) => visitor.visit_string(s),
            Some(tok) => self.fail(format!("expected string, got {tok:?}")),
            None => self.fail("expected string, token stream exhausted"),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.peek() {
            Some(Tok::None) => {
                self.next();
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.next() {
            Some(Tok::Newtype(n)) if n == name => visitor.visit_newtype_struct(self),
            Some(tok) => self.fail(format!("expected newtype {name}, got {tok:?}")),
            None => self.fail(format!("expected newtype {name}, token stream exhausted")),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Seq { d: self })
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Seq { d: self })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Seq { d: self })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map { d: self })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        match self.next() {
            Some(Tok::Struct(_)) => {
                // The derived visitor reads exactly the declared fields and
                // never requests a terminator, so consume the trailing `End`
                // here; otherwise it would leak into an enclosing seq. The
                // seq works on a clone that shares the same cursor.
                let value = visitor.visit_seq(Seq { d: self.clone() })?;
                if matches!(self.peek(), Some(Tok::End)) {
                    self.next();
                }
                Ok(value)
            }
            Some(tok) => self.fail(format!("expected struct, got {tok:?}")),
            None => self.fail("expected struct, token stream exhausted"),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        let (variant, access) = match self.next() {
            Some(Tok::Variant(_, variant)) => (variant, VarAccess::Unit),
            Some(Tok::NewtypeVariant(_, variant)) => (variant, VarAccess::Newtype { d: self }),
            Some(Tok::StructVariant(_, variant)) => (variant, VarAccess::Struct { d: self }),
            Some(tok) => return self.fail(format!("expected enum variant, got {tok:?}")),
            None => return self.fail("expected enum variant, token stream exhausted"),
        };
        visitor.visit_enum(EnumAcc { variant, access })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Fail>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}
