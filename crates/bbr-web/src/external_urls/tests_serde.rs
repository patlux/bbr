//! Serde contract tests for the external-URL contract.
//!
//! The opaque routing identities (`ExternalUrlRequestId`, `WindowId`,
//! `SpaceId`) serialize as **bare unsigned integers**: no strings, maps, or
//! nested structure, so no URL payload or platform pointer can ride along. A
//! minimal `U64Only` serializer proves that shape directly. Every other wire
//! type gets a compile-time `Serialize + Deserialize` bound, mirroring the
//! sibling file-panel suite.

use std::fmt;

use serde::de::Visitor;
use serde::ser::{Impossible, Serializer};
use serde::{Deserialize, Serialize};

use crate::external_urls::{
    ActivationHint, BurstConfig, ExternalUrlOutcome, ExternalUrlRequest, ExternalUrlRequestId,
    RoutingDecision, SpaceId, UrlClass, WebScheme, WebUrl, WindowId, WindowSnapshot,
};
use crate::ProfileId;

/// Serializer that accepts exactly one `u64` and rejects every other shape.
struct U64Only;

#[derive(Debug)]
struct NotABareInteger(String);

impl fmt::Display for NotABareInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NotABareInteger {}

impl serde::ser::Error for NotABareInteger {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

impl serde::de::Error for NotABareInteger {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

fn not_bare(shape: &'static str) -> NotABareInteger {
    NotABareInteger(format!("expected exactly one u64, found {shape}"))
}

impl Serializer for U64Only {
    type Ok = u64;
    type Error = NotABareInteger;
    type SerializeSeq = Impossible<u64, NotABareInteger>;
    type SerializeTuple = Impossible<u64, NotABareInteger>;
    type SerializeTupleStruct = Impossible<u64, NotABareInteger>;
    type SerializeTupleVariant = Impossible<u64, NotABareInteger>;
    type SerializeMap = Impossible<u64, NotABareInteger>;
    type SerializeStruct = Impossible<u64, NotABareInteger>;
    type SerializeStructVariant = Impossible<u64, NotABareInteger>;

    fn serialize_bool(self, _: bool) -> Result<u64, Self::Error> {
        Err(not_bare("bool"))
    }
    fn serialize_i8(self, _: i8) -> Result<u64, Self::Error> {
        Err(not_bare("i8"))
    }
    fn serialize_i16(self, _: i16) -> Result<u64, Self::Error> {
        Err(not_bare("i16"))
    }
    fn serialize_i32(self, _: i32) -> Result<u64, Self::Error> {
        Err(not_bare("i32"))
    }
    fn serialize_i64(self, _: i64) -> Result<u64, Self::Error> {
        Err(not_bare("i64"))
    }
    fn serialize_u8(self, _: u8) -> Result<u64, Self::Error> {
        Err(not_bare("u8"))
    }
    fn serialize_u16(self, _: u16) -> Result<u64, Self::Error> {
        Err(not_bare("u16"))
    }
    fn serialize_u32(self, _: u32) -> Result<u64, Self::Error> {
        Err(not_bare("u32"))
    }
    fn serialize_u64(self, value: u64) -> Result<u64, Self::Error> {
        Ok(value)
    }
    fn serialize_f32(self, _: f32) -> Result<u64, Self::Error> {
        Err(not_bare("f32"))
    }
    fn serialize_f64(self, _: f64) -> Result<u64, Self::Error> {
        Err(not_bare("f64"))
    }
    fn serialize_char(self, _: char) -> Result<u64, Self::Error> {
        Err(not_bare("char"))
    }
    fn serialize_str(self, _: &str) -> Result<u64, Self::Error> {
        Err(not_bare("string"))
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<u64, Self::Error> {
        Err(not_bare("bytes"))
    }
    fn serialize_none(self) -> Result<u64, Self::Error> {
        Err(not_bare("none"))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<u64, Self::Error> {
        Err(not_bare("some"))
    }
    fn serialize_unit(self) -> Result<u64, Self::Error> {
        Err(not_bare("unit"))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<u64, Self::Error> {
        Err(not_bare("unit struct"))
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<u64, Self::Error> {
        Err(not_bare("unit variant"))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<u64, Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<u64, Self::Error> {
        Err(not_bare("newtype variant"))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(not_bare("sequence"))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(not_bare("tuple"))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(not_bare("tuple struct"))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(not_bare("tuple variant"))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(not_bare("map"))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(not_bare("struct"))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(not_bare("struct variant"))
    }
    fn collect_str<T: ?Sized + fmt::Display>(self, _: &T) -> Result<u64, Self::Error> {
        Err(not_bare("string"))
    }
}

/// Deserializer that accepts exactly one `u64` (mirroring [`U64Only`]),
/// including through newtype-struct derivation.
struct U64ValueDeserializer {
    value: u64,
}

impl<'de> serde::Deserializer<'de> for U64ValueDeserializer {
    type Error = NotABareInteger;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.value)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.value)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}

/// Each opaque routing identity serializes as exactly one integer and
/// round-trips through a deserializer that accepts only integers.
#[test]
fn opaque_ids_round_trip_as_bare_integers() {
    let wire = ExternalUrlRequestId(7)
        .serialize(U64Only)
        .expect("request id must serialize as one u64");
    assert_eq!(wire, 7);
    let wire = WindowId(31)
        .serialize(U64Only)
        .expect("window id must serialize as one u64");
    assert_eq!(wire, 31);
    let wire = SpaceId(42)
        .serialize(U64Only)
        .expect("space id must serialize as one u64");
    assert_eq!(wire, 42);

    let back = ExternalUrlRequestId::deserialize(U64ValueDeserializer { value: 9 })
        .expect("bare integer deserializes back");
    assert_eq!(back, ExternalUrlRequestId(9));
    let back = WindowId::deserialize(U64ValueDeserializer { value: 1 })
        .expect("bare integer deserializes back");
    assert_eq!(back, WindowId(1));
    let back = SpaceId::deserialize(U64ValueDeserializer { value: 2 })
        .expect("bare integer deserializes back");
    assert_eq!(back, SpaceId(2));
}

/// A URL-payload-shaped value is rejected by the shape harness, so an opaque
/// identity can never silently grow a URL or native field without these
/// tests failing.
#[test]
fn wire_shape_rejects_url_payloads() {
    #[derive(Serialize)]
    struct IdAndUrl {
        id: u64,
        raw_url: String,
    }
    let payload = IdAndUrl {
        id: 1,
        raw_url: "https://user:secret@example.com/private?token=abc".into(),
    };
    assert!(payload.serialize(U64Only).is_err());
}

/// All external-URL wire types are `Serialize + Deserialize` (compile-time
/// proof); the concrete wire format lives where a format exists (the testkit
/// socket). Control-path errors and the borrowed `RoutingInput` stay
/// deliberately non-serde.
#[test]
fn wire_types_are_serde() {
    fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}
    assert_serde::<ExternalUrlRequestId>();
    assert_serde::<ActivationHint>();
    assert_serde::<ExternalUrlRequest>();
    assert_serde::<WindowId>();
    assert_serde::<SpaceId>();
    assert_serde::<WindowSnapshot>();
    assert_serde::<RoutingDecision>();
    assert_serde::<UrlClass>();
    assert_serde::<WebUrl>();
    assert_serde::<WebScheme>();
    assert_serde::<BurstConfig>();
    assert_serde::<ExternalUrlOutcome>();
    assert_serde::<ProfileId>();
}
