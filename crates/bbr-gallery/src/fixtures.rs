//! Fixture builders for the G1b Foundations stories (packet
//! `component-gallery-mvp.md`): plain `bbr-ui` DTOs with deterministic,
//! stable values. Every fixture is valid for its `bbr-ui` component (fields
//! non-empty, ids stable) — proven by the fixture-builder tests below.

use bbr_domain::ProfileHue;
use bbr_ui::components::{FaviconSpec, ProfileChipSpec, StatusModel};

/// A deterministic favicon spec for a known hostname (the §7.2 monogram
/// tile): the first letter of the registrable host, uppercased, on the
/// FNV-1a slate ramp.
pub fn favicon_spec(host: &str) -> FaviconSpec {
    let letter = host
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?');
    let mut hash = 0x811c9dc5u32;
    for byte in host.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    FaviconSpec {
        letter,
        hue: (hash as usize) % 6,
        image: None,
    }
}

/// The stable fixture hosts the favicon story renders.
pub const FAVICON_HOSTS: [&str; 4] = [
    "example.com",
    "github.com",
    "news.ycombinator.com",
    "localhost",
];

/// A profile chip spec for one §12 hue: a 1–2 character monogram derived
/// deterministically from the hue name.
pub fn profile_chip_spec(hue: ProfileHue) -> ProfileChipSpec {
    ProfileChipSpec {
        letter: monogram_for_hue(hue).to_string(),
        hue,
    }
}

/// All six §12 hues in round-robin order — the profile-chip story renders
/// one chip per hue.
pub const ALL_HUES: [ProfileHue; 6] = [
    ProfileHue::Cobalt,
    ProfileHue::Violet,
    ProfileHue::Fuchsia,
    ProfileHue::Amber,
    ProfileHue::Terracotta,
    ProfileHue::Slate,
];

/// A deterministic 1–2 character monogram per hue (the §12 non-color
/// identity companion).
pub fn monogram_for_hue(hue: ProfileHue) -> &'static str {
    match hue {
        ProfileHue::Cobalt => "Co",
        ProfileHue::Violet => "Vi",
        ProfileHue::Fuchsia => "Fu",
        ProfileHue::Amber => "Am",
        ProfileHue::Terracotta => "Te",
        ProfileHue::Slate => "Sl",
    }
}

/// The §10 state vocabulary — the States page's state selector options and
/// the favicon tile story's `TabVisualState` options.
pub const STATE_VOCABULARY: [&str; 11] = [
    "default",
    "hover",
    "active",
    "selected",
    "loading",
    "attention",
    "error",
    "disabled",
    "dragging",
    "drag-target",
    "focused",
];

/// The favicon tile story's state options (a subset of the §10 vocabulary
/// the tile actually renders).
pub const FAVICON_STATES: [&str; 3] = ["default", "loading", "error"];

/// A status-line model fixture: the plain text the §11.4 line renders.
/// The gallery builds the model directly — it never imports the app's
/// controller-snapshot derivation.
pub fn status_model(text: &str) -> StatusModel {
    StatusModel { text: text.into() }
}

/// The status-line story variants: the §11.8 derivation branches as plain
/// fixture text (empty, loading, title, URL fallback, crash).
pub const STATUS_VARIANTS: [&str; 5] = [
    "No page",
    "Loading…",
    "Example",
    "https://example.com/path",
    "Web content process terminated",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favicon_fixtures_are_valid_and_stable() {
        for host in FAVICON_HOSTS {
            let spec = favicon_spec(host);
            assert!(
                spec.letter.is_ascii_uppercase(),
                "monogram letter is uppercase for {host}"
            );
            assert!(
                (0..6).contains(&spec.hue),
                "slate hue index in range for {host}"
            );
            // Stable: rebuilding yields the identical spec.
            assert_eq!(spec, favicon_spec(host));
        }
        // Deterministic per host: different hosts differ in letter or hue.
        let a = favicon_spec("example.com");
        let b = favicon_spec("github.com");
        assert!(a != b, "distinct hosts yield distinct specs");
    }

    #[test]
    fn profile_chip_fixtures_are_valid_and_stable() {
        for hue in ALL_HUES {
            let spec = profile_chip_spec(hue);
            assert!(!spec.letter.is_empty(), "monogram non-empty for {hue:?}");
            assert_eq!(spec.hue, hue);
            assert_eq!(spec, profile_chip_spec(hue), "stable per hue");
        }
        // Every hue maps to a distinct monogram.
        let mut letters: Vec<&str> = ALL_HUES.iter().map(|&h| monogram_for_hue(h)).collect();
        letters.sort_unstable();
        letters.dedup();
        assert_eq!(letters.len(), ALL_HUES.len(), "monograms are distinct");
    }

    #[test]
    fn state_vocabulary_is_the_section_10_names() {
        assert_eq!(STATE_VOCABULARY.len(), 11);
        assert_eq!(STATE_VOCABULARY[0], "default");
        assert_eq!(STATE_VOCABULARY[4], "loading");
        assert_eq!(STATE_VOCABULARY[6], "error");
        assert_eq!(STATE_VOCABULARY[9], "drag-target");
        assert_eq!(FAVICON_STATES, ["default", "loading", "error"]);
    }

    #[test]
    fn status_fixtures_are_valid_and_stable() {
        for text in STATUS_VARIANTS {
            let model = status_model(text);
            assert_eq!(model.text, text, "fixture text round-trips");
            assert_eq!(model, status_model(text), "stable per text");
        }
        // Distinct texts yield distinct models.
        assert_ne!(status_model("No page"), status_model("Example"));
        // The crash line stays plain ASCII (DESIGN.md §14: no emoji).
        assert!(status_model("Web content process terminated")
            .text
            .is_ascii());
    }
}
