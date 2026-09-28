//! Space-configuration surface behavior (DESIGN.md §11.3, packet
//! `space-creation-appearance-ui.md`): the app-owned rename/identity
//! surface presented after Space creation and from the active Space's
//! management affordance.
//!
//! The surface is presentation-only (the render lives in
//! [`bbr_ui::surfaces::space_config`]); this module owns the app side: the
//! open state bound to the **exact persistent `SpaceId`**, the draft model
//! built from the one controller authority, the rename/theme/identity
//! commit through the one controller authorities, the keyboard seam
//! (printable chars edit the draft, Backspace deletes, Enter commits,
//! Escape cancels), the §13 outside-press dismissal, and the
//! create→activate→configure lifecycle both creation paths share.
//!
//! Exact-SpaceId / stale-source contract: the bound `SpaceId` is captured
//! at open and never re-derived; every commit targets that exact id
//! through the controller (which rejects unknown spaces whole), so a stale
//! surface can never mutate another Space after a switch. The draft is the
//! model built at open — Done applies the draft, never a re-read. Cancel /
//! Escape keeps the created Space and its valid defaults; creation is never
//! rolled back.

use bbr_browser::SpaceId;
use bbr_domain::{SpaceIdentity, SpaceTheme, TintIntensity};
use bbr_ui::surfaces::space_config::{
    SpaceConfigCallbacks, SpaceConfigCancel, SpaceConfigDone, SpaceConfigInsidePress,
    SpaceConfigModel, SpaceConfigSelectIdentity, SpaceConfigSelectIntensity,
    SpaceConfigSelectTheme, SpaceConfigToggleGrain,
};
use gpui::{Context, FocusHandle, KeyDownEvent, Stateful, Window};
use std::rc::Rc;

use super::BbrApp;

/// App-local Space-configuration state: the open flag, the bound persistent
/// [`SpaceId`], the presentation draft, and the surface's chrome focus
/// handle. Grouped here so the browser root stays bounded while every
/// rename/appearance authority stays in the controller.
pub(super) struct SpaceConfigState {
    pub open: bool,
    /// The exact persistent SpaceId the surface is bound to (captured at
    /// open; never re-derived, never the render position).
    pub space: Option<SpaceId>,
    /// The presentation draft built at open (title, rename draft, selected
    /// curated theme/identity).
    pub model: SpaceConfigModel,
    /// The surface's chrome focus handle (the §9 keyboard domain of the
    /// rename field while open).
    pub handle: FocusHandle,
    /// §13 one-pointer-action marker: a press inside the surface marks
    /// itself so the root outside-press handler never dismisses it.
    pub press_inside: bool,
}

impl SpaceConfigState {
    pub fn new(cx: &mut Context<BbrApp>) -> Self {
        Self {
            open: false,
            space: None,
            model: SpaceConfigModel::new(""),
            handle: cx.focus_handle(),
            press_inside: false,
        }
    }
}

impl BbrApp {
    /// Open the configuration surface for the **exact** persistent
    /// `space`, building the draft from the one controller authority (the
    /// space's current title and its effective appearance — explicit record
    /// or derived profile-hue default). At most one sidebar popover is open
    /// at a time: opening the surface closes the plus menu and the
    /// downloads panel. Returns `false` (without opening) when the space is
    /// unknown — a stale entry point never opens a surface for a ghost id.
    pub fn open_space_config(
        &mut self,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(title) = self.controller.space_title(space) else {
            log::error!("could not open space config for unknown space {space:?}");
            return false;
        };
        let appearance = self
            .controller
            .space_appearance(space)
            .expect("space exists");
        if self.u2.plus_menu.open {
            self.close_plus_menu(window, cx);
        }
        crate::app::downloads_behavior::downloads_close(self, window, cx);
        self.command_bar_mode = crate::command_bar::CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.command_ranking.close();
        self.space_config.open = true;
        self.space_config.space = Some(space);
        self.space_config.model = SpaceConfigModel {
            title,
            name: self
                .controller
                .space_title(space)
                .expect("space exists")
                .clone(),
            theme: appearance.theme,
            identity: appearance.identity,
            paper: appearance.paper,
        };
        self.space_config.press_inside = false;
        window.focus(&self.space_config.handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
        true
    }

    /// Close the configuration surface without applying the draft: the
    /// created Space and its valid defaults stay. Chrome focus returns to
    /// the permanent space strip.
    pub fn close_space_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.space_config.open {
            return;
        }
        self.space_config.open = false;
        self.space_config.space = None;
        self.space_config.press_inside = false;
        self.focus_space_strip(window, cx);
        cx.notify();
    }

    /// §11.3 draft selection: pick one curated theme token. The draft
    /// updates only; nothing reaches the controller until Done.
    pub fn space_config_select_theme(&mut self, token: SpaceTheme, cx: &mut Context<Self>) {
        if !self.space_config.open {
            return;
        }
        self.space_config.model.theme = token;
        cx.notify();
    }

