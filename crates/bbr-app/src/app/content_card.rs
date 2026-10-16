//! The seated content card (DESIGN.md §2, §5, §11.4, §13): the content
//! area sits in a rounded card on the chassis instead of running edge to
//! edge against the window frame.
//!
//! The chassis carries the rail's paint (one continuous body), the card
//! carries `material:canvas` behind the native slot and the static
//! `elev:1.content` shadow, and the slot's native layer clips the web view
//! to the card radius (`bbr-web-macos` hosting layer). The top chassis band
//! is empty, draggable titlebar paint: it keeps the native traffic lights
//! off web content when the sidebar is collapsed and never hosts a control.
//! Native full screen is full-bleed. The geometry authority is
//! [`bbr_ui::card::CardInsets`].

use gpui::{div, prelude::*, px, AnyElement, Context, MouseButton, Window};

use bbr_ui::card::{
    content_card_edge, content_card_shadow, corner_covers, edge_layer, CardInsets, CARD_SEAM,
    CHASSIS_BAND_ID, CONTENT_CARD_ID,
};
use bbr_ui::theme::Theme;

use super::BbrApp;

impl BbrApp {
    /// The card frame for the current window state: sidebar visibility,
    /// native full screen, and the native titlebar band height.
    pub(crate) fn content_card_frame(&self, window: &Window) -> CardInsets {
        CardInsets::resolve(
            self.sidebar_visible,
            self.is_full_screen(window),
            self.safe_area(window).height,
        )
    }

    /// The window's native full-screen state (testkit builds may present a
    /// forced state, see `SetFullScreenOverride`).
    pub(crate) fn is_full_screen(&self, window: &Window) -> bool {
        #[cfg(feature = "testkit")]
        if let Some(full_screen) = self.full_screen_override {
            return full_screen;
        }
        window.is_fullscreen()
    }

    /// Seat `content` in the card on the chassis. Inset, radius and shadow
    /// come from the same `card` the slots clip to, so all three land in
    /// the frame that changes the sidebar width or the full-screen state.
    pub(super) fn seated_content(
        &self,
        theme: &Theme,
        card: CardInsets,
        content: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let band = div()
            .id(CHASSIS_BAND_ID)
            .debug_selector(|| CHASSIS_BAND_ID.to_string())
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(card.top))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_titlebar_mouse_down));
        div()
            .id("bbr-chassis")
            .relative()
            .flex()
            .flex_row()
            .flex_1()
            .min_w_0()
            .h_full()
            // One tint layer in the browser root covers rail and chassis.
            // D6: the chassis shares the rail's static grain (beneath the
            // card, which covers the web content).
            .when(self.active_space_appearance().paper.grain, |chassis| {
                chassis.child(bbr_ui::components::grain::grain_layer(theme))
            })
            .pt(px(card.top))
            .pr(px(card.trailing))
            .pb(px(card.bottom))
            .pl(px(card.leading))
            .when(card.top > 0.0, |chassis| chassis.child(band))
            .child(
                div()
                    .id(CONTENT_CARD_ID)
                    .debug_selector(|| CONTENT_CARD_ID.to_string())
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_w_0()
                    .rounded(px(card.radius))
                    .when(card.shadow, |card| card.shadow(content_card_shadow(theme)))
                    .bg(theme.palette.surface_canvas)
                    .overflow_hidden()
                    .child(content)
                    // Testkit-only: where the card drew, for page-only
                    // visual readback (no element without `rect-probes`).
                    .children(bbr_ui::rect_probe::rect_probe(CONTENT_CARD_ID))
                    // §2: GPUI clips to rectangles, so a GPUI surface in the
                    // card would paint its corners over the arc; the covers
                    // paint the chassis back over those four wedges, above
                    // every GPUI child and beneath the self-clipping native
                    // view. The chassis-corner wedges follow the D4 tint
                    // cross-fade exactly like the chassis itself.
                    .when(card.radius > 0.0 && !self.glass_enabled(), |card_el| {
                        card_el
                            .child(corner_covers(card.radius, self.active_chrome_tint()))
                            .children(bbr_ui::components::tint_fade::tint_fade_painted(
                                theme,
                                "chassis-corners",
                                self.tint_fade(),
                                |from| corner_covers(card.radius, from),
                            ))
                    }),
            )
            // §2: the rim and seam as exact concentric rings, laid out as the
            // card dilated by the edge and painted after the card so they
            // sit above its children and corner covers (and on the rail's
            // last 2 pt along the flush leading edge). Full screen has no
            // edge; the card's `shadow` flag carries that state.
            .when(card.shadow && card.radius > 0.0, |chassis| {
                chassis.child(
                    div()
                        .absolute()
                        .top(px(card.top - CARD_SEAM))
                        .left(px(card.leading - CARD_SEAM))
                        .right(px(card.trailing - CARD_SEAM))
                        .bottom(px(card.bottom - CARD_SEAM))
                        .child(edge_layer(card.radius, content_card_edge(theme))),
                )
            })
            .into_any()
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    /// The typed §2 evidence: the frame this window state lays out plus the
    /// active WKWebView's native clip read back from its backing layer.
    pub(crate) fn content_card_snapshot(
        &self,
        window: &Window,
    ) -> bbr_testkit::protocol::ContentCardSnapshot {
        let card = self.content_card_frame(window);
        #[cfg(target_os = "macos")]
        let native_radius = self.geometry_engine.as_ref().and_then(|engine| {
            let view = self.controller.read().snapshot().active_view?;
            engine.borrow().web_view_corner_radius(view)
        });
        #[cfg(not(target_os = "macos"))]
        let native_radius: Option<f64> = None;
        bbr_testkit::protocol::ContentCardSnapshot {
            inset_top: card.top,
            inset_trailing: card.trailing,
            inset_bottom: card.bottom,
            inset_leading: card.leading,
            radius: card.radius,
            shadow: card.shadow,
            full_screen: self.is_full_screen(window),
            native_radius: native_radius.map(|radius| radius as f32),
            chrome_material: self.chrome_material_snapshot(),
        }
    }
}
