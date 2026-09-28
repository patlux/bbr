//! Blueprint D09: the Peek presentation story — the exact production
//! `bbr_ui::surfaces::peek_wash` over a stand-in origin page, at the
//! production `bbr_ui::card::PeekFrame` geometry. The stand-in page is
//! plain GPUI chrome (the app never samples or redraws web content; the
//! real origin is a native WKWebView under the portal). The Peek bar
//! itself is app-owned portal chrome and is exercised by the real-app
//! `e2e_shell_peek` suite; the story reserves its rect.

use bbr_ui::card::{PeekFrame, CARD_RADIUS};
use bbr_ui::surfaces::peek_wash::peek_wash;
use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, px, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page};

/// The gallery test-target selector prefix for the Peek families.
pub const PEEK_FAMILY_SELECTOR: &str = "peek-family-";

const CARD: [f32; 2] = [680.0, 420.0];

/// A stand-in page: a heading bar and three text lines on the canvas.
fn stand_in_page(theme: &Theme, width: f32, height: f32) -> impl IntoElement {
    let p = theme.palette;
    let line = |w: f32, h: f32| div().w(px(w)).h(px(h)).rounded(px(3.0)).bg(p.hairline);
    div()
        .w(px(width))
        .h(px(height))
        .p(px(28.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .bg(p.surface_canvas)
        .child(line(220.0, 14.0))
        .child(line(360.0, 8.0))
        .child(line(320.0, 8.0))
        .child(line(300.0, 8.0))
}

pub fn render_peek(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let bar_height = f32::from(theme.density.row());
    let frame = PeekFrame::resolve([0.0, 0.0, CARD[0], CARD[1]], CARD_RADIUS, bar_height);
    let [px_, py, pw, ph] = frame.preview;
    let [bx, by, bw, bh] = frame.bar;
    let family = div()
        .debug_selector(|| format!("{PEEK_FAMILY_SELECTOR}0"))
        .flex()
        .flex_col()
        .gap_2()
        .child(section_heading(
            theme,
            "Inset preview over the washed origin",
        ))
        .child(
            div()
                .relative()
                .w(px(CARD[0]))
                .h(px(CARD[1]))
                .rounded(px(CARD_RADIUS))
                .overflow_hidden()
                .child(stand_in_page(theme, CARD[0], CARD[1]))
                // The preview stand-in sits where the native preview goes.
                .child(
                    div()
                        .absolute()
                        .left(px(px_))
                        .top(px(py))
                        .rounded(px(bbr_ui::card::PEEK_PREVIEW_RADIUS))
                        .overflow_hidden()
                        .child(stand_in_page(theme, pw, ph)),
                )
                .children(peek_wash(theme, frame))
                // The bar's rect (app-owned portal chrome).
                .child(
                    div()
                        .absolute()
                        .left(px(bx))
                        .top(px(by))
                        .w(px(bw))
                        .h(px(bh))
                        .rounded(theme.radius.large())
                        .bg(theme.palette.surface_overlay)
                        .border_1()
                        .border_color(theme.palette.hairline),
                ),
        )
        .into_any();
    story_page(ctx, "Peek presentation", vec![family])
}
