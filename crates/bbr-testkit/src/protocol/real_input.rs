//! Real input (reality-gaps packet, item C): the harness builds real AppKit
//! events and hands them to `NSApplication.sendEvent:` inside the app, so a
//! click or a key press takes AppKit's own dispatch — hit testing, the key
//! window and first responder, menu key equivalents, WebKit's event
//! handling — instead of calling the method a listener would call. Nothing
//! is posted to the operating system and no permission is involved.

use serde::{Deserialize, Serialize};

/// Which mouse event to send.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealMousePhase {
    /// The button goes down.
    Down,
    /// The button comes up.
    Up,
    /// The pointer moves with the button down.
    Dragged,
    /// The pointer moves with no button down (hover).
    Moved,
}

/// Which mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RealMouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

/// A modifier key held during the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealModifier {
    Command,
    Shift,
    Option,
    Control,
}

/// Where AppKit took a delivered event, read back right after
/// `sendEvent:` returned.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RealInputReadback {
    /// The event resolved to the target window (`-[NSEvent window]`).
    pub routed_to_window: bool,
    /// The target window was the application's key window.
    pub window_was_key: bool,
    /// The class of the target window's first responder after delivery
    /// (`WKWebView`, GPUI's view class…).
    pub first_responder: Option<String>,
}