    /// v2 D7 draft selection: pick one curated tint intensity step. The
    /// draft updates only; nothing reaches the controller until Done.
    pub fn space_config_select_intensity(&mut self, step: TintIntensity, cx: &mut Context<Self>) {
        if !self.space_config.open {
            return;
        }
        self.space_config.model.paper.intensity = step;
        cx.notify();
    }

    /// v2 D6 draft toggle: turn the static grain on or off. The draft
    /// updates only; nothing reaches the controller until Done.
    pub fn space_config_toggle_grain(&mut self, cx: &mut Context<Self>) {
        if !self.space_config.open {
            return;
        }
        self.space_config.model.paper.grain = !self.space_config.model.paper.grain;
        cx.notify();
    }

    /// §11.3 draft selection: pick exactly one curated emoji or glyph
    /// identity. The draft updates only; nothing reaches the controller
    /// until Done.
    pub fn space_config_select_identity(
        &mut self,
        identity: SpaceIdentity,
        cx: &mut Context<Self>,
    ) {
        if !self.space_config.open {
            return;
        }
        self.space_config.model.identity = identity;
        cx.notify();
    }

    /// Commit the draft for the **exact bound SpaceId**: apply the rename
    /// (a non-empty draft that changed the title), the selected curated
    /// theme, and the selected identity through the one controller
    /// authorities, then persist the committed model. Unknown or vanished
    /// spaces reject whole (the controller authority) and only log — a
    /// stale surface never mutates another Space. Done never creates a
    /// second Space.
    pub fn space_config_done(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.space_config.open {
            return;
        }
        let Some(space) = self.space_config.space else {
            self.close_space_config(window, cx);
            return;
        };
        let draft_name = self.space_config.model.name.trim().to_string();
        let current = self.controller.space_title(space).unwrap_or_default();
        if !draft_name.is_empty() && draft_name != current {
            if let Err(error) = self.controller.rename_space(space, draft_name.clone()) {
                log::error!("could not rename space {space:?}: {error}");
            }
        }
        if let Err(error) = self
            .controller
            .set_space_theme(space, self.space_config.model.theme)
        {
            log::error!("could not set space theme {space:?}: {error}");
        }
        if let Err(error) = self
            .controller
            .set_space_identity(space, self.space_config.model.identity.clone())
        {
            log::error!("could not set space identity {space:?}: {error}");
        }
        if let Err(error) = self
            .controller
            .set_space_paper(space, self.space_config.model.paper)
        {
            log::error!("could not set space paper {space:?}: {error}");
        }
        // The rename + appearance are structural browser mutations: poke the
        // one browser-state writer so a force quit recovers the committed
        // model with the new title/theme/identity/paper (schema v4).
        self.persist_structure();
        self.close_space_config(window, cx);
        cx.notify();
    }

