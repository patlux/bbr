//! # Per-site overrides and effective resolution
//!
//! [`SiteSettingsOverrides`] is the sparse persisted record for one site: it
//! holds only the values the user explicitly changed. A missing record, or a
//! field missing from a record, means "use the documented default".
//!
//! [`EffectiveSiteSettings`] is the resolved view of one site: documented
//! defaults plus any stored override. The lifecycle guarantees that default
//! values are never stored as overrides, so resolution is a pure function of
//! the (possibly absent) sparse record.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::site_settings::ZoomPercent;

/// Sparse per-site setting overrides.
///
/// Zoom is currently the only per-site setting. The record is *sparse*: a
/// field is present only when the user changed it, so an override can never
/// silently store the default value. The empty record (`zoom: None`) means
/// "no overrides" and must never be persisted — the lifecycle removes the
/// record instead of storing it.
///
/// There is intentionally no public empty constructor: an empty record is a
/// meaningless persisted state and is only reachable through serde
/// deserialization (a wire value with no zoom field), which the lifecycle
/// and resolution both treat as "no override".
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SiteSettingsOverrides {
    zoom: Option<ZoomPercent>,
}

impl SiteSettingsOverrides {
    /// An override record for the zoom level.
    pub const fn from_zoom(zoom: ZoomPercent) -> Self {
        Self { zoom: Some(zoom) }
    }

    /// The overridden zoom level, or `None` when zoom is not overridden.
    pub const fn zoom(self) -> Option<ZoomPercent> {
        self.zoom
    }

    /// Whether this record overrides nothing and therefore must never be
    /// persisted.
    pub const fn is_empty(self) -> bool {
        self.zoom.is_none()
    }
}

/// Fully resolved per-site settings: documented defaults plus any stored
/// override for one site.
///
/// The default is the documented zoom default (100%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EffectiveSiteSettings {
    zoom: ZoomPercent,
}

impl EffectiveSiteSettings {
    /// Resolve the effective settings from the sparse stored override record.
    ///
    /// `None` (no stored record) and the empty record both resolve to the
    /// documented defaults; any overridden field wins over its default.
    pub fn resolve(overrides: Option<SiteSettingsOverrides>) -> Self {
        match overrides.and_then(SiteSettingsOverrides::zoom) {
            Some(zoom) => Self { zoom },
            None => Self::default(),
        }
    }

    /// The effective zoom level for the site.
    pub const fn zoom(self) -> ZoomPercent {
        self.zoom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zoom(percent: u16) -> ZoomPercent {
        ZoomPercent::new(percent).expect("fixture must be in bounds")
    }

    #[test]
    fn from_zoom_builds_a_non_empty_override() {
        let overrides = SiteSettingsOverrides::from_zoom(zoom(150));
        assert_eq!(overrides.zoom(), Some(zoom(150)));
        assert!(!overrides.is_empty());
    }

    #[test]
    fn resolve_none_uses_documented_defaults() {
        let effective = EffectiveSiteSettings::resolve(None);
        assert_eq!(effective.zoom(), ZoomPercent::default());
        assert_eq!(effective.zoom().as_u16(), 100);
    }

    #[test]
    fn resolve_override_wins_over_default() {
        let effective =
            EffectiveSiteSettings::resolve(Some(SiteSettingsOverrides::from_zoom(zoom(200))));
        assert_eq!(effective.zoom(), zoom(200));
    }

    #[test]
    fn effective_default_is_100_percent() {
        assert_eq!(EffectiveSiteSettings::default().zoom(), zoom(100));
    }

    #[test]
    fn overrides_are_copy_and_equality_is_exact() {
        let a = SiteSettingsOverrides::from_zoom(zoom(125));
        let b = a; // Copy
        assert_eq!(a, b);
        assert_ne!(a, SiteSettingsOverrides::from_zoom(zoom(150)));
    }

    #[test]
    fn debug_shapes_are_exact() {
        assert_eq!(
            format!("{:?}", SiteSettingsOverrides::from_zoom(zoom(125))),
            "SiteSettingsOverrides { zoom: Some(ZoomPercent(125)) }"
        );
        assert_eq!(
            format!("{:?}", EffectiveSiteSettings::default()),
            "EffectiveSiteSettings { zoom: ZoomPercent(100) }"
        );
    }
}
