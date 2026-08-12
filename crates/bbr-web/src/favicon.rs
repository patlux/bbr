//! # Favicon acquisition — portable per-page favicon contract
//!
//! Typed vocabulary and the deterministic best-candidate rule for real
//! favicon acquisition (task packet `real-favicon-pipeline.md`). No
//! framework, GPUI, `unsafe` code, or dependencies.
//!
//! The engine boundary carries a typed per-page favicon event:
//! [`crate::WebEvent::FaviconChanged`] with a [`FaviconContent`] payload
//! that is either the **candidate URLs** collected from the committed page
//! or the **resolved image bytes** of the chosen best candidate. The
//! contract is engine-agnostic: the macOS adapter implements it (an
//! isolated content-world script collects `<link rel~="icon">` candidates
//! with `/favicon.ico` as the final fallback, one best candidate is chosen
//! by [`choose_best_candidate`], and the download runs through a session
//! bound to the owning profile's data store), and the deterministic fake
//! engine proves the boundary without a display server.
//!
//! ## Deterministic selection rule
//!
//! [`choose_best_candidate`] prefers the declared size closest to the
//! target (`2×` the density icon size — 36 pt for the normal 18 pt tile,
//! per DESIGN §8), ties breaking in document order. Candidates without a
//! declared size rank after every sized candidate (the first unsized
//! candidate in document order wins when no sized candidate exists).
//! `None` for an empty candidate list — the caller then applies its
//! `/favicon.ico` fallback (the macOS adapter) or falls back silently to
//! the §7.2 monogram (any failure).

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One declared favicon candidate collected from the committed page
/// (`<link rel~="icon" href="…">`), with its declared square size.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaviconCandidate {
    /// Absolute URL of the icon (the `href`, resolved against the page).
    pub url: String,
    /// The declared square size in px (from the `sizes` attribute; the
    /// largest square token when several are declared), `None` when the
    /// link declares no size.
    pub size: Option<u32>,
}

/// The typed per-page favicon payload on the engine boundary.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaviconContent {
    /// Candidate URLs in document order, before any download. Emitted by
    /// adapters that hand candidate selection to a downstream authority;
    /// the macOS adapter selects internally and never emits this shape.
    Candidates(Vec<FaviconCandidate>),
    /// Resolved image bytes of the chosen best candidate, ready for decode
    /// at the presentation layer. The macOS adapter emits this shape after
    /// a successful bounded download; any failure is silent (no event), so
    /// the §7.2 monogram stays the deterministic fallback.
    Image(Vec<u8>),
}

/// Choose the best favicon candidate by the deterministic acquisition rule:
/// among candidates with a declared size, the one whose size is closest to
/// `target_size` (ties break in document order); when no candidate declares
/// a size, the first candidate in document order. `None` when the list is
/// empty (the caller applies its `/favicon.ico` fallback).
///
/// The rule is deterministic across declared sizes, `rel` variants (the
/// page script normalizes `rel~="icon"` to a flat list) and missing links
/// (an empty list yields `None`).
pub fn choose_best_candidate(candidates: &[FaviconCandidate], target_size: u32) -> Option<usize> {
    let mut best: Option<(usize, u32)> = None; // (index, |size - target|)
    for (index, candidate) in candidates.iter().enumerate() {
        match candidate.size {
            Some(size) => {
                let distance = size.abs_diff(target_size);
                if best
                    .map(|(_, best_distance)| distance < best_distance)
                    .unwrap_or(true)
                {
                    best = Some((index, distance));
                }
            }
            // Candidates without a declared size rank after every sized
            // candidate: they win only when no sized candidate exists at all
            // (any sized candidate processed earlier already set `best`), so
            // the first unsized candidate in document order is selected.
            None => {
                if best.is_none() {
                    best = Some((index, u32::MAX));
                }
            }
        }
    }
    best.map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(url: &str, size: Option<u32>) -> FaviconCandidate {
        FaviconCandidate {
            url: url.into(),
            size,
        }
    }

    #[test]
    fn empty_candidates_select_none() {
        assert_eq!(choose_best_candidate(&[], 36), None);
    }

    #[test]
    fn closest_declared_size_wins() {
        let candidates = [
            candidate("https://a/16.png", Some(16)),
            candidate("https://a/32.png", Some(32)),
            candidate("https://a/64.png", Some(64)),
        ];
        // Target 36 (2× the normal 18 pt icon): 32 is the closest.
        assert_eq!(choose_best_candidate(&candidates, 36), Some(1));
        // A different target flips the choice deterministically.
        assert_eq!(choose_best_candidate(&candidates, 64), Some(2));
        assert_eq!(choose_best_candidate(&candidates, 16), Some(0));
    }

    #[test]
    fn ties_break_in_document_order() {
        let candidates = [
            candidate("https://a/one.png", Some(32)),
            candidate("https://a/two.png", Some(32)),
        ];
        assert_eq!(choose_best_candidate(&candidates, 36), Some(0));
        let candidates = [
            candidate("https://a/one.png", Some(64)),
            candidate("https://a/two.png", Some(64)),
            candidate("https://a/three.png", Some(64)),
        ];
        assert_eq!(choose_best_candidate(&candidates, 36), Some(0));
    }

    #[test]
    fn unsized_candidates_rank_after_sized_ones() {
        let candidates = [
            candidate("https://a/unsized.ico", None),
            candidate("https://a/16.png", Some(16)),
            candidate("https://a/32.png", Some(32)),
        ];
        // A sized candidate wins over an earlier unsized one.
        assert_eq!(choose_best_candidate(&candidates, 36), Some(2));
        // Only unsized candidates: the first in document order wins.
        let candidates = [
            candidate("https://a/second.ico", None),
            candidate("https://a/first.ico", None),
        ];
        assert_eq!(choose_best_candidate(&candidates, 36), Some(0));
    }

    #[test]
    fn selection_is_stable_across_rel_and_size_variants() {
        // The rule only reads url + declared size: any rel/size parsing
        // upstream (the page script) produces the same flat list, so the
        // outcome is deterministic across variants of the same page.
        let apple = [
            candidate("https://a/apple-touch-icon.png", Some(180)),
            candidate("https://a/favicon-32x32.png", Some(32)),
            candidate("https://a/favicon-16x16.png", Some(16)),
        ];
        let plain = [
            candidate("https://a/favicon-32x32.png", Some(32)),
            candidate("https://a/favicon-16x16.png", Some(16)),
        ];
        assert_eq!(choose_best_candidate(&apple, 36), Some(1));
        assert_eq!(choose_best_candidate(&plain, 36), Some(0));
    }
}
