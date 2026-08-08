//! Adapter between the portable external-URL open contract and the app.
//!
//! This module contains **no** AppKit/WebKit imports and no `unsafe` code,
//! so the whole delivery mapping can be contract-tested without a display
//! server — the same split the downloads slice uses (`downloads_mapping`).
//!
//! The decision logic lives in the merged portable contract
//! (`bbr_web::external_urls`); this adapter only **delivers**: it builds the
//! [`ExternalUrlRequest`], keeps the request-id and tick watermarks the
//! coordinator requires, feeds the coordinator the app's current window
//! state, and hands the typed [`ExternalUrlOutcome`] back to the caller to
//! execute. It never re-classifies, re-decides, or re-orders anything.
//!
//! ## Delivery discipline
//!
//! - **One coordinator per app lifetime.** The adapter owns the
//!   [`ExternalUrlCoordinator`] and never reuses a request id or a tick
//!   (the coordinator rejects both as contract violations).
//! - **Ticks are per-delivery counters.** The contract's burst window is
//!   measured in caller-supplied monotonic ticks; the adapter supplies the
//!   delivery count. The burst configuration (3 ticks, 8 remembered URLs)
//!   matches the coordinator's own contract evidence: it absorbs an OS
//!   double-delivery burst while a repeat of an old URL stays fresh after a
//!   few unrelated deliveries.
//! - **No auto-navigation.** A [`ExternalUrlOutcome::Routed`] decision is
//!   returned, never executed here — execution runs through the existing
//!   controller in the composition root, exactly once.

use bbr_web::external_urls::{
    BurstConfig, ExternalUrlCoordinator, ExternalUrlError, ExternalUrlOutcome, ExternalUrlRequest,
    ExternalUrlRequestId, RoutingInput, WindowSnapshot,
};
use bbr_web::ProfileId;

/// The only URL schemes BBR declares as its own and requests default-handler
/// status for: exactly the supported web schemes of the merged contract
/// (`http`, `https`). Claiming any other scheme (a scheme the contract
/// classifies dangerous, unsupported, or malformed) would make BBR a black
/// hole for the user's links — the OS dispatches a registered scheme to BBR
/// and the contract then rejects it. This constant is the single source for
/// the adapter and the packaging declaration test.
pub const DECLARED_URL_SCHEMES: [&str; 2] = ["http", "https"];

/// The burst configuration for the app-lifetime coordinator: a 3-tick window
/// absorbs the OS double-delivery burst, and 8 remembered URLs bound
/// retention (oldest-first eviction; privacy-reduced keys only). Mirrors the
/// coordinator contract tests' own evidence configuration.
pub fn adapter_burst_config() -> BurstConfig {
    BurstConfig::new(3, 8).expect("3 ticks and 8 remembered URLs are a valid burst config")
}

/// The app-lifetime external-URL delivery store: owns the coordinator and
/// the request-id/tick watermarks the contract requires.
#[derive(Debug, Clone)]
pub struct ExternalUrlAdapterCore {
    coordinator: ExternalUrlCoordinator,
    next_request_id: u64,
    next_tick: u64,
}

impl ExternalUrlAdapterCore {
    /// A delivery store with the adapter's burst configuration.
    pub fn new() -> Self {
        Self::with_config(adapter_burst_config())
    }

    /// A delivery store with a caller-chosen burst configuration (contract
    /// tests use smaller windows; production uses [`ADAPTER_BURST_CONFIG`]).
    pub fn with_config(config: BurstConfig) -> Self {
        Self {
            coordinator: ExternalUrlCoordinator::new(config),
            next_request_id: 1,
            next_tick: 0,
        }
    }

