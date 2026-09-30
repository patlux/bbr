//! Real input events for the test harness (reality-gaps packet, item C;
//! `testkit` feature only).
//!
//! Builds the `NSEvent` a mouse or keyboard produces — type, window
//! location, modifier flags, click count, window number, key code and
//! characters — and hands it to `NSApplication.sendEvent:` inside the app.
//! AppKit then does what it does with a user's event: it resolves the
//! window, hit-tests the view (GPUI's view or the WKWebView), offers key
//! equivalents to the window and the main menu, and sends the rest to the
//! first responder. Nothing is posted to the operating system (no
//! `CGEventPost`, no event tap) and no permission is involved. A middle
//! button takes one CoreGraphics round trip because `NSEvent` has no
//! constructor argument for the button number.
//!
//! An event is only sent while its window is key: AppKit gives an inactive
//! window's first click to activation (GPUI does not accept the first
//! mouse) and key presses only to the key window, so an event for a window
//! that lost key status to another app is refused ([`NOT_KEY`]) rather than
//! half-delivered; the harness makes the window key again and resends it.
//!
//! Every call must run on the main thread **outside** any GPUI app borrow:
//! AppKit re-enters GPUI, which updates the app itself.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2::{msg_send, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSView, NSWindow};
use objc2_foundation::{NSPoint, NSString};

/// Which mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Down,
    Up,
    Dragged,
    Moved,
}

/// Which mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
}

/// Modifier keys held during the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub command: bool,
    pub shift: bool,
    pub option: bool,
    pub control: bool,
}

/// Where AppKit took the event.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Readback {
    pub routed_to_window: bool,
    pub window_was_key: bool,
    pub first_responder: Option<String>,
}

/// One key: the characters AppKit reports and the hardware key code (ANSI
/// US layout).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySpec {
    pub characters: String,
    pub key_code: u16,
}

extern "C" {
    /// CoreGraphics (linked through AppKit): set one integer field of an
    /// event object. Used on the in-process copy only; never posted.
    fn CGEventSetIntegerValueField(event: *mut c_void, field: u32, value: i64);
}

/// The error of an event refused because its window is not key.
pub const NOT_KEY: &str = "the window is not key";

/// `kCGMouseEventButtonNumber`.
const MOUSE_EVENT_BUTTON_NUMBER: u32 = 3;

/// Send one mouse event at the content point `(x, y)` (points, top-left
/// origin) of the window hosting `ns_view`.
///
/// # Safety
///
/// `ns_view` must be a live `NSView*`; call on the main thread outside any
/// GPUI app borrow.
pub unsafe fn send_mouse(
    ns_view: *mut c_void,
    phase: Phase,
    button: Button,
    x: f64,
    y: f64,
    modifiers: Modifiers,
    click_count: i64,
) -> Result<Readback, String> {
    let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
    // SAFETY: the caller promises a live NSView*.
    let view = unsafe { ns_view.cast::<NSView>().as_ref() }.ok_or("no view")?;
    let window = view.window().ok_or("the view has no window")?;
    if !window.isKeyWindow() {
        return Err(NOT_KEY.into());
    }
    let location = window_point(view, x, y);
    let kind = mouse_type(phase, button);
    let pressure = if matches!(phase, Phase::Down | Phase::Dragged) {
        1.0
    } else {
        0.0
    };
    let event = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
        kind,
        location,
        flags(modifiers),
        uptime(),
        window.windowNumber(),
        None,
        0,
        click_count as isize,
        pressure,
    )
    .ok_or("AppKit built no mouse event")?;
    let event = if button == Button::Middle && phase != Phase::Moved {
        // SAFETY: a live event on the main thread.
        unsafe { with_button_number(&event, 2) }?
    } else {
        event
    };
    Ok(deliver(mtm, &window, &event))
}

