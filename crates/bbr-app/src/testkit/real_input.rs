//! Reality gaps (item C): the `RealMouse` / `RealKey` seam. Builds one real
//! AppKit event for the current browser window and hands it to
//! `NSApplication.sendEvent:` (`bbr_web_macos::input_events`), then records
//! where AppKit took it for the snapshot's `real_input`. AppKit re-enters
//! GPUI (and WebKit), which updates the app itself, so this runs on the
//! pump between updates rather than inside `dispatch`.

use bbr_testkit::protocol::{Error, RealInputReadback, Request, Response};
use gpui::{AsyncApp, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

pub(super) fn drive(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    async_cx: &mut AsyncApp,
) -> Response {
    #[cfg(target_os = "macos")]
    {
        use bbr_web_macos::input_events as input;
        let view = async_cx.update(|cx| {
            (**window)
                .update(cx, |_root, window, _cx| {
                    super::text_input::root_view(window)
                })
                .ok()
                .flatten()
        });
        let Some(view) = view else {
            return Response::Error(Error::new("no_view", "the browser window has no root view"));
        };
        let delivered = match req {
            Request::RealMouse {
                phase,
                button,
                x,
                y,
                modifiers,
                click_count,
            } => {
                // SAFETY: `view` is the live GPUI root NSView of the browser
                // window; the pump runs on the main thread and holds no app
                // borrow here.
                unsafe {
                    input::send_mouse(
                        view,
                        mouse_phase(*phase),
                        mouse_button(*button),
                        *x,
                        *y,
                        modifier_set(modifiers),
                        *click_count,
                    )
                }
            }
            Request::RealClick {
                button,
                x,
                y,
                modifiers,
            } => {
                let button = mouse_button(*button);
                let modifiers = modifier_set(modifiers);
                // SAFETY: as above. `send_mouse` refuses a window that is
                // not key before sending; within this turn nothing else
                // runs between the three events.
                unsafe {
                    input::send_mouse(view, input::Phase::Moved, button, *x, *y, modifiers, 1)
                        .and_then(|_| {
                            input::send_mouse(
                                view,
                                input::Phase::Down,
                                button,
                                *x,
                                *y,
                                modifiers,
                                1,
                            )
                        })
                        .and_then(|_| {
                            input::send_mouse(view, input::Phase::Up, button, *x, *y, modifiers, 1)
                        })
                }
            }
            Request::RealKey { keystroke } => match gpui::Keystroke::parse(keystroke) {
                Ok(parsed) => {
                    let modifiers = input::Modifiers {
                        command: parsed.modifiers.platform,
                        shift: parsed.modifiers.shift,
                        option: parsed.modifiers.alt,
                        control: parsed.modifiers.control,
                    };
                    // SAFETY: as above.
                    unsafe { input::send_key(view, &parsed.key, modifiers) }
                }
                Err(error) => Err(error.to_string()),
            },
            _ => return Response::Error(Error::new("not_real_input", "not a real-input request")),
        };
        let readback = match delivered {
            Ok(readback) => readback,
            Err(error) if error == input::NOT_KEY => {
                return Response::Error(Error::new("not_key", error))
            }
            Err(error) => return Response::Error(Error::new("real_input", error)),
        };
        async_cx.update(|cx| {
            let _ = with_app(window, app, cx, |view, _window, _cx| {
                view.real_input_readback = Some(RealInputReadback {
                    routed_to_window: readback.routed_to_window,
                    window_was_key: readback.window_was_key,
                    first_responder: readback.first_responder,
                });
            });
        });
        Response::Ok
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (req, window, app, async_cx);
        Response::Error(Error::new("unsupported", "no native input events"))
    }
}

#[cfg(target_os = "macos")]
fn mouse_phase(phase: bbr_testkit::protocol::RealMousePhase) -> bbr_web_macos::input_events::Phase {
    use bbr_testkit::protocol::RealMousePhase as P;
    use bbr_web_macos::input_events::Phase;
    match phase {
        P::Down => Phase::Down,
        P::Up => Phase::Up,
        P::Dragged => Phase::Dragged,
        P::Moved => Phase::Moved,
    }
}

#[cfg(target_os = "macos")]
fn mouse_button(
    button: bbr_testkit::protocol::RealMouseButton,
) -> bbr_web_macos::input_events::Button {
    use bbr_testkit::protocol::RealMouseButton as B;
    use bbr_web_macos::input_events::Button;
    match button {
        B::Left => Button::Left,
        B::Right => Button::Right,
        B::Middle => Button::Middle,
    }
}

#[cfg(target_os = "macos")]
fn modifier_set(
    modifiers: &[bbr_testkit::protocol::RealModifier],
) -> bbr_web_macos::input_events::Modifiers {
    use bbr_testkit::protocol::RealModifier as M;
    let mut set = bbr_web_macos::input_events::Modifiers::default();
    for modifier in modifiers {
        match modifier {
            M::Command => set.command = true,
            M::Shift => set.shift = true,
            M::Option => set.option = true,
            M::Control => set.control = true,
        }
    }
    set
}
