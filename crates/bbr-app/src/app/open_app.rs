//! Blueprint D13 (board M-12, DESIGN §11.13): a page's link to another
//! app's scheme (`mailto:`, `zoommtg:`, `slack:`) asks before it leaves
//! the browser.
//!
//! The engine cancels such navigations and reports them. With no app for
//! the scheme, a quiet banner says so ("No app on this Mac opens “foo:”
//! links.") instead of failing silently. Otherwise the sheet names the app
//! and the site that asked; Open hands the link to macOS, Cancel drops it,
//! and "Always allow" remembers the site and scheme per profile in
//! `site-permissions.txt`, so later links open without asking. One sheet
//! at a time: further requests while it shows are dropped.

use std::time::Duration;

use bbr_ui::surfaces::open_app::{OpenAppEvent, OpenAppModel};
use bbr_web::{ProfileId, WebViewId};
use gpui::{Context, Window};

use super::BbrApp;
use crate::site_permissions;

/// How long the no-app banner stays.
const NO_APP_BANNER_DURATION: Duration = Duration::from_secs(8);

/// The link waiting for an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppRequest {
    pub view: WebViewId,
    pub url: String,
    pub scheme: String,
    pub app: String,
    pub profile: ProfileId,
    /// The asking page's origin (`None` for a page without one).
    pub origin: Option<String>,
}

/// The app's state for links to other apps.
#[derive(Default)]
pub(crate) struct OpenAppState {
    pub request: Option<AppRequest>,
    pub remember: bool,
    /// The no-app banner's message and generation.
    pub banner: Option<(String, u64)>,
    pub banner_bounds: std::rc::Rc<std::cell::Cell<Option<[f32; 4]>>>,
    banner_generation: u64,
    banner_task: Option<gpui::Task<()>>,
    /// Links handed to macOS (testkit evidence; the testkit build records
    /// them instead of launching another app).
    #[cfg(feature = "testkit")]
    pub opened: Vec<String>,
}

/// The scheme of a URL, lower-cased.
pub(crate) fn scheme_of(url: &str) -> Option<String> {
    let (scheme, _) = url.split_once(':')?;
    let valid = !scheme.is_empty()
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    valid.then(|| scheme.to_ascii_lowercase())
}

impl BbrApp {
    /// The engine reported a link to another app's scheme.
    pub(crate) fn on_external_scheme(
        &mut self,
        view: WebViewId,
        url: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_app.request.is_some() {
            return;
        }
        let Some(scheme) = scheme_of(url) else {
            return;
        };
        let Some(profile) = self.controller.read().profile_of_view(view) else {
            return;
        };
        let Some(app) = self.engine.borrow().external_app_name(url) else {
            self.show_no_app_banner(&scheme, cx);
            return;
        };
        let origin = self
            .controller
            .read()
            .live_view_urls()
            .into_iter()
            .find(|(candidate, _, _)| *candidate == view)
            .and_then(|(_, _, page)| site_permissions::origin_of(&page));
        let allowed = origin.as_ref().is_some_and(|origin| {
            self.permissions
                .policies
                .read()
                .app_allowed(profile, origin, &scheme)
        });
        if allowed {
            self.hand_to_app(url);
            return;
        }
        self.open_app.request = Some(AppRequest {
            view,
            url: url.to_string(),
            scheme,
            app,
            profile,
            origin,
        });
        self.open_app.remember = false;
        self.engine.borrow_mut().focus_host_chrome();
        window.focus(&self.chrome_handle, cx);
        self.sync_portal_overlays(cx);
    }

    fn hand_to_app(&mut self, url: &str) {
        #[cfg(feature = "testkit")]
        {
            self.open_app.opened.push(url.to_string());
        }
        #[cfg(not(feature = "testkit"))]
        {
            if !self.engine.borrow_mut().open_external(url) {
                log::warn!("macOS refused to open a link in another app");
            }
        }
    }