/// Send one key press (key down, then key up) for `key` in the
/// `DispatchKeystroke` spelling of its last segment (`"l"`, `"enter"`,
/// `"tab"`, `"left"`…) to the window hosting `ns_view`.
///
/// # Safety
///
/// `ns_view` must be a live `NSView*`; call on the main thread outside any
/// GPUI app borrow.
pub unsafe fn send_key(
    ns_view: *mut c_void,
    key: &str,
    modifiers: Modifiers,
) -> Result<Readback, String> {
    let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
    // SAFETY: the caller promises a live NSView*.
    let view = unsafe { ns_view.cast::<NSView>().as_ref() }.ok_or("no view")?;
    let window = view.window().ok_or("the view has no window")?;
    if !window.isKeyWindow() {
        return Err(NOT_KEY.into());
    }
    let spec = key_spec(key, modifiers.shift).ok_or_else(|| format!("unknown key {key:?}"))?;
    let characters = NSString::from_str(&spec.characters);
    let mut readback = Readback::default();
    for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
        let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            kind,
            NSPoint::new(0.0, 0.0),
            flags(modifiers),
            uptime(),
            window.windowNumber(),
            None,
            &characters,
            &characters,
            false,
            spec.key_code,
        )
        .ok_or("AppKit built no key event")?;
        readback = deliver(mtm, &window, &event);
    }
    Ok(readback)
}

/// The characters and key code of one key (US layout); `shift` upper-cases
/// a letter the way the keyboard does.
pub fn key_spec(key: &str, shift: bool) -> Option<KeySpec> {
    let named = |characters: &str, key_code: u16| {
        Some(KeySpec {
            characters: characters.to_string(),
            key_code,
        })
    };
    match key {
        "enter" => return named("\r", 36),
        "tab" => return named("\t", 48),
        "space" => return named(" ", 49),
        "backspace" => return named("\u{7f}", 51),
        "escape" => return named("\u{1b}", 53),
        "left" => return named("\u{f702}", 123),
        "right" => return named("\u{f703}", 124),
        "down" => return named("\u{f701}", 125),
        "up" => return named("\u{f700}", 126),
        _ => {}
    }
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    const CODES: &[(char, u16)] = &[
        ('a', 0),
        ('s', 1),
        ('d', 2),
        ('f', 3),
        ('h', 4),
        ('g', 5),
        ('z', 6),
        ('x', 7),
        ('c', 8),
        ('v', 9),
        ('b', 11),
        ('q', 12),
        ('w', 13),
        ('e', 14),
        ('r', 15),
        ('y', 16),
        ('t', 17),
        ('1', 18),
        ('2', 19),
        ('3', 20),
        ('4', 21),
        ('6', 22),
        ('5', 23),
        ('=', 24),
        ('9', 25),
        ('7', 26),
        ('-', 27),
        ('8', 28),
        ('0', 29),
        (']', 30),
        ('o', 31),
        ('u', 32),
        ('[', 33),
        ('i', 34),
        ('p', 35),
        ('l', 37),
        ('j', 38),
        ('\'', 39),
        ('k', 40),
        (';', 41),
        ('\\', 42),
        (',', 43),
        ('/', 44),
        ('n', 45),
        ('m', 46),
        ('.', 47),
        ('`', 50),
    ];
    let lower = ch.to_ascii_lowercase();
    let key_code = CODES.iter().find(|(c, _)| *c == lower)?.1;
    let characters = if shift && lower.is_ascii_alphabetic() {
        lower.to_ascii_uppercase().to_string()
    } else {
        lower.to_string()
    };
    Some(KeySpec {
        characters,
        key_code,
    })
}

fn mouse_type(phase: Phase, button: Button) -> NSEventType {
    match (phase, button) {
        (Phase::Moved, _) => NSEventType::MouseMoved,
        (Phase::Down, Button::Left) => NSEventType::LeftMouseDown,
        (Phase::Up, Button::Left) => NSEventType::LeftMouseUp,
        (Phase::Dragged, Button::Left) => NSEventType::LeftMouseDragged,
        (Phase::Down, Button::Right) => NSEventType::RightMouseDown,
        (Phase::Up, Button::Right) => NSEventType::RightMouseUp,
        (Phase::Dragged, Button::Right) => NSEventType::RightMouseDragged,
        (Phase::Down, Button::Middle) => NSEventType::OtherMouseDown,
        (Phase::Up, Button::Middle) => NSEventType::OtherMouseUp,
        (Phase::Dragged, Button::Middle) => NSEventType::OtherMouseDragged,
    }
}

