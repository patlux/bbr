//! Blueprint D08: the Portal's overlay layer — the toast (bottom-centre of
//! the card) and the zoom readout (top-centre) above any active
//! presentation. The toast hugs its content, so its rendered bounds are
//! recorded here and handed back to the app, which converges the Portal's
//! hit regions (see `crate::app::transient`).

use gpui::{canvas, div, prelude::*, px, AnyElement, Context};

use bbr_ui::surfaces::toast::{toast, toast_rect, zoom_readout, zoom_readout_rect, TOAST_HEIGHT};

use super::probe::PortalRoot;

/// Records an overlay's laid-out bounds (window points) for its hit region;
/// a change re-syncs the Portal once. The toast and the banner hug their
/// content, so their regions are measured, not predicted.
fn bounds_recorder(
    bounds: std::rc::Rc<std::cell::Cell<Option<[f32; 4]>>>,
    app: gpui::Entity<crate::app::BbrApp>,
    browser_window: gpui::WindowHandle<gpui_component::Root>,
) -> impl IntoElement {
    canvas(
        move |item_bounds, _window, cx| {
            let rect = [
                f32::from(item_bounds.origin.x),
                f32::from(item_bounds.origin.y),
                f32::from(item_bounds.size.width),
                f32::from(item_bounds.size.height),
            ];
            if bounds.get() != Some(rect) {
                bounds.set(Some(rect));
                let app = app.clone();
                cx.defer(move |cx| {
                    let _ = crate::window_seam::with_app(
                        &browser_window,
                        &app,
                        cx,
                        |view, _window, cx| view.sync_portal_overlays(cx),
                    );
                });
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

/// The overlay elements for the current app state (empty at rest).
pub(super) fn render_overlays(root: &PortalRoot, cx: &mut Context<PortalRoot>) -> Vec<AnyElement> {
    let Some(app) = root.app.clone() else {
        return Vec::new();
    };
    let (theme, card, toast_state, readout, bounds, notice, notice_bounds) = {
        let app = app.read(cx);
        (
            app.theme(),
            app.transient.card,
            app.transient.toast.clone(),
            app.transient.zoom_readout,
            app.transient.toast_bounds.clone(),
            app.transient.notice.clone(),
            app.transient.notice_bounds.clone(),
        )
    };
    let Some(card) = card else {
        return Vec::new();
    };
    let browser_window = root.browser_window;
    let mut overlays = render_permission_overlays(root, cx);
    // Blueprint D23: the collapsed split's chip and pane list.
    overlays.extend(super::split_chip::render(root, cx));
    overlays.extend(super::split_chip::render_drop(root, cx));
    if let Some(state) = toast_state {
        let [x, y, width, _] = toast_rect(card, card[2]);
        let action_app = app.clone();
        let element = toast(&theme, &state.model, move |_event, _window, cx| {
            let app = action_app.clone();
            cx.defer(move |cx| {
                let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                    view.toast_action(cx)
                });
            });
        });
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(width))
                .h(px(TOAST_HEIGHT))
                .flex()
                .justify_center()
                .child(div().relative().child(element).child(bounds_recorder(
                    bounds,
                    app.clone(),
                    browser_window,
                )))
                .into_any_element(),
        );
    }
    if let Some(notice) = notice {
        let [x, y, width] = bbr_ui::surfaces::toast::banner_slot(card);
        let dismiss_app = app.clone();
        let banner = bbr_ui::surfaces::toast::info_banner(
            &theme,
            notice.message.into(),
            "Dismiss".into(),
            move |_event, _window, cx| {
                let app = dismiss_app.clone();
                cx.defer(move |cx| {
                    let _ =
                        crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                            view.dismiss_restore_notice(cx)
                        });
                });
            },
        );
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(width))
                .child(div().relative().child(banner).child(bounds_recorder(
                    notice_bounds,
                    app.clone(),
                    browser_window,
                )))
                .into_any_element(),
        );
    }
    // Blueprint D13: the blocked pop-up notice, under the restore notice
    // when both show.
    let (popup_message, popup_bounds, notice_shown) = {
        let app = app.read(cx);
        (
            app.popup_notice_message(),
            app.permissions.popup_bounds.clone(),
            app.transient.notice.is_some(),
        )
    };
    if let Some(message) = popup_message {
        let [x, y, width] = bbr_ui::surfaces::toast::banner_slot(card);
        let y = if notice_shown { y + 52.0 } else { y };
        let action = |app: gpui::Entity<crate::app::BbrApp>, allow: bool| {
            Box::new(
                move |_event: &gpui::MouseDownEvent,
                      _window: &mut gpui::Window,
                      cx: &mut gpui::App| {
                    let app = app.clone();
                    cx.defer(move |cx| {
                        let _ = crate::window_seam::with_app(
                            &browser_window,
                            &app,
                            cx,
                            |view, window, cx| {
                                if allow {
                                    view.popup_always_allow(window, cx)
                                } else {
                                    view.popup_open(window, cx)
                                }
                            },
                        );
                    });
                },
            ) as bbr_ui::surfaces::toast::PressFn
        };
        let banner = bbr_ui::surfaces::toast::banner(
            &theme,
            [
                "bbr-popup-blocked",
                "bbr-popup-blocked-message",
                "bbr-popup-blocked-appear",
            ],
            message.into(),
            vec![
                bbr_ui::surfaces::toast::BannerAction {
                    id: "bbr-popup-blocked-open".into(),
                    label: "Open".into(),
                    on_press: action(app.clone(), false),
                },
                bbr_ui::surfaces::toast::BannerAction {
                    id: "bbr-popup-blocked-allow".into(),
                    label: "Always Allow".into(),
                    on_press: action(app.clone(), true),
                },
            ],
        );
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(width))
                .child(div().relative().child(banner).child(bounds_recorder(
                    popup_bounds,
                    app.clone(),
                    browser_window,
                )))
                .into_any_element(),
        );
    }
    // Blueprint D13: the no-app banner.
    let (no_app, no_app_bounds, popup_shown) = {
        let app = app.read(cx);
        (
            app.open_app
                .banner
                .as_ref()
                .map(|(message, _)| message.clone()),
            app.open_app.banner_bounds.clone(),
            app.popup_notice_visible().is_some(),
        )
    };
    if let Some(message) = no_app {
        let [x, y, width] = bbr_ui::surfaces::toast::banner_slot(card);
        let stacked = usize::from(notice_shown) + usize::from(popup_shown);
        let banner = bbr_ui::surfaces::toast::banner(
            &theme,
            [
                "bbr-external-no-app",
                "bbr-external-no-app-message",
                "bbr-external-no-app-appear",
            ],
            message.into(),
            Vec::new(),
        );
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y + 52.0 * stacked as f32))
                .w(px(width))
                .child(div().relative().child(banner).child(bounds_recorder(
                    no_app_bounds,
                    app.clone(),
                    browser_window,
                )))
                .into_any_element(),
        );
    }
    if let Some(readout) = readout {
        let [x, y, _, _] = zoom_readout_rect(card);
        let minus_app = app.clone();
        let plus_app = app.clone();
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .child(zoom_readout(
                    &theme,
                    readout.percent,
                    readout.generation,
                    move |_event, _window, cx| {
                        let app = minus_app.clone();
                        cx.defer(move |cx| {
                            let _ = crate::window_seam::with_app(
                                &browser_window,
                                &app,
                                cx,
                                |view, _w, cx| view.zoom_out(cx),
                            );
                        });
                    },
                    move |_event, _window, cx| {
                        let app = plus_app.clone();
                        cx.defer(move |cx| {
                            let _ = crate::window_seam::with_app(
                                &browser_window,
                                &app,
                                cx,
                                |view, _w, cx| view.zoom_in(cx),
                            );
                        });
                    },
                ))
                .into_any_element(),
        );
    }
    overlays
}

