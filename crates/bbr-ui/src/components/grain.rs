//! v2 D6 static paper grain: an optional, never-animated noise texture on
//! the BBR-owned tinted chrome (the rail and the chassis around the content
//! card). It is a background layer beneath every child, so it never paints
//! over web content (the card and the native view sit above it) or over the
//! controls' text.
//!
//! The texture is one deterministic 128 × 128 px tile generated in code (no
//! binary asset): straight-alpha specks of uniformly spread opacity up to
//! the tone's maximum, white on light chrome and black on dark chrome — the specks only ever move the
//! paper away from the text, so no text bar can drop. It is tiled at one
//! texel per device pixel and
//! anchored to the window origin, so the rail and the chassis read as one
//! continuous sheet. Painting it only issues image sprites from the one
//! cached texture; nothing requests a frame.

use std::sync::{Arc, OnceLock};

use gpui::{canvas, point, prelude::*, px, size, Bounds, ContentMask, Corners, Image, ImageFormat};

use crate::theme::{Theme, ThemeMode};

/// The tile edge in texels.
const TILE: u32 = 128;
/// The strongest speck's opacity per tone. The specks only raise text
/// contrast (the gate below), so the ceiling is set by how the paper reads:
/// a fine, visible tooth — a dark rail has less luminance to give, so its
/// black specks need more opacity for the same texture.
const GRAIN_ALPHA_WHITE: f32 = 0.28;
const GRAIN_ALPHA_BLACK: f32 = 0.36;

/// The static grain layer for one chrome surface: an absolute full-size
/// canvas painted beneath the surface's children.
pub fn grain_layer(theme: &Theme) -> gpui::AnyElement {
    let tone = Tone::for_mode(theme.mode);
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            let Some(image) = texture(tone).clone().get_render_image(window, cx) else {
                return;
            };
            let edge = px(TILE as f32 / window.scale_factor());
            let start_x = (bounds.left() / edge).floor() * edge;
            let start_y = (bounds.top() / edge).floor() * edge;
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                let mut y = start_y;
                while y < bounds.bottom() {
                    let mut x = start_x;
                    while x < bounds.right() {
                        let tile = Bounds::new(point(x, y), size(edge, edge));
                        let _ =
                            window.paint_image(tile, Corners::default(), image.clone(), 0, false);
                        x += edge;
                    }
                    y += edge;
                }
            });
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

/// The speck tone: the one that moves the chrome away from its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    /// White specks on light chrome (dark text).
    White,
    /// Black specks on dark chrome (light text).
    Black,
}

impl Tone {
    fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Tone::White,
            ThemeMode::Dark => Tone::Black,
        }
    }

    fn max_alpha(self) -> f32 {
        match self {
            Tone::White => GRAIN_ALPHA_WHITE,
            Tone::Black => GRAIN_ALPHA_BLACK,
        }
    }

    fn value(self) -> u8 {
        match self {
            Tone::White => 0xFF,
            Tone::Black => 0x00,
        }
    }
}

/// The cached grain texture per tone (PAM `RGB_ALPHA`, decoded by GPUI's
/// image pipeline).
fn texture(tone: Tone) -> &'static Arc<Image> {
    static WHITE: OnceLock<Arc<Image>> = OnceLock::new();
    static BLACK: OnceLock<Arc<Image>> = OnceLock::new();
    let cell = match tone {
        Tone::White => &WHITE,
        Tone::Black => &BLACK,
    };
    cell.get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Pnm, pam(&texels(tone)))))
}

/// The deterministic straight-alpha RGBA texels of the tile: every other
/// texel (on average) is a speck of `tone` whose opacity is spread
/// uniformly up to the tone's maximum; the rest is clear, which keeps the
/// sheet's mean shift of the tint small (a quarter of the maximum).
fn texels(tone: Tone) -> Vec<[u8; 4]> {
    let mut state: u32 = 0x9E37_79B9;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    };
    (0..TILE * TILE)
        .map(|_| {
            let bits = next();
            let strength = if bits & 0x100 == 0 {
                0.0
            } else {
                (bits & 0xFF) as f32 / 255.0
            };
            let alpha = (tone.max_alpha() * strength * 255.0).round() as u8;
            let value = tone.value();
            [value, value, value, alpha]
        })
        .collect()
}

/// A PAM (`P7`) container for the RGBA texels.
fn pam(texels: &[[u8; 4]]) -> Vec<u8> {
    let mut bytes = format!(
        "P7\nWIDTH {TILE}\nHEIGHT {TILE}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n"
    )
    .into_bytes();
    bytes.extend(texels.iter().flatten());
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{contrast, over};
    use crate::theme::{ContrastMode, Density};
    use bbr_domain::{SpaceTheme, TintIntensity};

    #[test]
    fn the_tile_is_deterministic_bounded_and_one_toned() {
        for tone in [Tone::White, Tone::Black] {
            let first = texels(tone);
            assert_eq!(first, texels(tone), "deterministic");
            assert_eq!(first.len(), (TILE * TILE) as usize);
            let max = (tone.max_alpha() * 255.0).round() as u8;
            assert!(first.iter().all(|texel| texel[3] <= max));
            assert!(first.iter().any(|texel| texel[3] == max), "reaches the max");
            let clear = first.iter().filter(|texel| texel[3] == 0).count();
            assert!(
                (0.45..0.56).contains(&(clear as f64 / first.len() as f64)),
                "about half the sheet is clear"
            );
            assert!(first.iter().all(|texel| texel[..3] == [tone.value(); 3]));
            let header =
                b"P7\nWIDTH 128\nHEIGHT 128\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n";
            let container = pam(&first);
            assert_eq!(&container[..header.len()], header);
            assert_eq!(container.len(), header.len() + first.len() * 4);
        }
        assert_eq!(Tone::for_mode(ThemeMode::Light), Tone::White);
        assert_eq!(Tone::for_mode(ThemeMode::Dark), Tone::Black);
    }

    /// D6 + D3: the grain never lowers a rail text bar — the strongest speck
    /// of the mode's tone over the untinted rail and every token ×
    /// intensity moves the paper away from `ink`, `ink.secondary` and
    /// `accent.ink` (each ratio only grows), so every bar still holds.
    #[test]
    fn the_strongest_speck_only_raises_the_rail_text_contrast() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            for contrast_mode in [ContrastMode::Standard, ContrastMode::Increased] {
                let theme = Theme::new(mode, contrast_mode, Density::Normal);
                let p = theme.palette;
                let tone = Tone::for_mode(mode);
                let mut speck = gpui::rgb(u32::from(tone.value()) * 0x010101);
                speck.a = tone.max_alpha();
                let mut rails = vec![p.surface_sidebar];
                for token in SpaceTheme::ALL {
                    for step in TintIntensity::ALL {
                        rails.push(crate::theme::space_tint(token, step, &theme).chrome);
                    }
                }
                for rail in rails {
                    let grained = over(speck, rail);
                    for (name, ink) in [
                        ("ink", p.ink),
                        ("secondary", p.ink_secondary),
                        ("accent.ink", p.accent_ink),
                    ] {
                        let (before, after) = (contrast(ink, rail), contrast(ink, grained));
                        assert!(
                            after >= before,
                            "{mode:?} {contrast_mode:?}: {name} {after:.3} ≥ {before:.3}"
                        );
                    }
                }
            }
        }
    }
}