fn flags(modifiers: Modifiers) -> NSEventModifierFlags {
    let mut flags = NSEventModifierFlags::empty();
    if modifiers.command {
        flags |= NSEventModifierFlags::Command;
    }
    if modifiers.shift {
        flags |= NSEventModifierFlags::Shift;
    }
    if modifiers.option {
        flags |= NSEventModifierFlags::Option;
    }
    if modifiers.control {
        flags |= NSEventModifierFlags::Control;
    }
    flags
}

fn uptime() -> f64 {
    objc2_foundation::NSProcessInfo::processInfo().systemUptime()
}

/// The window-coordinate location of the content point `(x, y)` of `view`
/// (top-left origin, as GPUI and the snapshot measure).
fn window_point(view: &NSView, x: f64, y: f64) -> NSPoint {
    let height = view.frame().size.height;
    let local = NSPoint::new(x, if view.isFlipped() { y } else { height - y });
    view.convertPoint_toView(local, None)
}

/// A copy of `event` with another button number.
///
/// # Safety
///
/// A live event on the main thread.
unsafe fn with_button_number(event: &NSEvent, button: i64) -> Result<Retained<NSEvent>, String> {
    // SAFETY: `-[NSEvent CGEvent]` returns the event's CGEventRef (owned by
    // the event, alive while it is); the field write edits that object only.
    let cg: *mut c_void = unsafe { msg_send![event, CGEvent] };
    if cg.is_null() {
        return Err("the event has no CGEvent".into());
    }
    // SAFETY: a live CGEventRef; `kCGMouseEventButtonNumber` is an integer
    // field of mouse events.
    unsafe { CGEventSetIntegerValueField(cg, MOUSE_EVENT_BUTTON_NUMBER, button) };
    // SAFETY: `+[NSEvent eventWithCGEvent:]` copies the live CGEventRef.
    let copy: Option<Retained<NSEvent>> =
        unsafe { msg_send![objc2::class!(NSEvent), eventWithCGEvent: cg] };
    copy.ok_or_else(|| "AppKit built no event from the CGEvent".into())
}

fn deliver(mtm: MainThreadMarker, window: &NSWindow, event: &NSEvent) -> Readback {
    // SAFETY: `-[NSEvent window]` is a plain getter on a live event.
    let target: Option<Retained<NSWindow>> = unsafe { msg_send![event, window] };
    let routed_to_window = target.is_some_and(|target| std::ptr::eq(&*target, window));
    NSApplication::sharedApplication(mtm).sendEvent(event);
    Readback {
        routed_to_window,
        window_was_key: window.isKeyWindow(),
        first_responder: window
            .firstResponder()
            .map(|responder| responder.class().name().to_string_lossy().into_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_carry_the_us_layout_code_and_the_shifted_character() {
        assert_eq!(
            key_spec("l", false),
            Some(KeySpec {
                characters: "l".into(),
                key_code: 37
            })
        );
        assert_eq!(
            key_spec("t", true),
            Some(KeySpec {
                characters: "T".into(),
                key_code: 17
            })
        );
        assert_eq!(key_spec("enter", false).map(|k| k.key_code), Some(36));
        assert_eq!(
            key_spec("left", false).map(|k| k.characters),
            Some("\u{f702}".into())
        );
        assert_eq!(key_spec("f13", false), None);
    }

    #[test]
    fn phases_and_buttons_map_to_appkit_types() {
        assert_eq!(
            mouse_type(Phase::Down, Button::Middle),
            NSEventType::OtherMouseDown
        );
        assert_eq!(
            mouse_type(Phase::Moved, Button::Right),
            NSEventType::MouseMoved
        );
        assert_eq!(
            mouse_type(Phase::Dragged, Button::Left),
            NSEventType::LeftMouseDragged
        );
        let all = Modifiers {
            command: true,
            shift: true,
            option: true,
            control: true,
        };
        assert_eq!(
            flags(all),
            NSEventModifierFlags::Command
                | NSEventModifierFlags::Shift
                | NSEventModifierFlags::Option
                | NSEventModifierFlags::Control
        );
    }
}