    /// Deliver one OS-dispatched external URL open against the app's current
    /// window state.
    ///
    /// `windows` is the caller's front-to-back window order (the composition
    /// root reports its one always-eligible browser window); `default_profile`
    /// and `default_space` are the caller-declared defaults a new-window
    /// decision carries. The decision itself is always the contract's —
    /// never this adapter's.
    pub fn handle(
        &mut self,
        raw_url: String,
        windows: &[WindowSnapshot],
        default_profile: ProfileId,
        default_space: bbr_web::external_urls::SpaceId,
    ) -> Result<ExternalUrlOutcome, ExternalUrlError> {
        let request = ExternalUrlRequest {
            id: ExternalUrlRequestId(self.next_request_id),
            raw_url,
            // The OS delivers no activation intent through the current
            // AppKit path (GPUI's `application:openURLs:` forwarding); the
            // routing rule never depends on it.
            activation: None,
        };
        let input = RoutingInput {
            windows,
            default_profile,
            default_space,
        };
        let outcome = self.coordinator.handle(request, self.next_tick, &input);
        self.next_request_id += 1;
        self.next_tick += 1;
        outcome
    }

    /// The next request id this adapter would allocate (diagnostics only).
    pub fn next_request_id(&self) -> u64 {
        self.next_request_id
    }
}