    /// Cancel / Escape: close without applying. The created Space and its
    /// valid defaults stay usable; creation is never rolled back.
    pub fn space_config_cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.space_config.open {
            return;
        }
        self.close_space_config(window, cx);
    }

    /// §13 one-pointer-action marker: a press inside the surface marks
    /// itself so the root outside-press handler never dismisses it.
    pub fn mark_space_config_press_inside(&mut self) {
        self.space_config.press_inside = true;
    }

    /// §13: a pointer press outside the open surface dismisses it (one
    /// pointer action). The surface's own inside-press handler runs first
    /// in the bubble phase and stops propagation, so a press inside never
    /// reaches this root handler; any other press — a footer control, a
    /// tab row, the divider, the web content — closes the surface without
    /// applying the draft.
    pub fn dismiss_space_config_on_outside_press(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inside = self.space_config.press_inside;
        self.space_config.press_inside = false;
        if self.space_config.open && !inside {
            self.close_space_config(window, cx);
        }
    }

    /// The §11.3 keyboard seam (the surface owns chrome focus while open):
    /// printable characters edit the rename draft, Backspace deletes the
    /// last character, Enter commits (Done), Escape cancels. Every decision
    /// routes through the same app methods the pointer controls drive.
    pub fn on_space_config_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.space_config.open {
            return;
        }
        let key = event.keystroke.key.as_str();
        match key {
            "backspace" => {
                self.space_config.model.name.pop();
                cx.notify();
                cx.stop_propagation();
            }
            "enter" => {
                cx.stop_propagation();
                self.space_config_done(window, cx);
            }
            "escape" => {
                cx.stop_propagation();
                self.space_config_cancel(window, cx);
            }
            // A single unmodified printable character (letters, digits,
            // space, the §4 punctuation set) edits the draft. Modified
            // keystrokes (e.g. the global `cmd-shift-n` New Space binding)
            // have no `key_char` and pass through to the app's action
            // system, exactly like the find bar's query editing.
            _ => {
                if let Some(ch) = event.keystroke.key_char.as_deref() {
                    if let Some(ch) = ch.chars().next() {
                        if is_editable_char(ch) {
                            self.space_config.model.name.push(ch);
                            cx.notify();
                            cx.stop_propagation();
                        }
                    }
                }
            }
        }
    }

    /// The §11.3 configuration surface element (in-rail panel above the
    /// footer row, the §11.12 panel family), or `None` while closed. The
    /// production listener bundle re-enters the app's draft/commit
    /// authorities on real pointer/keyboard events; the surface renders
    /// only.
    pub(crate) fn space_config_surface(
        &mut self,
        theme: &bbr_ui::theme::Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<gpui::Div>> {
        // Render milestone for the §11.3 presentation probe: record the
        // surface presentation this frame actually drew (open/closed plus
        // the bound exact persistent SpaceId and the drawn curated
        // theme/identity), so a pending Space-create/config benchmark can
        // commit only from a render that presented the armed identity. The
        // observation runs on every frame — closed frames record `false` so
        // a post-commit sample can commit from a frame that omitted the
        // surface.
        #[cfg(feature = "testkit")]
        {
            let bound = self
                .space_config
                .space
                .map(|space| bbr_testkit::protocol::TestSpaceId(space.0));
            self.performance.observe_space_config(
                self.space_config.open,
                bound,
                self.space_config.model.theme.name().to_string(),
                self.space_config.model.identity.name().to_string(),
            );
        }
        if !self.space_config.open {
            return None;
        }
        let focused = self.space_config.handle.is_focused(window);
        let entity = cx.entity();
        let on_select_theme: SpaceConfigSelectTheme = Rc::new(move |token, _window, cx| {
            entity.update(cx, |view, cx| view.space_config_select_theme(token, cx));
        });
        let entity = cx.entity();
        let on_select_identity: SpaceConfigSelectIdentity =
            Rc::new(move |identity, _window, cx| {
                entity.update(cx, |view, cx| {
                    view.space_config_select_identity(identity, cx);
                });
            });
        let entity = cx.entity();
        let on_select_intensity: SpaceConfigSelectIntensity = Rc::new(move |step, _window, cx| {
            entity.update(cx, |view, cx| view.space_config_select_intensity(step, cx));
        });
        let entity = cx.entity();
        let on_toggle_grain: SpaceConfigToggleGrain = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| view.space_config_toggle_grain(cx));
        });
        let entity = cx.entity();
        let on_inside_press: SpaceConfigInsidePress = Rc::new(move |_e, _window, cx| {
            entity.update(cx, |view, _cx| view.mark_space_config_press_inside());
            cx.stop_propagation();
        });
        let entity = cx.entity();
        let on_done: SpaceConfigDone = Rc::new(move |window, cx| {
            entity.update(cx, |view, cx| view.space_config_done(window, cx));
        });
        let entity = cx.entity();
        let on_cancel: SpaceConfigCancel = Rc::new(move |window, cx| {
            entity.update(cx, |view, cx| view.space_config_cancel(window, cx));
        });
        let callbacks = SpaceConfigCallbacks {
            on_select_theme,
            on_select_identity,
            on_select_intensity,
            on_toggle_grain,
            on_inside_press,
            on_done,
            on_cancel,
        };
        Some(bbr_ui::surfaces::space_config::space_config_surface(
            theme,
            &self.space_config.model,
            focused,
            &self.space_config.handle,
            &callbacks,
        ))
    }
    /// §11.3 Space-configuration surface evidence for the E2E snapshot:
    /// the bound exact persistent SpaceId, the header title, the rename
    /// draft, the selected curated theme/identity names, and whether the
    /// surface owns chrome focus. `None` while closed (the surface is not
    /// rendered). Never mutates the surface.
    #[cfg(feature = "testkit")]
    pub(crate) fn space_config_snapshot(
        &self,
        window: &Window,
    ) -> Option<bbr_testkit::protocol::SpaceConfigSnapshot> {
        if !self.space_config.open {
            return None;
        }
        let model = &self.space_config.model;
        Some(bbr_testkit::protocol::SpaceConfigSnapshot {
            space: bbr_testkit::protocol::TestSpaceId(
                self.space_config.space.map(|s| s.0).unwrap_or_default(),
            ),
            title: model.title.clone(),
            name: model.name.clone(),
            theme: model.theme.name().to_string(),
            identity: model.identity.name().to_string(),
            intensity: model.paper.intensity.name().to_string(),
            grain: model.paper.grain,
            focused: self.space_config.handle.is_focused(window),
        })
    }
}

/// The rename-draft character set: printable §4 chrome text (letters,
/// digits, space, and the punctuation the §4 truncation tolerates). Emoji
/// and control characters are not editable in the draft.
fn is_editable_char(ch: char) -> bool {
    ch.is_alphanumeric()
        || matches!(
            ch,
            ' ' | '-'
                | '_'
                | '.'
                | ','
                | ':'
                | ';'
                | '/'
                | '\\'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '!'
                | '?'
                | '&'
                | '+'
                | '='
                | '#'
                | '@'
                | '\''
        )
}

/// The testkit-only benchmark drivers (create→activate→configure and
/// identity-commit measurement seams) live in the sibling `benchmarks`
/// module, split by concern so each file stays under the ~500-line budget.
#[cfg(feature = "testkit")]
mod benchmarks;

#[cfg(test)]
mod tests;
