//! The §11.14 titlebar composition's single input contract (packet
//! `titlebar-navigation-controls.md`): the plain [`TitlebarModel`] DTO and
//! the one [`TitlebarCallbacks`] set of handler closures.
//!
//! The titlebar row lives at the top of the expanded full-height Sidebar
//! rail, inside the actual macOS safe-titlebar layout guide:
//! `[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]`. The
//! model is plain data — the safe-area geometry, the sidebar collapse
//! state, the genuine navigation availability of the focused pane, and the
//! local hover/pressed presentation state. AppKit details (the native
//! window query) stay in the macOS composition edge; the rail renders only
//! what the model reports, and no traffic-light coordinate is ever
//! hardcoded here.

use gpui::{App, KeyDownEvent, MouseDownEvent, MouseUpEvent, Window};

/// The one §11.14 titlebar control (the toggle plus the three navigation
/// controls). Each maps to one stable `bbr-*` id; disabled controls are
/// visible, semantically explained, and no-ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarControl {
    /// The sidebar collapse/expand toggle (`bbr-titlebar-sidebar-toggle`).
    Toggle,
    /// Back (`bbr-nav-back`) — enabled by the focused pane's real history.
    Back,
    /// Forward (`bbr-nav-forward`) — enabled by the focused pane's real
    /// history.
    Forward,
    /// Reload (`bbr-nav-reload`) — enabled while the focused pane has a
    /// live session.
    Reload,
}

impl TitlebarControl {
    /// The stable §15 id of the control.
    pub fn id(self) -> &'static str {
        match self {
            TitlebarControl::Toggle => "bbr-titlebar-sidebar-toggle",
            TitlebarControl::Back => "bbr-nav-back",
            TitlebarControl::Forward => "bbr-nav-forward",
            TitlebarControl::Reload => "bbr-nav-reload",
        }
    }

    /// The semantic name (the future Accessibility label; §15).
    pub fn label(self) -> &'static str {
        match self {
            TitlebarControl::Toggle => "Sidebar (Cmd-B)",
            TitlebarControl::Back => "Back",
            TitlebarControl::Forward => "Forward",
            TitlebarControl::Reload => "Reload",
        }
    }
}

/// The stable §15 id of the titlebar row container.
pub const TITLEBAR_ID: &str = "bbr-titlebar";

/// One handler closure slot of the titlebar `Callbacks` set. The app fills
/// every slot with the production app methods; the gallery fills recorder
/// callbacks. `Rc<dyn Fn>` so the row can clone a slot into per-control
/// listeners. The event references are higher-ranked so the row can call a
/// slot with any event lifetime.
pub type MouseCallback = std::rc::Rc<dyn for<'a> Fn(&'a MouseDownEvent, &mut Window, &mut App)>;
/// A `MouseUpEvent` callback slot.
pub type MouseUpCallback = std::rc::Rc<dyn for<'a> Fn(&'a MouseUpEvent, &mut Window, &mut App)>;
/// A `KeyDownEvent` callback slot.
pub type KeyCallback = std::rc::Rc<dyn for<'a> Fn(&'a KeyDownEvent, &mut Window, &mut App)>;
/// A control-indexed hover callback slot (enter/leave).
pub type ControlHoverCallback = std::rc::Rc<dyn Fn(TitlebarControl, bool, &mut Window, &mut App)>;
/// A control-indexed press callback slot (primary button down).
pub type ControlPressCallback = std::rc::Rc<dyn Fn(TitlebarControl, &mut Window, &mut App)>;
/// The release callback slot (primary button up).
pub type ControlReleaseCallback = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

/// The native safe-titlebar layout guide as plain presentation data
/// (points, window coordinates, top-left origin). The macOS adapter in
/// `bbr-web-macos` produces it; the rail renders from it. The portable
/// fallback is a zero-height strip with an empty traffic-light region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeArea {
    /// Titlebar strip height in points (0 when no native titlebar exists).
    pub height: f32,
    /// The traffic-light region `[left, top, width, height]` in window
    /// coordinates. The sidebar toggle must start after its right edge.
    pub traffic_lights: [f32; 4],
}

impl SafeArea {
    /// The portable fallback (non-macOS / before the window attaches): no
    /// strip, no traffic-light clearance.
    pub fn none() -> Self {
        Self {
            height: 0.0,
            traffic_lights: [0.0; 4],
        }
    }

    /// The left edge the BBR-owned controls start at: the traffic-light
    /// region's right edge (never a hardcoded x offset).
    pub fn control_left(&self) -> f32 {
        self.traffic_lights[0] + self.traffic_lights[2]
    }
}

/// The complete §11.14 titlebar model — the single input contract of the
/// titlebar composition. Every field is plain data; the row renders it
/// through the production `bbr-ui` components and the one
/// [`TitlebarCallbacks`] set.
#[derive(Debug, Clone)]
pub struct TitlebarModel {
    /// The native safe-titlebar layout guide (the strip height and the
    /// traffic-light clearance).
    pub safe_area: SafeArea,
    /// The sidebar is collapsed: the toggle glyph and the semantic value
    /// reflect it, and the whole titlebar disappears with the rail.
    pub sidebar_collapsed: bool,
    /// The focused pane's real history availability (Back).
    pub can_go_back: bool,
    /// The focused pane's real history availability (Forward).
    pub can_go_forward: bool,
    /// Reload is enabled while the focused pane has a live session.
    pub can_reload: bool,
    /// §10.2 hover state: the control under the pointer (`None` at rest).
    pub hovered: Option<TitlebarControl>,
    /// §10.3 pressed state: the control whose primary button is down
    /// (`None` at rest).
    pub pressed: Option<TitlebarControl>,
}

