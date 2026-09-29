//! External-URL open composition (macOS native slice, queue item 13).
//!
//! The app-lifetime [`ExternalUrlAdapterCore`] (owned by [`BbrApp`]) is fed
//! one URL per OS delivery — either through GPUI's `application:openURLs:`
//! forwarding (`cx.on_open_urls` in the composition root, which also buffers
//! links arriving before the app finishes launching) or through the testkit
//! `DeliverExternalUrl` seam into the same channel. The decision is always
//! the merged portable contract's; this module only executes the typed
//! outcome through the existing controller, never through a second path.

use bbr_web::external_urls::{
    ExternalUrlOutcome, RoutingDecision, SpaceId as ExternalSpaceId, WindowId, WindowSnapshot,
};
use gpui::Context;

use super::BbrApp;

impl BbrApp {
    /// Handle one delivered external URL on the main thread: feed it through
    /// the app-lifetime coordinator (classification, burst dedup, routing —
    /// all contract-owned) and execute the typed outcome through the
    /// existing controller path.
    ///
    /// macOS-only: delivery arrives through the AppKit delegate path. The
    /// pump calls this only under `target_os = "macos"`.
    #[cfg(target_os = "macos")]
    pub fn on_external_url(
        &mut self,
        raw_url: String,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let snapshot = self.controller.snapshot();
        // The one browser window: always eligible (BBR has no private or
        // utility windows), presenting the controller's active space in its
        // own profile — frontmost by construction.
        let windows = [WindowSnapshot {
            window: WindowId(0),
            profile: snapshot.active_profile,
            space: ExternalSpaceId(snapshot.active_space.0),
            accepts_external_urls: true,
        }];
        // Caller-declared defaults for the contract's new-window fallback:
        // the first-created profile (the app's default browsing profile)
        // and its first space. Never guessed by the contract — declared by
        // the adapter, which is what the contract requires.
        let default_profile = snapshot
            .profiles
            .first()
            .map(|profile| profile.id)
            .unwrap_or(snapshot.active_profile);
        let default_space = snapshot
            .spaces
            .iter()
            .find(|space| space.profile == default_profile)
            .map(|space| ExternalSpaceId(space.id.0))
            .unwrap_or(ExternalSpaceId(snapshot.active_space.0));

        match self
            .external_urls
            .handle(raw_url.clone(), &windows, default_profile, default_space)
        {
            Ok(ExternalUrlOutcome::Routed { request, decision }) => {
                log::info!("external url routed (request {request:?}): {decision:?}");
                self.execute_external_decision(decision, &raw_url, window, cx);
            }
            Ok(ExternalUrlOutcome::Deduplicated {
                request,
                original,
                observations,
            }) => {
                // A burst repeat of the same URL: exactly one navigation
                // already happened; nothing more opens.
                log::info!(
                    "external url deduplicated (request {request:?} repeats {original:?}, \
                     {observations} total observations)"
                );
            }
            Ok(ExternalUrlOutcome::Rejected { request, reason }) => {
                // The contract rejected the URL (dangerous scheme,
                // unsupported scheme, or malformed). The reason's display
                // form is the privacy-reduced diagnostic; the URL is never
                // logged in full.
                log::warn!("external url rejected (request {request:?}): {reason}");
            }
            Err(error) => {
                // A contract violation by this adapter — never user input.
                log::error!("external url delivery violated the contract: {error:?}");
            }
        }
    }

    /// Execute the contract's routing decision through the existing
    /// controller: a reused window gets a fresh tab in its own profile and
    /// space; a new-window decision is unreachable in the single-window
    /// composition and is refused rather than disturbing the session.
    #[cfg(target_os = "macos")]
    fn execute_external_decision(
        &mut self,
        decision: RoutingDecision,
        url: &str,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        match decision {
            RoutingDecision::ReuseWindow {
                window: _window,
                profile,
                space,
            } => {
                // The window's own profile/space are the controller's current
                // ones (the window presents the active space), so the fresh
                // tab joins exactly this space — no profile leak, no space
                // change, no reordering of existing tabs. The assertions
                // pin the single-window invariant; the open itself runs
                // through the existing `open_tab` controller path.
                let snapshot = self.controller.snapshot();
                debug_assert_eq!(
                    space.0, snapshot.active_space.0,
                    "the routed space is the space the window presents"
                );
                debug_assert_eq!(
                    profile, snapshot.active_profile,
                    "the routed profile is the profile the window presents"
                );
                let _ = (profile, space);
                // Blueprint D13: the last active space, or the chooser.
                self.route_external_link(url, window, cx);
            }
            RoutingDecision::NewWindow { profile, space } => {
                // Unreachable: the one browser window is always eligible, so
                // the contract's fallback can never fire while it exists.
                // Silently opening in a different space or profile here
                // would violate the no-space-change invariant, and a true
                // second window is a later composition slice — refuse
                // loudly instead.
                log::warn!(
                    "external url requested a new window ({profile:?} / {space:?}) but BBR \
                     runs one always-eligible window; refusing the link rather than disturbing \
                     the session"
                );
            }
        }
    }

    /// User-triggered default-browser request: claims `http`/`https` for the
    /// main bundle under LaunchServices. Never invoked automatically — only
    /// the "Set as Default Browser" command routes here.
    pub fn request_default_browser(&mut self) {
        #[cfg(target_os = "macos")]
        {
            match bbr_web_macos::external_urls_launch_services::request_default_browser() {
                Ok(()) => log::info!(
                    "requested default-browser status for {}",
                    bbr_web_macos::external_urls_adapter::DECLARED_URL_SCHEMES.join(", ")
                ),
                Err(error) => log::warn!("could not request default-browser status: {error}"),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::warn!("default-browser request is macOS-only");
        }
    }
}
