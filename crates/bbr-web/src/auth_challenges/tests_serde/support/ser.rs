//! Wire-shape serializer for the serde contract tests. Records the exact
//! serialization order as [`Tok`]s; see the parent module for why no JSON
//! dependency is available.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

/// Error type of the shape serializer: our contract types cannot fail to
/// serialize, so serialization errors are unrepresentable.
#[derive(Debug)]
pub(crate) struct Never;

impl fmt::Display for Never {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test shape serializer cannot fail")
    }
}

impl std::error::Error for Never {}

impl serde::ser::Error for Never {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Never
    }
}

/// One wire token: struct/field/variant names and scalar values, in
/// serialization order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tok {
    Struct(&'static str),
    End,
    Variant(&'static str, &'static str),
    NewtypeVariant(&'static str, &'static str),
    StructVariant(&'static str, &'static str),
    Newtype(&'static str),
    Field(&'static str),
    Str(String),
    U(u64),
    Bool(bool),
    None,
}

/// Serializer that records the wire shape as [`Tok`]s. The token buffer is
/// shared through `Rc<RefCell>` because the `Serialize*` trait impls receive
/// `&mut self` while nested values must be serialized through an owned
/// [`Serializer`] (serde_core provides no blanket `Serializer for &mut S`).
#[derive(Default, Clone)]
pub(crate) struct S {
    toks: Rc<RefCell<Vec<Tok>>>,
}

impl S {
    pub(crate) fn tokens(&self) -> Vec<Tok> {
        self.toks.borrow().clone()
    }
}

macro_rules! ser_int {
    ($($m:ident => $t:ty),* $(,)?) => {
        $(fn $m(self, v: $t) -> Result<Self::Ok, Self::Error> {
            self.toks.borrow_mut().push(Tok::U(v as u64));
            Ok(())
        })*
    };
}

impl Serializer for S {
    type Ok = ();
    type Error = Never;
    type SerializeSeq = S;
    type SerializeTuple = S;
    type SerializeTupleStruct = S;
    type SerializeTupleVariant = S;
    type SerializeMap = S;
    type SerializeStruct = S;
    type SerializeStructVariant = S;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Bool(v));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::U(v));
        Ok(())
    }

    ser_int! {
        serialize_i8 => i8,
        serialize_i16 => i16,
        serialize_i32 => i32,
        serialize_i64 => i64,
        serialize_u8 => u8,
        serialize_u16 => u16,
        serialize_u32 => u32,
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str(v.to_string()));
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str(v.to_string()));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str(v.to_string()));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str(v.to_owned()));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str(format!("{v:?}")));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::None);
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str("()".into()));
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Newtype(name));
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Variant(name, variant));
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Newtype(name));
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.toks
            .borrow_mut()
            .push(Tok::NewtypeVariant(name, variant));
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.toks.borrow_mut().push(Tok::Struct(name));
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.toks
            .borrow_mut()
            .push(Tok::StructVariant(name, variant));
        Ok(self)
    }

    fn collect_str<T: ?Sized + fmt::Display>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.toks.borrow_mut().push(Tok::Str(value.to_string()));
        Ok(())
    }
}

impl SerializeSeq for S {
    type Ok = ();
    type Error = Never;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}

impl SerializeTuple for S {
    type Ok = ();
    type Error = Never;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}

impl SerializeTupleStruct for S {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}

impl SerializeTupleVariant for S {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}

impl SerializeMap for S {
    type Ok = ();
    type Error = Never;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Never> {
        key.serialize(self.clone())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Never> {
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}

impl SerializeStruct for S {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::Field(key));
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}

impl SerializeStructVariant for S {
    type Ok = ();
    type Error = Never;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::Field(key));
        value.serialize(self.clone())
    }

    fn end(self) -> Result<(), Never> {
        self.toks.borrow_mut().push(Tok::End);
        Ok(())
    }
}