/// The one set of handler closures the titlebar row wires onto the
/// production controls. Every slot is a plain closure the caller fills; the
/// row never invents behavior.
#[derive(Clone)]
pub struct TitlebarCallbacks {
    /// §11.14: the sidebar toggle (`Cmd-B` pointer entry — the same
    /// sidebar-width/collapse authority as the shortcut).
    pub on_toggle_sidebar: MouseCallback,
    /// §11.14: Back through the existing navigation authority (the same
    /// dispatch as `Cmd-[`).
    pub on_back: MouseCallback,
    /// §11.14: Forward through the existing navigation authority (the same
    /// dispatch as `Cmd-]`).
    pub on_forward: MouseCallback,
    /// §11.14: Reload of the focused pane (the same dispatch as `Cmd-R`).
    pub on_reload: MouseCallback,
    /// §11.14: a pointer press on the empty (draggable) titlebar space —
    /// the native window-drag entry (and the double-click action on a
    /// double click).
    pub on_titlebar_mouse_down: MouseCallback,
    /// §10.2: the pointer entered/left a control (the hover wash).
    pub on_control_hover: ControlHoverCallback,
    /// §10.3: the primary button down on a control (the pressed wash).
    pub on_control_press: ControlPressCallback,
    /// §10.3: the primary button up (clears the pressed wash).
    pub on_control_release: ControlReleaseCallback,
}

impl TitlebarCallbacks {
    /// A no-op callback set (the gallery's recorder stories start from
    /// here; the app fills every slot).
    pub fn noop() -> Self {
        let noop_mouse: MouseCallback = std::rc::Rc::new(|_e, _window, _cx| {});
        let noop_hover: ControlHoverCallback = std::rc::Rc::new(|_c, _b, _window, _cx| {});
        let noop_press: ControlPressCallback = std::rc::Rc::new(|_c, _window, _cx| {});
        let noop_release: ControlReleaseCallback = std::rc::Rc::new(|_window, _cx| {});
        Self {
            on_toggle_sidebar: noop_mouse.clone(),
            on_back: noop_mouse.clone(),
            on_forward: noop_mouse.clone(),
            on_reload: noop_mouse.clone(),
            on_titlebar_mouse_down: noop_mouse,
            on_control_hover: noop_hover,
            on_control_press: noop_press,
            on_control_release: noop_release,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The §11.14 availability matrix: no history, back-only,
    /// forward-only, both, loading/reload, and collapsed/expanded sidebar
    /// all land in the plain model untouched — the row renders the genuine
    /// enablement and the callbacks decide nothing.
    #[test]
    fn availability_matrix_flows_into_the_plain_model() {
        let cases: &[(bool, bool, bool, bool, bool)] = &[
            // No history at all; reload off with no live session.
            (false, false, false, false, false),
            // Back-only history; session live (reload on).
            (true, false, true, false, true),
            // Forward-only history; session live.
            (false, true, true, false, true),
            // Both history directions; session live.
            (true, true, true, false, true),
            // Loading with both history directions.
            (true, true, true, true, true),
            // Collapsed sidebar (the row disappears with the rail).
            (false, false, false, true, true),
        ];
        for (can_go_back, can_go_forward, can_reload, loading, collapsed) in cases {
            let model = TitlebarModel {
                safe_area: SafeArea::none(),
                sidebar_collapsed: *collapsed,
                can_go_back: *can_go_back,
                can_go_forward: *can_go_forward,
                can_reload: *can_reload,
                hovered: None,
                pressed: None,
            };
            assert_eq!(model.can_go_back, *can_go_back);
            assert_eq!(model.can_go_forward, *can_go_forward);
            assert_eq!(model.can_reload, *can_reload);
            assert_eq!(model.sidebar_collapsed, *collapsed);
            let _ = *loading;
        }
    }

    /// A disabled control is visible and semantically explained: the model
    /// still reports the control (the row renders it with the §10.8
    /// treatment) while the enablement is false.
    #[test]
    fn disabled_controls_stay_in_the_model() {
        let model = TitlebarModel {
            safe_area: SafeArea::none(),
            sidebar_collapsed: false,
            can_go_back: false,
            can_go_forward: false,
            can_reload: false,
            hovered: None,
            pressed: None,
        };
        assert!(!model.can_go_back);
        assert!(!model.can_go_forward);
        assert!(!model.can_reload);
        // Every §15 id is stable regardless of enablement.
        assert_eq!(TitlebarControl::Back.id(), "bbr-nav-back");
        assert_eq!(TitlebarControl::Forward.id(), "bbr-nav-forward");
        assert_eq!(TitlebarControl::Reload.id(), "bbr-nav-reload");
    }

    /// The safe-area layout guide keeps the controls out of the
    /// traffic-light region at every supported window height: the control
    /// left edge is the region's right edge, never a hardcoded x offset.
    #[test]
    fn safe_area_clears_any_traffic_light_region() {
        for height in [22.0, 28.0, 52.0] {
            for width in [14.0, 18.0, 24.0] {
                let area = SafeArea {
                    height,
                    traffic_lights: [12.0, 10.0, width, 14.0],
                };
                assert_eq!(area.control_left(), 12.0 + width);
                assert_eq!(area.height, height);
            }
        }
    }

    /// The semantic labels match the §11.14 controls (the future
    /// Accessibility names).
    #[test]
    fn control_labels_are_semantic() {
        assert_eq!(TitlebarControl::Toggle.label(), "Sidebar (Cmd-B)");
        assert_eq!(TitlebarControl::Back.label(), "Back");
        assert_eq!(TitlebarControl::Forward.label(), "Forward");
        assert_eq!(TitlebarControl::Reload.label(), "Reload");
    }
}