/// Blueprint D11: the permission prompt and the site-settings popover,
/// hanging from the address control (their bounds are their hit regions).
fn render_permission_overlays(root: &PortalRoot, cx: &mut Context<PortalRoot>) -> Vec<AnyElement> {
    use bbr_ui::surfaces::permissions::{permission_prompt, site_settings_popover};
    let Some(app) = root.app.clone() else {
        return Vec::new();
    };
    let (theme, anchor, prompt, popover, prompt_bounds, popover_bounds) = {
        let app = app.read(cx);
        (
            app.theme(),
            app.permission_anchor(),
            app.permission_prompt_model(),
            app.site_settings_model(),
            app.permissions.prompt_bounds.clone(),
            app.permissions.popover_bounds.clone(),
        )
    };
    let Some((x, y)) = anchor else {
        return Vec::new();
    };
    let browser_window = root.browser_window;
    let mut overlays = Vec::new();
    // Blueprint D13: the space chooser for a link from another app, at the
    // palette's place (top-centre of the card).
    let chooser = app.read(cx).space_chooser_model();
    if let (Some(model), Some([cx0, cy0, cw, _])) = (chooser, app.read(cx).transient.card) {
        let events_app = app.clone();
        let card = bbr_ui::surfaces::space_chooser::space_chooser(
            &theme,
            &model,
            std::rc::Rc::new(move |index, _window, cx| {
                let app = events_app.clone();
                cx.defer(move |cx| {
                    let _ = crate::window_seam::with_app(
                        &browser_window,
                        &app,
                        cx,
                        |view, window, cx| view.space_chooser_choose(index, window, cx),
                    );
                });
            }),
        );
        let width = bbr_ui::surfaces::space_chooser::SPACE_CHOOSER_WIDTH;
        overlays.push(
            div()
                .absolute()
                .left(px(cx0 + (cw - width) / 2.0))
                .top(px(cy0 + 72.0))
                .child(bbr_ui::surfaces::floating::appear(
                    &theme,
                    "bbr-space-chooser-appear",
                    card,
                ))
                .into_any_element(),
        );
    }
    // Blueprint D13: the open-in-app sheet blocks the card.
    let open_app = app.read(cx).open_app_model();
    if let (Some(model), Some(card)) = (open_app, app.read(cx).transient.card) {
        let events_app = app.clone();
        overlays.push(
            bbr_ui::surfaces::open_app::open_app_sheet(
                &theme,
                &model,
                card,
                std::rc::Rc::new(move |event, _window, cx| {
                    let app = events_app.clone();
                    cx.defer(move |cx| {
                        let _ = crate::window_seam::with_app(
                            &browser_window,
                            &app,
                            cx,
                            |view, window, cx| view.on_open_app_event(event, window, cx),
                        );
                    });
                }),
            )
            .into_any_element(),
        );
    }
    // Blueprint D12: the client-certificate sheet blocks the card.
    let (certificate, card) = {
        let app = app.read(cx);
        (app.client_certificate_model(), app.transient.card)
    };
    if let (Some(model), Some(card)) = (certificate, card) {
        let events_app = app.clone();
        overlays.push(
            bbr_ui::surfaces::client_certificate::client_certificate_sheet(
                &theme,
                &model,
                card,
                std::rc::Rc::new(move |event, _window, cx| {
                    let app = events_app.clone();
                    cx.defer(move |cx| {
                        let _ = crate::window_seam::with_app(
                            &browser_window,
                            &app,
                            cx,
                            |view, window, cx| view.on_client_certificate_event(event, window, cx),
                        );
                    });
                }),
            )
            .into_any_element(),
        );
    }
    // The popover and the prompt share the anchor; the prompt (a page
    // waiting for an answer) wins.
    if let Some(model) = prompt {
        let events_app = app.clone();
        let card = permission_prompt(
            &theme,
            &model,
            std::rc::Rc::new(move |event, _window, cx| {
                use bbr_ui::surfaces::permissions::PermissionPromptEvent as E;
                let app = events_app.clone();
                cx.defer(move |cx| {
                    let _ =
                        crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                            match event {
                                E::Remember => view.toggle_permission_remember(cx),
                                E::Deny => view.decide_current_permission(false, cx),
                                E::Allow => view.decide_current_permission(true, cx),
                            }
                        });
                });
            }),
        );
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .child(
                    div()
                        .relative()
                        .child(bbr_ui::surfaces::floating::appear(
                            &theme,
                            "bbr-permission-prompt-appear",
                            card,
                        ))
                        .child(bounds_recorder(prompt_bounds, app.clone(), browser_window)),
                )
                .into_any_element(),
        );
    } else if let Some(model) = popover {
        let events_app = app.clone();
        let card = site_settings_popover(
            &theme,
            &model,
            std::rc::Rc::new(move |event, _window, cx| {
                let app = events_app.clone();
                cx.defer(move |cx| {
                    let _ =
                        crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                            view.on_site_settings_event(event, cx)
                        });
                });
            }),
        );
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .child(
                    div()
                        .relative()
                        .child(bbr_ui::surfaces::floating::appear(
                            &theme,
                            "bbr-site-settings-appear",
                            card,
                        ))
                        .child(bounds_recorder(popover_bounds, app.clone(), browser_window)),
                )
                .into_any_element(),
        );
    }
    overlays
}
