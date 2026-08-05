//! The `BbrApp` root view: GPUI chrome (sidebar + command bar + status) with
//! the native web-view slot; also owns focus choreography and navigation
//! command routing.

use bbr_web::{WebAction, WebEvent};
use gpui::{
    div, prelude::*, px, rgb, App, Context, FocusHandle, Focusable, KeyDownEvent, MouseButton,
    MouseDownEvent, Render, SharedString, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::engine::EngineRef;
use crate::shell::ShellState;
#[cfg(feature = "testkit")]
use crate::shell::TestSnapshot;
use crate::slot::NativeWebViewSlot;

/// Fixed sidebar width in points (Phase-4 will make this adjustable).
pub const SIDEBAR_WIDTH_PX: f32 = 260.0;

/// Focus-change sink: invoked on the main thread after any focus mutation
/// with the actual chrome-focus state. Used by the testkit gateway glue.
pub type FocusSink = Box<dyn FnMut(bool)>;

/// Default window size for the spike.
pub const DEFAULT_WINDOW_SIZE: (f32, f32) = (1080.0, 700.0);

/// The root view of the browser window.
pub struct BbrApp {
    engine: EngineRef,
    shell: ShellState,
    /// GPUI focus handle for the chrome (sidebar/command bar).
    chrome_handle: FocusHandle,
    /// Whether the command bar is the active input surface.
    pub command_bar_active: bool,
    /// Current command-bar text (mirrors the live URL when not editing).
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    /// Focus-change sink (main thread): called after any focus mutation with
    /// the actual chrome-focus state. Wired by the testkit glue to broadcast
    /// `ChromeFocusChanged`; unused in production builds.
    focus_sink: Rc<RefCell<Option<FocusSink>>>,
}

impl BbrApp {
    pub fn new(
        engine: EngineRef,
        shell: ShellState,
        evt_tx: smol::channel::Sender<WebEvent>,
        cx: &mut Context<Self>,
    ) -> Self {
        // Engine → app events: the sink runs on the main thread (delegate
        // callbacks); the channel is drained by the pump task that mirrors
        // into `shell` and (under testkit) broadcasts to E2E clients.
        let sink_tx = evt_tx.clone();
        engine
            .borrow_mut()
            .set_events(Box::new(move |ev: WebEvent| {
                let _ = sink_tx.try_send(ev);
            }));

        BbrApp {
            engine: engine.clone(),
            shell,
            chrome_handle: cx.focus_handle(),
            command_bar_active: false,
            command_bar_text: String::new(),
            sidebar_visible: true,
            focus_sink: Rc::new(RefCell::new(None)),
        }
    }

    /// Start the actual web view; the initial URL is loaded by the caller
    /// AFTER the window's first frame so `attach_slot` can frame it.
    pub fn create_web_view(&mut self, initial_url: &str) {
        self.engine.borrow_mut().create_web_view(
            self.shell.active_view,
            self.shell.profile,
            initial_url,
        );
        self.shell.web_view_hosted = true;
    }

    /// Route a generic [`WebAction`] to the engine (E2E gateway + unit tests).
    #[cfg(feature = "testkit")]
    pub fn web_action(&mut self, action: WebAction) {
        match &action {
            WebAction::Load { url } => {
                // Load goes through the same path as the command bar so the
                // command-bar text + focus stay consistent.
                let url = url.clone();
                self.engine
                    .borrow_mut()
                    .action(self.shell.active_view, WebAction::Load { url });
            }
            _ => self
                .engine
                .borrow_mut()
                .action(self.shell.active_view, action),
        }
    }

    /// Navigate the active view (used by Enter in the command bar, and by
    /// the E2E gateway).
    pub fn navigate(&mut self, input: &str, window: &mut Window, cx: &mut Context<Self>) {
        let url = bbr_web::normalize_url(input);
        if url.is_empty() {
            return;
        }
        self.engine
            .borrow_mut()
            .action(self.shell.active_view, WebAction::Load { url: url.clone() });
        self.command_bar_text = url;
        self.focus_web(window, cx);
    }

    pub fn go_back(&mut self) {
        self.engine
            .borrow_mut()
            .action(self.shell.active_view, WebAction::Back);
    }

    pub fn go_forward(&mut self) {
        self.engine
            .borrow_mut()
            .action(self.shell.active_view, WebAction::Forward);
    }

    pub fn reload(&mut self) {
        self.engine
            .borrow_mut()
            .action(self.shell.active_view, WebAction::Reload);
    }

    /// Mirror one engine event into the shell and the command-bar display.
    pub fn on_web_event(&mut self, event: WebEvent, cx: &mut Context<Self>) {
        self.shell.nav.apply(&event);
        if let WebEvent::UrlChanged(_, url) = &event {
            if !self.command_bar_active {
                self.command_bar_text = url.clone();
            }
        }
        cx.notify();
    }

    /// Move GPUI focus + AppKit first responder to the command bar.
    pub fn activate_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_active = true;
        window.focus(&self.chrome_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Move GPUI focus + AppKit first responder back to the web view.
    pub fn focus_web(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_active = false;
        window.blur();
        self.engine.borrow_mut().focus_native_web();
        cx.notify();
        self.emit_focus(window);
    }

    /// GPUI chrome gained focus without activating the command bar (click).
    fn focus_chrome(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_active = false;
        window.focus(&self.chrome_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Report the actual chrome-focus state through the focus sink (main
    /// thread). No-op when no sink is installed.
    fn emit_focus(&mut self, window: &Window) {
        if let Some(sink) = self.focus_sink.borrow_mut().as_mut() {
            let focused = self.chrome_handle.is_focused(window);
            sink(focused);
        }
    }

    /// Install a focus-change sink (used by the E2E gateway).
    #[cfg(feature = "testkit")]
    pub fn set_focus_sink(&mut self, sink: FocusSink) {
        *self.focus_sink.borrow_mut() = Some(sink);
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        cx.notify();
    }

    /// Handle printable typing + backspace/enter while the command bar is
    /// the active input surface.
    fn on_chrome_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_bar_active {
            return;
        }
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "backspace" => {
                self.command_bar_text.pop();
                cx.stop_propagation();
            }
            "enter" => {
                let input = self.command_bar_text.clone();
                cx.stop_propagation();
                self.navigate(&input, window, cx);
            }
            _ => {
                if let Some(ch) = ks.key_char.as_deref() {
                    if !ch.is_empty() {
                        self.command_bar_text.push_str(ch);
                        cx.stop_propagation();
                    }
                }
            }
        }
        cx.notify();
    }

    /// A snapshot for the E2E protocol (also used by unit tests).
    #[cfg(feature = "testkit")]
    pub fn snapshot(&self, window: &Window) -> TestSnapshot {
        TestSnapshot {
            url: self.shell.nav.url.clone(),
            title: self.shell.nav.title.clone(),
            loading: self.shell.nav.loading,
            can_go_back: self.shell.nav.can_go_back,
            can_go_forward: self.shell.nav.can_go_forward,
            web_view_hosted: self.shell.web_view_hosted,
            engine_kind: self.shell.engine_kind,
            chrome_focused: self.chrome_handle.is_focused(window),
            command_bar_text: self.command_bar_text.clone(),
            sidebar_visible: self.sidebar_visible,
            window_size: [
                f32::from(window.bounds().size.width),
                f32::from(window.bounds().size.height),
            ],
        }
    }
}

impl Focusable for BbrApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.chrome_handle.clone()
    }
}

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome_focused = self.chrome_handle.is_focused(window);
        let status_text: SharedString = self.shell.nav.status_line().into();

        let nav_dot = |label: &'static str, enabled: bool| {
            let (color, bg) = if enabled {
                (rgb(0xe8e8e8), rgb(0x333333))
            } else {
                (rgb(0x555555), rgb(0x222222))
            };
            div()
                .id(label)
                .px_8()
                .py_4()
                .rounded_md()
                .bg(bg)
                .text_color(color)
                .child(label)
        };

        let sidebar = div()
            .id("bbr-sidebar")
            .w(px(SIDEBAR_WIDTH_PX))
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            .border_r_1()
            .border_color(rgb(0x333333))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| this.focus_chrome(window, cx)),
            )
            .child(
                div()
                    .id("bbr-brand")
                    .px_16()
                    .py_12()
                    .text_xl()
                    .text_color(rgb(0xf2f2f2))
                    .child("BBR"),
            )
            .child(
                div()
                    .id("bbr-nav")
                    .flex()
                    .flex_row()
                    .gap_8()
                    .px_12()
                    .py_8()
                    .child(nav_dot("◀", self.shell.nav.can_go_back).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, _cx| this.go_back()),
                    ))
                    .child(nav_dot("▶", self.shell.nav.can_go_forward).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, _cx| this.go_forward()),
                    ))
                    .child(nav_dot("⟳", true).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, _cx| this.reload()),
                    )),
            )
            .child(
                div()
                    .id("bbr-command-bar")
                    .mx_12()
                    .my_8()
                    .px_12()
                    .py_8()
                    .rounded_md()
                    .bg(if chrome_focused {
                        rgb(0x2c2c2c)
                    } else {
                        rgb(0x232323)
                    })
                    .border_1()
                    .border_color(if chrome_focused {
                        rgb(0x6aa5ff)
                    } else {
                        rgb(0x3c3c3c)
                    })
                    .flex()
                    .flex_row()
                    .gap_8()
                    .child(div().text_color(rgb(0x9a9a9a)).child("▶"))
                    .child(
                        div()
                            .id("bbr-command-text")
                            .flex_1()
                            .text_color(rgb(0xe8e8e8))
                            .child(if self.command_bar_text.is_empty() {
                                SharedString::from("Type a URL…")
                            } else {
                                SharedString::from(self.command_bar_text.clone())
                            }),
                    )
                    .when(chrome_focused && self.command_bar_active, |el| {
                        el.child(
                            div()
                                .id("bbr-command-cursor")
                                .w(px(2.0))
                                .h_full()
                                .bg(rgb(0x6aa5ff)),
                        )
                    }),
            )
            .child(
                div()
                    .id("bbr-status")
                    .px_16()
                    .py_8()
                    .text_color(rgb(0x9a9a9a))
                    .text_xs()
                    .child(status_text),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("bbr-sidebar-toggle")
                    .px_16()
                    .py_8()
                    .text_color(rgb(0x9a9a9a))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, cx| this.toggle_sidebar(cx)),
                    )
                    .child("Sidebar (Cmd-B)"),
            );

        div()
            .id("bbr-root")
            .track_focus(&self.chrome_handle)
            .key_context("bbr_chrome")
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(0x141414))
            .on_key_down(cx.listener(Self::on_chrome_key_down))
            .child(sidebar)
            .child(
                // The content area is entirely the native web-view slot; GPUI
                // draws nothing here (the WKWebView shows through).
                NativeWebViewSlot::new(self.engine.clone()),
            )
    }
}
