//! Serde wire-shape contract tests for site identity and zoom values.
//!
//! The exact wire shapes are pinned through the capture serializer in
//! [`super::wire`] (compact JSON, no serde format dependency), and the
//! deserialize direction reuses serde's built-in value deserializers.

use serde::de::value::{MapDeserializer, StrDeserializer, U16Deserializer};
use serde::Deserialize;

use super::wire::{wire_serialize, WireError, WireValue};
use super::{ProfileSiteKey, SiteKey, SiteSettingsOverrides, ZoomPercent};
use crate::ProfileId;

fn site() -> SiteKey {
    SiteKey::parse("https://example.com").expect("fixture must parse")
}

#[test]
fn site_key_wire_shape_is_single_canonical_string() {
    assert_eq!(wire_serialize(&site()), "\"https://example.com:443\"");
    assert_eq!(
        wire_serialize(&SiteKey::parse("http://[::1]:8080").unwrap()),
        "\"http://[::1]:8080\""
    );
}

#[test]
fn site_key_deserializes_from_canonical_string_and_rejects_invalid() {
    let key = SiteKey::deserialize(StrDeserializer::<WireError>::new(
        "https://example.com:8443",
    ))
    .expect("valid wire origin");
    assert_eq!(key, SiteKey::parse("https://example.com:8443").unwrap());

    for invalid in [
        "example.com",
        "ftp://example.com",
        "",
        "https://example.com:99999",
    ] {
        assert!(
            SiteKey::deserialize(StrDeserializer::<WireError>::new(invalid)).is_err(),
            "wire origin {invalid:?} must be rejected"
        );
    }
}

#[test]
fn zoom_wire_shape_is_integer_number() {
    assert_eq!(wire_serialize(&ZoomPercent::default()), "100");
    assert_eq!(wire_serialize(&ZoomPercent::new(25).unwrap()), "25");
    assert_eq!(wire_serialize(&ZoomPercent::new(400).unwrap()), "400");
}

#[test]
fn zoom_deserializes_with_bounds_revalidation() {
    assert_eq!(
        ZoomPercent::deserialize(U16Deserializer::<WireError>::new(100)).unwrap(),
        ZoomPercent::default()
    );
    assert_eq!(
        ZoomPercent::deserialize(U16Deserializer::<WireError>::new(137)).unwrap(),
        ZoomPercent::new(137).unwrap()
    );
    for invalid in [0, 24, 401, u16::MAX] {
        assert!(
            ZoomPercent::deserialize(U16Deserializer::<WireError>::new(invalid)).is_err(),
            "wire zoom {invalid} must be rejected"
        );
    }
}

#[test]
fn profile_site_key_wire_shape_is_compact_object() {
    let key = ProfileSiteKey::new(ProfileId(1), site());
    assert_eq!(
        wire_serialize(&key),
        "{\"profile\":1,\"site\":\"https://example.com:443\"}"
    );
}

#[test]
fn profile_site_key_deserializes_through_derived_struct() {
    let deserializer = MapDeserializer::<_, WireError>::new(
        vec![
            ("profile", WireValue::U64(1)),
            ("site", WireValue::Str("https://example.com:443")),
        ]
        .into_iter(),
    );
    let key = ProfileSiteKey::deserialize(deserializer).expect("valid wire key");
    assert_eq!(key, ProfileSiteKey::new(ProfileId(1), site()));
}

#[test]
fn profile_site_key_deserialization_rejects_invalid_site() {
    let deserializer = MapDeserializer::<_, WireError>::new(
        vec![
            ("profile", WireValue::U64(1)),
            ("site", WireValue::Str("example.com")),
        ]
        .into_iter(),
    );
    assert!(ProfileSiteKey::deserialize(deserializer).is_err());
}

#[test]
fn overrides_wire_shape_is_an_object_with_optional_zoom() {
    let overrides = SiteSettingsOverrides::from_zoom(ZoomPercent::new(125).unwrap());
    assert_eq!(wire_serialize(&overrides), "{\"zoom\":125}");
}

#[test]
fn overrides_deserialize_round_trips_the_zoom_field() {
    let deserializer =
        MapDeserializer::<_, WireError>::new(vec![("zoom", WireValue::U64(125))].into_iter());
    let overrides = SiteSettingsOverrides::deserialize(deserializer).expect("valid wire record");
    assert_eq!(
        overrides,
        SiteSettingsOverrides::from_zoom(ZoomPercent::new(125).unwrap())
    );
}

#[test]
fn overrides_deserialize_missing_zoom_is_an_empty_record() {
    // A record without the zoom field (or the whole record absent) means
    // "no override": the empty record resolves to the documented default
    // and is never itself persisted.
    let empty: Vec<(&str, WireValue)> = Vec::new();
    let deserializer = MapDeserializer::<_, WireError>::new(empty.into_iter());
    let overrides = SiteSettingsOverrides::deserialize(deserializer).expect("empty wire record");
    assert!(overrides.is_empty());
    assert_eq!(overrides.zoom(), None);
}

#[test]
fn overrides_deserialize_explicit_null_zoom_is_an_empty_record() {
    // An explicit `{"zoom":null}` also means "no override".
    let deserializer =
        MapDeserializer::<_, WireError>::new(vec![("zoom", WireValue::Null)].into_iter());
    let overrides =
        SiteSettingsOverrides::deserialize(deserializer).expect("null zoom wire record");
    assert!(overrides.is_empty());
    assert_eq!(overrides.zoom(), None);
}

#[test]
fn overrides_deserialize_revalidates_zoom_bounds() {
    for invalid in [0, 24, 401] {
        let deserializer = MapDeserializer::<_, WireError>::new(
            vec![("zoom", WireValue::U64(invalid))].into_iter(),
        );
        assert!(
            SiteSettingsOverrides::deserialize(deserializer).is_err(),
            "wire zoom {invalid} must be rejected"
        );
    }
}
