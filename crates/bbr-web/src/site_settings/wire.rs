//! Test-only wire support: a minimal compact-JSON capture serializer and a
//! scalar deserializer for the deserialize direction.
//!
//! `bbr-web` may not add a serde format dependency (no manifests/lock
//! changes), so the exact wire shape is pinned here instead of with
//! `serde_json`. The deserialize direction reuses serde's built-in value
//! deserializers ([`serde::de::value`]) plus [`WireValueDeserializer`] for
//! map values.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant, Serializer,
};

/// Error type for the test wire format.
#[derive(Debug)]
pub struct WireError;

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("wire capture error")
    }
}

impl std::error::Error for WireError {}

impl serde::ser::Error for WireError {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        WireError
    }
}

impl serde::de::Error for WireError {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        WireError
    }
}

/// Serializer writing compact JSON into a shared buffer.
#[derive(Debug, Clone)]
struct Wire(Rc<RefCell<String>>);

impl Wire {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(String::new())))
    }

    fn push(&self, text: &str) {
        self.0.borrow_mut().push_str(text);
    }

    fn into_string(self) -> String {
        Rc::try_unwrap(self.0)
            .map(|buf| buf.into_inner())
            .unwrap_or_else(|rc| (*rc.borrow()).clone())
    }
}

/// JSON string escaping for the capture format.
fn escape_json(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Generate the `Serializer` methods this capture format never supports:
/// each delegates to [`WireError`]. Container forms keep their required
/// `WireUnsupported` return type.
macro_rules! unsupported_serialize {
    ($(fn $name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ok:ty;)*) => {
        $(
            fn $name(self, $($arg: $ty),*) -> Result<$ok, WireError> {
                Err(WireError)
            }
        )*
    };
}

impl Serializer for Wire {
    type Ok = ();
    type Error = WireError;
    type SerializeSeq = WireUnsupported;
    type SerializeTuple = WireUnsupported;
    type SerializeTupleStruct = WireUnsupported;
    type SerializeTupleVariant = WireUnsupported;
    type SerializeMap = WireUnsupported;
    type SerializeStruct = WireStruct;
    type SerializeStructVariant = WireUnsupported;

    fn serialize_str(self, value: &str) -> Result<(), WireError> {
        self.push(&format!("\"{}\"", escape_json(value)));
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<(), WireError> {
        self.push(&value.to_string());
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<(), WireError> {
        self.push(&value.to_string());
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<(), WireError> {
        self.push(&value.to_string());
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), WireError> {
        value.serialize(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<WireStruct, WireError> {
        self.push("{");
        Ok(WireStruct {
            wire: self,
            first: true,
        })
    }

    // Unsupported scalar/container forms of this capture format; the wire
    // contract never serializes them.
    unsupported_serialize! {
        fn serialize_bool(_v: bool) -> ();
        fn serialize_i8(_v: i8) -> ();
        fn serialize_i16(_v: i16) -> ();
        fn serialize_i32(_v: i32) -> ();
        fn serialize_i64(_v: i64) -> ();
        fn serialize_i128(_v: i128) -> ();
        fn serialize_u32(_v: u32) -> ();
        fn serialize_u128(_v: u128) -> ();
        fn serialize_f32(_v: f32) -> ();
        fn serialize_f64(_v: f64) -> ();
        fn serialize_char(_v: char) -> ();
        fn serialize_bytes(_v: &[u8]) -> ();
        fn serialize_unit() -> ();
        fn serialize_unit_struct(_name: &'static str) -> ();
        fn serialize_unit_variant(_name: &'static str, _idx: u32, _variant: &'static str) -> ();
        fn serialize_seq(_len: Option<usize>) -> WireUnsupported;
        fn serialize_tuple(_len: usize) -> WireUnsupported;
        fn serialize_tuple_struct(_name: &'static str, _len: usize) -> WireUnsupported;
        fn serialize_tuple_variant(
            _name: &'static str,
            _idx: u32,
            _variant: &'static str,
            _len: usize,
        ) -> WireUnsupported;
        fn serialize_map(_len: Option<usize>) -> WireUnsupported;
        fn serialize_struct_variant(
            _name: &'static str,
            _idx: u32,
            _variant: &'static str,
            _len: usize,
        ) -> WireUnsupported;
    }

    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
        self,
        _name: &'static str,
        _idx: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), WireError> {
        Err(WireError)
    }

    fn serialize_none(self) -> Result<(), WireError> {
        self.push("null");
        Ok(())
    }
    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> Result<(), WireError> {
        value.serialize(self)
    }
}

/// Struct serializer used by the derived `ProfileSiteKey` serialize.
#[derive(Debug, Clone)]
struct WireStruct {
    wire: Wire,
    first: bool,
}

impl SerializeStruct for WireStruct {
    type Ok = ();
    type Error = WireError;

    fn serialize_field<T: ?Sized + serde::Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), WireError> {
        if !self.first {
            self.wire.push(",");
        }
        self.first = false;
        self.wire.push(&format!("\"{key}\":"));
        value.serialize(self.wire.clone())
    }

    fn end(self) -> Result<(), WireError> {
        self.wire.push("}");
        Ok(())
    }
}

/// Placeholder for container forms this contract never serializes.
#[derive(Debug, Clone)]
struct WireUnsupported;

macro_rules! unsupported_trait {
    ($trait:ident, $method:ident) => {
        impl $trait for WireUnsupported {
            type Ok = ();
            type Error = WireError;

            fn $method<T: ?Sized + serde::Serialize>(
                &mut self,
                _value: &T,
            ) -> Result<(), WireError> {
                Err(WireError)
            }

            fn end(self) -> Result<(), WireError> {
                Err(WireError)
            }
        }
    };
}

unsupported_trait!(SerializeSeq, serialize_element);
unsupported_trait!(SerializeTuple, serialize_element);
unsupported_trait!(SerializeTupleStruct, serialize_field);

impl SerializeTupleVariant for WireUnsupported {
    type Ok = ();
    type Error = WireError;

    fn serialize_field<T: ?Sized + serde::Serialize>(
        &mut self,
        _value: &T,
    ) -> Result<(), WireError> {
        Err(WireError)
    }

    fn end(self) -> Result<(), WireError> {
        Err(WireError)
    }
}

impl SerializeMap for WireUnsupported {
    type Ok = ();
    type Error = WireError;

    fn serialize_key<T: ?Sized + serde::Serialize>(&mut self, _key: &T) -> Result<(), WireError> {
        Err(WireError)
    }

    fn serialize_value<T: ?Sized + serde::Serialize>(
        &mut self,
        _value: &T,
    ) -> Result<(), WireError> {
        Err(WireError)
    }

    fn end(self) -> Result<(), WireError> {
        Err(WireError)
    }
}

impl serde::ser::SerializeStructVariant for WireUnsupported {
    type Ok = ();
    type Error = WireError;

    fn serialize_field<T: ?Sized + serde::Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), WireError> {
        Err(WireError)
    }

    fn end(self) -> Result<(), WireError> {
        Err(WireError)
    }
}

/// Serialize `value` and return the compact-JSON wire string.
pub fn wire_serialize<T: serde::Serialize>(value: &T) -> String {
    let wire = Wire::new();
    value
        .serialize(wire.clone())
        .expect("capture serializer must succeed");
    wire.into_string()
}

/// A scalar value inside a map used for the deserialize direction.
#[derive(Debug, Clone, Copy)]
pub enum WireValue {
    U64(u64),
    Str(&'static str),
    /// JSON `null`, used for absent optional fields.
    Null,
}

/// Minimal deserializer for the scalar values used by the key types.
#[derive(Debug, Clone, Copy)]
pub struct WireValueDeserializer(WireValue);

macro_rules! unsupported_value {
    ($($method:ident),* $(,)?) => {
        $(
            fn $method<V>(self, _visitor: V) -> Result<V::Value, WireError>
            where
                V: serde::de::Visitor<'de>,
            {
                Err(WireError)
            }
        )*
    };
}

impl<'de> serde::Deserializer<'de> for WireValueDeserializer {
    type Error = WireError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            WireValue::U64(value) => visitor.visit_u64(value),
            WireValue::Str(value) => visitor.visit_str(value),
            WireValue::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            WireValue::Null => visitor.visit_none(),
            value => visitor.visit_some(WireValueDeserializer(value)),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            WireValue::U64(value) => visitor.visit_u64(value),
            WireValue::Str(_) | WireValue::Null => Err(WireError),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            WireValue::U64(value) => visitor.visit_u64(value),
            WireValue::Str(_) | WireValue::Null => Err(WireError),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            WireValue::Str(value) => visitor.visit_str(value),
            WireValue::U64(_) | WireValue::Null => Err(WireError),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0 {
            WireValue::U64(value) => visitor.visit_u64(value),
            WireValue::Str(value) => visitor.visit_str(value),
            WireValue::Null => visitor.visit_unit(),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(_len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, WireError>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    unsupported_value!(
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u32,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
    );
}

impl<'de> serde::de::IntoDeserializer<'de, WireError> for WireValue {
    type Deserializer = WireValueDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        WireValueDeserializer(self)
    }
}
