//! Blueprint D12 (board M-11, DESIGN §18.4): the client-certificate sheet.
//!
//! A server's request arrives as [`WebEvent::ClientCertificateRequested`]
//! with the matching Keychain identities (metadata only). A remembered
//! choice for the profile and server answers at once (an identity only
//! while it is offered and valid); otherwise the sheet shows over the card
//! for the active page — one at a time, further requests wait. Continue
//! sends the chosen identity, Don't Send continues without one, Cancel
//! cancels the request; Remember keeps the answer for the site.

use std::collections::VecDeque;

use bbr_ui::surfaces::client_certificate::{
    CertificateRow, ClientCertificateEvent, ClientCertificateModel,
};
use bbr_web::client_certificates::{ClientCertificateAnswer, ClientIdentityInfo};
use bbr_web::{ProfileId, WebViewId};
use gpui::{Context, Window};

use super::history_page::time::Clock;
use super::BbrApp;
use crate::certificate_choices::{self, CertificateChoice, CertificateChoices};

/// One request waiting for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingCertificate {
    pub request: u64,
    pub view: WebViewId,
    pub profile: ProfileId,
    pub host: String,
    pub identities: Vec<ClientIdentityInfo>,
}

/// The app's client-certificate state.
#[derive(Default)]
pub(crate) struct CertificateState {
    pub choices: CertificateChoices,
    pub path: Option<std::path::PathBuf>,
    pub pending: VecDeque<PendingCertificate>,
    pub selected: Option<usize>,
    pub remember: bool,
    /// Views that started a load since their last URL report.
    pub loading: std::collections::BTreeSet<WebViewId>,
    /// Every answer handed to the engine (testkit evidence).
    #[cfg(feature = "testkit")]
    pub answers: Vec<(u64, ClientCertificateAnswer)>,
}

fn now_unix() -> i64 {
    (crate::store::history_ops::now_ms() / 1000) as i64
}

/// The first identity that can be chosen.
fn first_valid(identities: &[ClientIdentityInfo], now: i64) -> Option<usize> {
    identities
        .iter()
        .position(|identity| !identity.expired(now))
}

/// What a remembered choice answers for these identities (`None`: ask).
pub(crate) fn remembered_answer(
    choice: Option<&CertificateChoice>,
    identities: &[ClientIdentityInfo],
    now: i64,
) -> Option<ClientCertificateAnswer> {
    match choice? {
        CertificateChoice::None => Some(ClientCertificateAnswer::NoCertificate),
        CertificateChoice::Identity { fingerprint, .. } => identities
            .iter()
            .find(|identity| &identity.fingerprint == fingerprint && !identity.expired(now))
            .map(|identity| ClientCertificateAnswer::Identity(identity.fingerprint.clone())),
    }
}

impl BbrApp {
    pub(crate) fn load_certificate_choices(&mut self, path: std::path::PathBuf) {
        self.certificates.choices = certificate_choices::load(&path);
        self.certificates.path = Some(path);
    }

    pub(crate) fn save_certificate_choices(&self) {
        if let Some(path) = self.certificates.path.as_ref() {
            if let Err(error) = certificate_choices::save(path, &self.certificates.choices) {
                log::warn!("client-certificate choices not saved: {error}");
            }
        }
    }

    /// The engine event: answer from memory, or queue the sheet.
    pub(crate) fn on_client_certificate_request(
        &mut self,
        view: WebViewId,
        request: u64,
        host: &str,
        identities: &[ClientIdentityInfo],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(profile) = self.controller.profile_of_view(view) else {
            self.answer_certificate_engine(request, ClientCertificateAnswer::Cancel);
            return;
        };
        let remembered = remembered_answer(
            self.certificates.choices.get(profile, host),
            identities,
            now_unix(),
        );
        if let Some(answer) = remembered {
            self.answer_certificate_engine(request, answer);
            return;
        }
        self.certificates.pending.push_back(PendingCertificate {
            request,
            view,
            profile,
            host: host.to_string(),
            identities: identities.to_vec(),
        });
        self.sync_certificate_sheet(window, cx);
    }

    fn answer_certificate_engine(&mut self, request: u64, answer: ClientCertificateAnswer) {
        #[cfg(feature = "testkit")]
        self.certificates.answers.push((request, answer.clone()));
        self.engine
            .borrow_mut()
            .answer_client_certificate(request, answer);
    }

