/// Serde-contract tests for the portable permission identities, taxonomy,
/// and validation: compile-time wire-shape bounds for every contract type and
/// origin re-validation at the deserialization boundary.
///
/// The wire tests use [`serde::de::value::MapDeserializer`] with a tiny local
/// value enum (serde 1.0.x removed the old `de::value::Value`), so no
/// serialization-format crate is needed.
use super::*;
use serde::de::value::{Error, MapDeserializer};
use serde::Deserialize;
use std::marker::PhantomData;

fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}

/// A map value in a minimal wire test: a string, an unsigned integer, or
/// absent (unit / `None`, which deserializes as `Option::None`).
enum WireValue {
    Str(String),
    U64(u64),
    None,
}

impl<'de> serde::de::IntoDeserializer<'de, Error> for WireValue {
    type Deserializer = WireValueDeserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        WireValueDeserializer {
            value: self,
            _marker: PhantomData,
        }
    }
}

struct WireValueDeserializer<'de> {
    value: WireValue,
    _marker: PhantomData<&'de ()>,
}

impl<'de> serde::Deserializer<'de> for WireValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            WireValue::Str(s) => visitor.visit_string(s),
            WireValue::U64(n) => visitor.visit_u64(n),
            WireValue::None => visitor.visit_unit(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            WireValue::None => visitor.visit_none(),
            other => visitor.visit_some(WireValueDeserializer {
                value: other,
                _marker: PhantomData,
            }),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

type WireMap = MapDeserializer<'static, std::vec::IntoIter<(&'static str, WireValue)>, Error>;

fn origin_map(scheme: &str, host: &str, port: WireValue) -> WireMap {
    MapDeserializer::new(
        vec![
            ("scheme", WireValue::Str(scheme.to_string())),
            ("host", WireValue::Str(host.to_string())),
            ("port", port),
        ]
        .into_iter(),
    )
}

/// Every contract type derives stable wire shapes behind the `serde` feature.
#[test]
fn all_contract_types_are_serde_roundtrippable() {
    assert_serde::<PermissionRequestId>();
    assert_serde::<PermissionCapability>();
    assert_serde::<PermissionOrigin>();
    assert_serde::<PermissionScope>();
    assert_serde::<PermissionRequest>();
    assert_serde::<PermissionDecision>();
    assert_serde::<PermissionPersistence>();
    assert_serde::<PermissionVerdict>();
    assert_serde::<PermissionPolicyKey>();
    assert_serde::<PermissionPolicyValue>();
    assert_serde::<PermissionPolicyEntry>();
    assert_serde::<PermissionRequestOutcome>();
    assert_serde::<PermissionTransition>();
    assert_serde::<PermissionResolution>();
    assert_serde::<PermissionRejection>();
    assert_serde::<PermissionCompletion>();
    assert_serde::<PermissionPersistenceCommand>();
    assert_serde::<OriginError>();
}

/// Deserializing an origin re-runs the same validation as parsing: valid
/// spellings normalize, default ports collapse, a missing port defaults, and
/// invalid or empty origins are rejected at the wire boundary.
#[test]
fn origin_deserializes_and_revalidates() {
    let wire = PermissionOrigin::deserialize(origin_map("HTTPS", "Example.COM", WireValue::None))
        .expect("valid origin must deserialize");
    assert_eq!(
        wire,
        PermissionOrigin::parse("https://example.com").unwrap()
    );

    let with_port =
        PermissionOrigin::deserialize(origin_map("https", "example.com", WireValue::U64(8443)))
            .expect("port must deserialize");
    assert_eq!(with_port.port(), Some(8443));

    let default_port =
        PermissionOrigin::deserialize(origin_map("https", "example.com", WireValue::U64(443)))
            .expect("default port must deserialize");
    assert_eq!(default_port.port(), None);
    assert_eq!(
        default_port,
        PermissionOrigin::parse("https://example.com").unwrap()
    );

    let missing_port = vec![
        ("scheme", WireValue::Str("https".to_string())),
        ("host", WireValue::Str("example.com".to_string())),
    ];
    let parsed = PermissionOrigin::deserialize(MapDeserializer::new(missing_port.into_iter()))
        .expect("missing port is optional");
    assert_eq!(
        parsed,
        PermissionOrigin::parse("https://example.com").unwrap()
    );

    let missing_host = vec![("scheme", WireValue::Str("https".to_string()))];
    assert!(PermissionOrigin::deserialize(MapDeserializer::new(missing_host.into_iter())).is_err());

    assert!(PermissionOrigin::deserialize(origin_map("https", "", WireValue::None)).is_err());
    assert!(PermissionOrigin::deserialize(origin_map("https", "a b", WireValue::None)).is_err());
    assert!(PermissionOrigin::deserialize(origin_map(
        "https",
        "user@example.com",
        WireValue::None
    ))
    .is_err());
    assert!(PermissionOrigin::deserialize(origin_map("", "example.com", WireValue::None)).is_err());
    assert!(
        PermissionOrigin::deserialize(origin_map("https", "example.com", WireValue::U64(0)))
            .is_err()
    );
    assert!(PermissionOrigin::deserialize(origin_map(
        "https",
        "example.com",
        WireValue::U64(65536)
    ))
    .is_err());
}