impl Default for ExternalUrlAdapterCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_web::external_urls::{
        classify, ExternalUrlRequest, MalformedUrl, RejectionReason, RoutingDecision, WindowId,
    };

    fn snapshot(window: u64, profile: u64, space: u64, accepts: bool) -> WindowSnapshot {
        WindowSnapshot {
            window: WindowId(window),
            profile: ProfileId(profile),
            space: bbr_web::external_urls::SpaceId(space),
            accepts_external_urls: accepts,
        }
    }

    fn window(id: u64) -> WindowSnapshot {
        snapshot(id, 10, 100, true)
    }

    /// The declared schemes are exactly the contract's supported web schemes:
    /// every declared scheme classifies as a web URL, and the dangerous /
    /// unsupported scheme sets are not declared. Claiming a rejected scheme
    /// would make BBR a black hole for links the contract refuses.
    #[test]
    fn declared_schemes_match_the_contract() {
        for scheme in DECLARED_URL_SCHEMES {
            assert!(
                matches!(
                    classify(&format!("{scheme}://example.com/")),
                    bbr_web::external_urls::UrlClass::Web(_)
                ),
                "{scheme} must be a supported web scheme"
            );
        }
        for dangerous in ["javascript", "data", "file", "about", "vbscript"] {
            assert!(
                !DECLARED_URL_SCHEMES.contains(&dangerous),
                "dangerous scheme {dangerous} must never be declared"
            );
        }
        for unsupported in ["mailto", "tel", "ftp"] {
            assert!(
                !DECLARED_URL_SCHEMES.contains(&unsupported),
                "unsupported scheme {unsupported} must never be declared"
            );
        }
    }

    /// A delivered web URL routes into the frontmost eligible window with the
    /// window's own profile and space — never a cross-profile move.
    #[test]
    fn routes_web_url_into_the_frontmost_eligible_window() {
        let mut adapter = ExternalUrlAdapterCore::new();
        let windows = [snapshot(7, 10, 100, true)];
        let outcome = adapter
            .handle(
                "https://example.com/page".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("valid delivery");
        assert_eq!(
            outcome,
            ExternalUrlOutcome::Routed {
                request: ExternalUrlRequestId(1),
                decision: RoutingDecision::ReuseWindow {
                    window: WindowId(7),
                    profile: ProfileId(10),
                    space: bbr_web::external_urls::SpaceId(100),
                },
            }
        );
    }

    /// An ineligible window is skipped: the decision falls back to a new
    /// window in the caller-declared default profile/space.
    #[test]
    fn ineligible_window_falls_back_to_a_new_window_in_declared_defaults() {
        let mut adapter = ExternalUrlAdapterCore::new();
        let windows = [snapshot(7, 10, 100, false)];
        let outcome = adapter
            .handle(
                "https://example.com/page".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("valid delivery");
        assert_eq!(
            outcome,
            ExternalUrlOutcome::Routed {
                request: ExternalUrlRequestId(1),
                decision: RoutingDecision::NewWindow {
                    profile: ProfileId(99),
                    space: bbr_web::external_urls::SpaceId(999),
                },
            }
        );
    }

    /// A repeat of the same URL within the burst window is deduplicated, not
    /// routed twice; a different URL routes on its own.
    #[test]
    fn burst_duplicates_are_suppressed_through_the_store() {
        let mut adapter = ExternalUrlAdapterCore::new();
        let windows = [window(1)];
        let first = adapter
            .handle(
                "https://example.com/a".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("first delivery");
        assert!(matches!(first, ExternalUrlOutcome::Routed { .. }));
        let second = adapter
            .handle(
                "https://example.com/a".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("duplicate delivery");
        assert_eq!(
            second,
            ExternalUrlOutcome::Deduplicated {
                request: ExternalUrlRequestId(2),
                original: ExternalUrlRequestId(1),
                observations: 2,
            }
        );
        let different = adapter
            .handle(
                "https://example.com/b".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("different delivery");
        assert!(matches!(different, ExternalUrlOutcome::Routed { .. }));
    }

    /// Dangerous schemes are rejected by the contract and never routed.
    #[test]
    fn dangerous_schemes_are_rejected_never_opened() {
        let mut adapter = ExternalUrlAdapterCore::new();
        let windows = [window(1)];
        let outcome = adapter
            .handle(
                "javascript:alert(1)".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("delivery is handled without panicking");
        assert_eq!(
            outcome,
            ExternalUrlOutcome::Rejected {
                request: ExternalUrlRequestId(1),
                reason: RejectionReason::DangerousScheme {
                    scheme: "javascript".into(),
                },
            }
        );
    }

    /// Malformed input is rejected with the fixed placeholder, never guessed.
    #[test]
    fn malformed_input_is_rejected() {
        let mut adapter = ExternalUrlAdapterCore::new();
        let windows = [window(1)];
        let outcome = adapter
            .handle(
                "no scheme here".to_string(),
                &windows,
                ProfileId(99),
                bbr_web::external_urls::SpaceId(999),
            )
            .expect("delivery is handled without panicking");
        assert_eq!(
            outcome,
            ExternalUrlOutcome::Rejected {
                request: ExternalUrlRequestId(1),
                reason: RejectionReason::Malformed(MalformedUrl::InvalidCharacters),
            }
        );
    }

    /// Request ids stay strictly increasing for the adapter's lifetime, so
    /// the coordinator's duplicate-request guard can never fire from here.
    #[test]
    fn request_ids_are_strictly_increasing() {
        let mut adapter = ExternalUrlAdapterCore::new();
        let windows = [window(1)];
        let mut last = 0u64;
        for _ in 0..5 {
            let outcome = adapter
                .handle(
                    "https://example.com/x".to_string(),
                    &windows,
                    ProfileId(99),
                    bbr_web::external_urls::SpaceId(999),
                )
                .expect("valid delivery");
            match outcome {
                ExternalUrlOutcome::Routed { request, .. } => {
                    assert!(request.0 > last);
                    last = request.0;
                }
                // The identical URL is a burst duplicate after the first; the
                // id watermark still advances.
                ExternalUrlOutcome::Deduplicated { request, .. } => {
                    assert!(request.0 > last);
                    last = request.0;
                }
                ExternalUrlOutcome::Rejected { .. } => unreachable!("web url"),
            }
        }
        assert_eq!(adapter.next_request_id(), 6);
    }

    /// `ExternalUrlRequest` is constructible with the same shape the contract
    /// consumes (compile-time wire check for the adapter seam).
    #[test]
    fn request_shape_matches_the_contract() {
        let request = ExternalUrlRequest {
            id: ExternalUrlRequestId(1),
            raw_url: "https://example.com/".to_string(),
            activation: None,
        };
        assert_eq!(request.raw_url, "https://example.com/");
        assert_eq!(request.id, ExternalUrlRequestId(1));
    }
}