    /// The engine events that end waiting requests: a URL report is a
    /// navigation only when the URL is new or the view started a load since
    /// its last report — WebKit re-sends the same URL when a title lands,
    /// and that must not cancel a request.
    pub(crate) fn certificate_web_event(
        &mut self,
        event: &bbr_web::WebEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            bbr_web::WebEvent::LoadingChanged(view, true) => {
                self.certificates.loading.insert(*view);
            }
            bbr_web::WebEvent::UrlChanged(view, url) => {
                let started = self.certificates.loading.remove(view);
                if started || self.controller.view_url(*view) != Some(url.as_str()) {
                    self.certificate_navigation(*view, url, window, cx);
                }
            }
            _ => {}
        }
    }

    /// A navigation of `view` to another server cancels its waiting
    /// requests (their load is gone).
    fn certificate_navigation(
        &mut self,
        view: WebViewId,
        url: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let authority = crate::site_permissions::origin_of(url).map(|origin| {
            let rest = origin
                .split_once("://")
                .map_or(origin.as_str(), |(_, rest)| rest);
            rest.strip_suffix(":443").unwrap_or(rest).to_string()
        });
        let stale: Vec<u64> = self
            .certificates
            .pending
            .iter()
            .filter(|p| p.view == view && Some(&p.host) != authority.as_ref())
            .map(|p| p.request)
            .collect();
        if stale.is_empty() {
            return;
        }
        for request in stale {
            self.certificates.pending.retain(|p| p.request != request);
            self.answer_certificate_engine(request, ClientCertificateAnswer::Cancel);
        }
        self.certificates.selected = None;
        self.sync_certificate_sheet(window, cx);
    }

    /// The request the sheet shows now (the first one of the active page;
    /// requests of views that are gone are dropped — the engine cancelled
    /// them).
    pub(crate) fn current_certificate_request(&self) -> Option<&PendingCertificate> {
        let active = self.controller.snapshot().active_view?;
        self.certificates
            .pending
            .iter()
            .find(|p| p.view == active && self.controller.profile_of_view(p.view).is_some())
    }

    /// A sheet that just became current takes the keyboard (the page loses
    /// it) and selects its first valid identity.
    fn sync_certificate_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(current) = self.current_certificate_request() else {
            self.sync_portal_overlays(cx);
            return;
        };
        let selected = first_valid(&current.identities, now_unix());
        if self
            .certificates
            .selected
            .is_none_or(|index| index >= current.identities.len())
        {
            self.certificates.selected = selected;
        }
        self.engine.borrow_mut().focus_host_chrome();
        window.focus(&self.chrome_handle, cx);
        self.sync_portal_overlays(cx);
    }

    /// Whether the sheet shows.
    pub(crate) fn certificate_sheet_open(&self) -> bool {
        self.current_certificate_request().is_some()
    }

    /// Every sheet control (pointer, keys and the testkit).
    pub(crate) fn on_client_certificate_event(
        &mut self,
        event: ClientCertificateEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(current) = self.current_certificate_request().cloned() else {
            return;
        };
        let now = now_unix();
        let answer = match event {
            ClientCertificateEvent::Select(index) => {
                if current
                    .identities
                    .get(index)
                    .is_some_and(|identity| !identity.expired(now))
                {
                    self.certificates.selected = Some(index);
                }
                self.sync_portal_overlays(cx);
                return;
            }
            ClientCertificateEvent::Remember => {
                self.certificates.remember = !self.certificates.remember;
                self.sync_portal_overlays(cx);
                return;
            }
            ClientCertificateEvent::Continue => {
                let Some(identity) = self
                    .certificates
                    .selected
                    .and_then(|index| current.identities.get(index))
                    .filter(|identity| !identity.expired(now))
                else {
                    return;
                };
                if self.certificates.remember {
                    self.certificates.choices.set(
                        current.profile,
                        &current.host,
                        CertificateChoice::Identity {
                            fingerprint: identity.fingerprint.clone(),
                            name: identity.name.clone(),
                        },
                    );
                }
                ClientCertificateAnswer::Identity(identity.fingerprint.clone())
            }
            ClientCertificateEvent::DontSend => {
                if self.certificates.remember {
                    self.certificates.choices.set(
                        current.profile,
                        &current.host,
                        CertificateChoice::None,
                    );
                }
                ClientCertificateAnswer::NoCertificate
            }
            ClientCertificateEvent::Cancel => ClientCertificateAnswer::Cancel,
        };
        if self.certificates.remember && answer != ClientCertificateAnswer::Cancel {
            self.save_certificate_choices();
        }
        self.certificates
            .pending
            .retain(|p| p.request != current.request);
        self.certificates.selected = None;
        self.certificates.remember = false;
        self.answer_certificate_engine(current.request, answer);
        if self.certificate_sheet_open() {
            self.sync_certificate_sheet(window, cx);
        } else {
            self.sync_portal_overlays(cx);
            self.focus_web(window, cx);
        }
    }

    /// The sheet's keyboard: ↑↓ move over the valid rows, Return continues.
    /// Escape is the global binding (it cancels).
    pub(crate) fn on_certificate_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(current) = self.current_certificate_request() else {
            return;
        };
        let now = now_unix();
        let valid: Vec<usize> = current
            .identities
            .iter()
            .enumerate()
            .filter(|(_, identity)| !identity.expired(now))
            .map(|(index, _)| index)
            .collect();
        match event.keystroke.key.as_str() {
            "up" | "down" if !valid.is_empty() => {
                cx.stop_propagation();
                let at = self
                    .certificates
                    .selected
                    .and_then(|selected| valid.iter().position(|index| *index == selected));
                let next = match (at, event.keystroke.key == "down") {
                    (None, _) => 0,
                    (Some(at), true) => (at + 1).min(valid.len() - 1),
                    (Some(at), false) => at.saturating_sub(1),
                };
                self.certificates.selected = Some(valid[next]);
                self.sync_portal_overlays(cx);
            }
            "enter" => {
                cx.stop_propagation();
                self.on_client_certificate_event(ClientCertificateEvent::Continue, window, cx);
            }
            _ => {}
        }
    }

    /// The sheet's model for the Portal.
    pub(crate) fn client_certificate_model(&self) -> Option<ClientCertificateModel> {
        let current = self.current_certificate_request()?;
        let profile = self
            .controller
            .snapshot()
            .profiles
            .iter()
            .find(|p| p.id == current.profile)
            .map(|p| p.title.clone())
            .unwrap_or_default();
        let clock = Clock::system();
        let now = now_unix();
        Some(ClientCertificateModel {
            message: format!(
                "{} asks for a certificate to identify you. Profile · {profile}.",
                current.host
            )
            .into(),
            rows: current
                .identities
                .iter()
                .map(|identity| {
                    let expired = identity.expired(now);
                    let when = identity.expires_unix.map(|expires| {
                        let date = clock.short_date(expires);
                        if expired {
                            format!("expired {date}")
                        } else {
                            format!("expires {date}")
                        }
                    });
                    let detail = match (identity.issuer.is_empty(), when) {
                        (false, Some(when)) => format!("{} · {when}", identity.issuer),
                        (false, None) => identity.issuer.clone(),
                        (true, Some(when)) => when,
                        (true, None) => String::new(),
                    };
                    CertificateRow {
                        name: identity.name.clone().into(),
                        detail: detail.into(),
                        expired,
                    }
                })
                .collect(),
            selected: self.certificates.selected,
            remember: self.certificates.remember,
        })
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    pub(crate) fn client_certificates_snapshot(
        &self,
    ) -> bbr_testkit::protocol::ClientCertificatesSnapshot {
        let sheet = self.current_certificate_request().and_then(|current| {
            let model = self.client_certificate_model()?;
            Some(bbr_testkit::protocol::ClientCertificateSnapshot {
                request: current.request,
                message: model.message.to_string(),
                rows: model
                    .rows
                    .iter()
                    .map(|row| (row.name.to_string(), row.detail.to_string(), row.expired))
                    .collect(),
                selected: model.selected,
                remember: model.remember,
                queued: self.certificates.pending.len() - 1,
            })
        });
        bbr_testkit::protocol::ClientCertificatesSnapshot {
            sheet,
            answers: self
                .certificates
                .answers
                .iter()
                .map(|(request, answer)| {
                    let answer = match answer {
                        ClientCertificateAnswer::Identity(fp) => format!("identity:{fp}"),
                        ClientCertificateAnswer::NoCertificate => "none".to_string(),
                        ClientCertificateAnswer::Cancel => "cancel".to_string(),
                    };
                    (*request, answer)
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(fingerprint: &str, expires: i64) -> ClientIdentityInfo {
        ClientIdentityInfo {
            fingerprint: fingerprint.into(),
            name: fingerprint.to_uppercase(),
            issuer: "Corp CA".into(),
            expires_unix: Some(expires),
        }
    }

    #[test]
    fn remembered_choices_answer_only_with_an_offered_valid_identity() {
        let identities = [identity("aa", 200), identity("bb", 50)];
        let choose = |fp: &str| CertificateChoice::Identity {
            fingerprint: fp.into(),
            name: String::new(),
        };
        assert_eq!(remembered_answer(None, &identities, 100), None);
        assert_eq!(
            remembered_answer(Some(&choose("aa")), &identities, 100),
            Some(ClientCertificateAnswer::Identity("aa".into()))
        );
        assert_eq!(
            remembered_answer(Some(&choose("bb")), &identities, 100),
            None,
            "expired"
        );
        assert_eq!(
            remembered_answer(Some(&choose("cc")), &identities, 100),
            None,
            "not offered"
        );
        assert_eq!(
            remembered_answer(Some(&CertificateChoice::None), &identities, 100),
            Some(ClientCertificateAnswer::NoCertificate)
        );
        assert_eq!(
            first_valid(&[identity("x", 10), identity("y", 500)], 100),
            Some(1)
        );
    }
}
