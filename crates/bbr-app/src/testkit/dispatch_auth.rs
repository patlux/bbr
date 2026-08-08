//! Testkit routing for the credential sheet (queue item 10).
//!
//! One `Request` → `Response` arm per auth capability, split from
//! `super::dispatch` so no gateway module outgrows the ~500-line budget
//! (the same split pattern as `dispatch_find`).
//!
//! - `AuthPresentChallenge` drives the **production challenge-arrival
//!   seam** (`BbrApp::auth_on_challenge`): it builds a validated contract
//!   challenge from the fixture's authority metadata (host/port/realm/
//!   scheme/authority — never credential text) and presents the sheet.
//! - `AuthRecordChallenge` is the deterministic seeding seam
//!   (`BbrApp::auth_record_challenge`): it records the awaiting challenge
//!   WITHOUT presenting the sheet (the benchmark's fixture).
//! - `BenchmarkAuthSheetOpen` arms and starts the production presentation
//!   measurement (`BbrApp::benchmark_auth_sheet_open`); the render-inclusive
//!   commit is scheduled inside the app (the sheet renders in the one
//!   `NativeAboveWeb` portal window, so the main-window draw used by the
//!   other command benchmarks cannot present it).
//!
//! The sheet is presented through the portal, so every presentation request
//! flushes one bounded portal-window draw after the mutable `BbrApp` lease
//! is released (the same reentrancy rule as `dispatch_portal` / `dispatch_find`).

use bbr_testkit::protocol::{Error, Request, Response};
use bbr_web::auth_challenges::{AuthChallenge, ChallengeId, ChallengeKind, ProtectionSpace};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_portal::flush_portal_frame;
use super::dispatch_util::err_response;

/// Handle the auth `Request` arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_auth(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        // The production challenge-arrival seam: build the validated
        // contract challenge from the fixture's authority metadata and
        // present its sheet (the exact path a future WebKit delegate
        // drives). Credentials are never part of this request.
        Request::AuthPresentChallenge {
            profile,
            web_view,
            http_scheme,
            scheme,
            host,
            port,
            realm,
            authority,
            user_initiated,
        } => {
            let space =
                ProtectionSpace::new(*http_scheme, host, *port, realm.as_deref(), *authority);
            let space = match space {
                Ok(space) => space,
                Err(error) => {
                    return Some(Response::Error(Error::new(
                        "auth_space_invalid",
                        format!("protection space rejected: {error:?}"),
                    )))
                }
            };
            let challenge = AuthChallenge::new(
                // The coordinator mints the stable sequential id on record;
                // this id is a placeholder the contract re-keys.
                ChallengeId(0),
                *profile,
                *web_view,
                space,
                ChallengeKind::HttpAuth(*scheme),
                *user_initiated,
            );
            let challenge = match challenge {
                Ok(challenge) => challenge,
                Err(error) => {
                    return Some(Response::Error(Error::new(
                        "auth_challenge_invalid",
                        format!("challenge fixture rejected: {error:?}"),
                    )))
                }
            };
            with_app(window, app, cx, |view, window, cx| {
                let presented = view.auth_on_challenge(challenge, window, cx);
                (presented, view.portal_frame_handle())
            })
            .map(|(presented, handle)| {
                flush_portal_frame(handle, cx);
                if presented {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "auth_noop",
                        "challenge recorded but the sheet could not be presented",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        // Deterministic seeding: record the awaiting challenge WITHOUT
        // presenting the sheet (the fixture the measured open presents).
        Request::AuthRecordChallenge {
            profile,
            web_view,
            http_scheme,
            scheme,
            host,
            port,
            realm,
            authority,
            user_initiated,
        } => {
            let space =
                ProtectionSpace::new(*http_scheme, host, *port, realm.as_deref(), *authority);
            let space = match space {
                Ok(space) => space,
                Err(error) => {
                    return Some(Response::Error(Error::new(
                        "auth_space_invalid",
                        format!("protection space rejected: {error:?}"),
                    )))
                }
            };
            with_app(window, app, cx, |view, _window, _cx| {
                view.auth_record_challenge(*profile, *web_view, space, *scheme, *user_initiated)
            })
            .map(|recorded| {
                if recorded {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "auth_record_rejected",
                        "the challenge record was rejected by the contract",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        // The production credential-sheet open measurement: the app starts
        // the sample at `auth_present_sheet`, arms the exact awaiting
        // challenge identity, and schedules the render-inclusive commit
        // (the sheet renders in the portal window). A sample commits only
        // from a portal render that drew the sheet with the armed
        // challenge id — never from a frame that omitted it.
        Request::BenchmarkAuthSheetOpen { operation_id } => {
            with_app(window, app, cx, |view, window, cx| {
                view.benchmark_auth_sheet_open(*operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_noop",
                        "auth sheet open did not present the credential sheet",
                    ))
                }
            })
            .unwrap_or_else(err_response)
        }

        _ => return None,
    };
    Some(response)
}
