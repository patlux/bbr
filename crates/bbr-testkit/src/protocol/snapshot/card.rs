//! DESIGN.md §2 seated content card evidence on the wire.

use serde::{Deserialize, Serialize};

/// The content card frame the render laid out for the current window state
/// and the native clip the hosting layer applied. The native slot rect
/// itself is the snapshot's `content_rect` / `pane_rects` (the card's inner
/// box).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ContentCardSnapshot {
    /// `sp:content-inset`, or the native titlebar band while the sidebar
    /// is collapsed; `0` in native full screen.
    pub inset_top: f32,
    pub inset_trailing: f32,
    pub inset_bottom: f32,
    /// `0` with the rail (flush), `sp:content-inset` while collapsed.
    pub inset_leading: f32,
    /// `radius:lg`; `0` in native full screen.
    pub radius: f32,
    /// Whether the static `elev:1.content` shadow paints.
    pub shadow: bool,
    /// The window is in native full screen (full-bleed card).
    pub full_screen: bool,
    /// The active WKWebView backing layer's corner radius — the native clip
    /// evidence; `None` without a live native view.
    pub native_radius: Option<f32>,
    #[serde(default)]
    pub chrome_material: Option<ChromeMaterialSnapshot>,
}

/// Native material observations, alongside the effective chrome policy.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ChromeMaterialSnapshot {
    pub attached: bool,
    pub enabled: bool,
    pub card_mask_valid: bool,
    pub behind_window: bool,
    pub follows_activation: bool,
    pub window_opaque: bool,
    pub size: [f32; 2],
    pub tint_alpha: f32,
    pub reduce_transparency: bool,
    pub web_view_opaque: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{dsl, Request};

    #[test]
    fn chrome_material_and_transparency_override_round_trip() {
        let material = ChromeMaterialSnapshot {
            attached: true,
            enabled: true,
            card_mask_valid: true,
            behind_window: true,
            follows_activation: true,
            window_opaque: false,
            size: [1080.0, 700.0],
            tint_alpha: 0.9,
            reduce_transparency: false,
            web_view_opaque: Some(true),
        };
        let value = serde_json::to_value(material).unwrap();
        assert_eq!(
            serde_json::from_value::<ChromeMaterialSnapshot>(value).unwrap(),
            material
        );
        let request = dsl::set_display_accessibility(Some(false), None, Some(true));
        assert_eq!(
            serde_json::from_value::<Request>(serde_json::to_value(&request).unwrap()).unwrap(),
            request
        );
        let mut old_card = serde_json::to_value(ContentCardSnapshot::default()).unwrap();
        old_card.as_object_mut().unwrap().remove("chrome_material");
        assert_eq!(
            serde_json::from_value::<ContentCardSnapshot>(old_card).unwrap(),
            ContentCardSnapshot::default(),
        );
        let old = serde_json::json!({"SetAccessibilityOverride": {"increase_contrast": false, "reduce_motion": null}});
        assert_eq!(
            serde_json::from_value::<Request>(old).unwrap(),
            dsl::set_display_accessibility(Some(false), None, None)
        );
    }
}