    fn show_no_app_banner(&mut self, scheme: &str, cx: &mut Context<Self>) {
        self.open_app.banner_generation += 1;
        let generation = self.open_app.banner_generation;
        self.open_app.banner = Some((
            format!("No app on this Mac opens “{scheme}:” links."),
            generation,
        ));
        self.open_app.banner_bounds.set(None);
        self.open_app.banner_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(NO_APP_BANNER_DURATION).await;
            let _ = this.update(cx, |app, cx| {
                if app
                    .open_app
                    .banner
                    .as_ref()
                    .is_some_and(|(_, current)| *current == generation)
                {
                    app.open_app.banner = None;
                    app.sync_portal_overlays(cx);
                }
            });
        }));
        self.sync_portal_overlays(cx);
    }

    /// The sheet shows while its page is the active one.
    pub(crate) fn open_app_sheet_open(&self) -> bool {
        self.open_app.request.as_ref().is_some_and(|request| {
            self.controller.read().snapshot().active_view == Some(request.view)
        })
    }

    /// The sheet's model.
    pub(crate) fn open_app_model(&self) -> Option<OpenAppModel> {
        if !self.open_app_sheet_open() {
            return None;
        }
        let request = self.open_app.request.as_ref()?;
        let host = request.origin.as_deref().map(site_permissions::origin_host);
        Some(OpenAppModel {
            title: format!("Open in {}?", request.app).into(),
            message: match host {
                Some(host) => format!("{host} wants to open the {} app.", request.app),
                None => format!("A page wants to open the {} app.", request.app),
            }
            .into(),
            remember_label: host
                .map(|host| format!("Always allow {host} to open {} links", request.app).into()),
            remember: self.open_app.remember,
            confirm: format!("Open {}", request.app).into(),
        })
    }

    /// Every sheet control (pointer, keys and the testkit).
    pub(crate) fn on_open_app_event(
        &mut self,
        event: OpenAppEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.open_app_sheet_open() {
            return;
        }
        match event {
            OpenAppEvent::Remember => {
                self.open_app.remember = !self.open_app.remember;
                self.sync_portal_overlays(cx);
                return;
            }
            OpenAppEvent::Cancel => {
                self.open_app.request = None;
            }
            OpenAppEvent::Open => {
                let Some(request) = self.open_app.request.take() else {
                    return;
                };
                if self.open_app.remember {
                    if let Some(origin) = request.origin.as_ref() {
                        self.permissions.policies.get().allow_app(
                            request.profile,
                            origin,
                            &request.scheme,
                        );
                        self.save_site_permissions();
                    }
                }
                self.hand_to_app(&request.url);
            }
        }
        self.open_app.remember = false;
        self.sync_portal_overlays(cx);
        self.focus_web(window, cx);
    }

    /// The sheet's keyboard: Return opens (Escape is the global binding).
    pub(crate) fn on_open_app_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "enter" {
            cx.stop_propagation();
            self.on_open_app_event(OpenAppEvent::Open, window, cx);
        }
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    pub(crate) fn open_app_snapshot(&self) -> bbr_testkit::protocol::OpenAppSnapshot {
        bbr_testkit::protocol::OpenAppSnapshot {
            sheet: self
                .open_app_model()
                .map(|model| bbr_testkit::protocol::OpenAppSheetSnapshot {
                    title: model.title.to_string(),
                    message: model.message.to_string(),
                    remember_label: model.remember_label.map(|label| label.to_string()),
                    remember: model.remember,
                    confirm: model.confirm.to_string(),
                }),
            no_app_banner: self
                .open_app
                .banner
                .as_ref()
                .map(|(message, _)| message.clone()),
            opened: self.open_app.opened.clone(),
            space_chooser: self.space_chooser_model().map(|model| {
                (
                    model.title.to_string(),
                    model.rows.iter().map(|row| row.to_string()).collect(),
                    model.selected,
                )
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scheme_of;

    #[test]
    fn schemes_are_read_and_lower_cased() {
        assert_eq!(scheme_of("MailTo:a@example.com").as_deref(), Some("mailto"));
        assert_eq!(
            scheme_of("zoommtg://zoom.us/join?x=1").as_deref(),
            Some("zoommtg")
        );
        assert_eq!(scheme_of("not a url"), None);
        assert_eq!(scheme_of("1abc:x"), None);
    }
}
