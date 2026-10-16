//! One chrome background per browser window; solid controls and content are
//! above it. Never change the component theme or the native web views.

use super::BbrApp;
use bbr_ui::theme::{motion::Moment, ContrastMode, Theme};
use gpui::{div, prelude::*, AnimationExt, AnyElement, Context, Rgba, SharedString, Window};

/// Conservative tint coverage preserves the existing D3 text/accent ratios
/// even over a worst-case black/white desktop. AppKit supplies the actual blur.
const CHROME_TINT_ALPHA: f32 = 0.85;

pub(super) fn glass_tint(mut color: Rgba, theme: &Theme, enabled: bool) -> Rgba {
    if enabled && theme.contrast != ContrastMode::Increased {
        // Keep every compensated channel in gamut: vivid space tints need
        // more coverage than neutral Ink. At the worst backdrop, the result
        // is exactly the original contrast-safe tint, not a clipped hue.
        let alpha = match theme.mode {
            bbr_ui::theme::ThemeMode::Light => color.r.max(color.g).max(color.b),
            bbr_ui::theme::ThemeMode::Dark => 1.0 - color.r.min(color.g).min(color.b),
        }
        .max(CHROME_TINT_ALPHA);
        color.a = alpha;
        let channel = |value: f32| match theme.mode {
            bbr_ui::theme::ThemeMode::Light => value / alpha,
            bbr_ui::theme::ThemeMode::Dark => ((value - (1.0 - alpha)) / alpha).max(0.0),
        };
        color.r = channel(color.r);
        color.g = channel(color.g);
        color.b = channel(color.b);
    }
    color
}

impl BbrApp {
    pub(crate) fn prepare_chrome_material(&mut self, window: &mut Window) {
        #[cfg(target_os = "macos")]
        if !self.chrome_material_attempted {
            self.chrome_material_attempted = true;
            let pointer = crate::extract_ns_view(window);
            if let Some(mtm) = objc2::MainThreadMarker::new() {
                // SAFETY: pointer captured from this live window on the main thread.
                *self.chrome_material.borrow_mut() = unsafe {
                    bbr_web_macos::chrome_material::ChromeMaterial::install(mtm, pointer)
                };
            }
            if self.chrome_material.borrow().is_none() {
                log::warn!("native chrome material unavailable; retaining opaque chrome");
            }
        }
        #[cfg(not(target_os = "macos"))]
        let _ = window;
    }

    pub(super) fn glass_enabled(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            let prefs = self.effective_accessibility();
            self.chrome_material.borrow().is_some()
                && !prefs.reduce_transparency
                && !prefs.increase_contrast
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    #[cfg(feature = "testkit")]
    pub(super) fn chrome_material_snapshot(
        &self,
    ) -> Option<bbr_testkit::protocol::ChromeMaterialSnapshot> {
        #[cfg(target_os = "macos")]
        {
            let native = self.chrome_material.borrow().as_ref()?.observation();
            let web_view_opaque = self.geometry_engine.as_ref().and_then(|engine| {
                engine
                    .borrow()
                    .web_view_opaque(self.controller.read().snapshot().active_view?)
            });
            Some(bbr_testkit::protocol::ChromeMaterialSnapshot {
                attached: native.attached,
                enabled: native.enabled,
                card_mask_valid: native.card_mask_valid,
                behind_window: native.behind_window,
                follows_activation: native.follows_activation,
                window_opaque: native.window_opaque,
                size: native.size,
                tint_alpha: glass_tint(
                    self.active_chrome_tint(),
                    &self.theme(),
                    self.glass_enabled(),
                )
                .a,
                reduce_transparency: self.effective_accessibility().reduce_transparency,
                web_view_opaque,
            })
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }

    /// Draw the tint once across sidebar + chassis. The fade interpolates
    /// RGBA directly, not source-over translucent tint layers.
    pub(super) fn chrome_background(
        &self,
        content: AnyElement,
        card: bbr_ui::card::CardInsets,
        rail_width: f32,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme();
        let enabled = self.glass_enabled();
        let target = self.active_chrome_tint();
        let to = glass_tint(target, &theme, enabled);
        let presented = std::rc::Rc::new(std::cell::Cell::new(to));
        let animated_color = presented.clone();
        #[cfg(target_os = "macos")]
        let native = self.chrome_material.clone();
        let paint = move |color: Rgba| {
            #[cfg(target_os = "macos")]
            let native = native.clone();
            div().size_full().bg(color).child(
                gpui::canvas(
                    |_, _, _| {},
                    move |bounds, _, _window, _cx| {
                        let color = presented.get();
                        #[cfg(target_os = "macos")]
                        if let Some(material) = native.borrow_mut().as_mut() {
                            material.set_enabled(enabled);
                            material.update(
                                theme.mode == bbr_ui::theme::ThemeMode::Dark,
                                card.inner_rect(
                                    [bounds.size.width.into(), bounds.size.height.into()],
                                    rail_width,
                                ),
                                card.radius,
                                [color.r, color.g, color.b, color.a],
                            );
                        }
                        #[cfg(not(target_os = "macos"))]
                        let _ = (bounds, card, rail_width, color);
                    },
                )
                .absolute()
                .size_full(),
            )
        };
        let background = paint(to).child(content);
        if let Some((from, key)) = self.tint_fade() {
            if let Some(plan) = Moment::SpaceTintFade.plan(theme.motion_mode) {
                // The native corner tint is synchronized by the animation's
                // canvas, using the exact same eased color as the root.
                return background
                    .with_animation(
                        SharedString::from(format!("bbr-chrome-glass-{key}")),
                        plan.animation(),
                        move |element, t| {
                            let color = glass_tint(
                                Rgba {
                                    r: from.r + (target.r - from.r) * t,
                                    g: from.g + (target.g - from.g) * t,
                                    b: from.b + (target.b - from.b) * t,
                                    a: 1.0,
                                },
                                &theme,
                                enabled,
                            );
                            animated_color.set(color);
                            element.bg(color)
                        },
                    )
                    .into_any_element();
            }
        }
        background.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_domain::{SpaceTheme, TintIntensity};
    use bbr_ui::theme::{space_tint, Density, ThemeMode};

    fn ratio(a: Rgba, b: Rgba) -> f32 {
        let luminance = |c: Rgba| {
            let linear = |v: f32| {
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
        };
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn translucent_chrome_keeps_existing_contrast_bars_over_any_desktop() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
            for token in SpaceTheme::ALL {
                for intensity in TintIntensity::ALL {
                    let tint = space_tint(token, intensity, &theme).chrome;
                    let fill = glass_tint(tint, &theme, true);
                    assert!(fill.a < 1.0);
                    // Channel-wise extremes bound every possible desktop RGB.
                    for backdrop in [gpui::rgb(0), gpui::rgb(0xFFFFFF)] {
                        let background = bbr_ui::palette::over(fill, backdrop);
                        for (ink, minimum) in [
                            (theme.palette.ink, 12.0),
                            (theme.palette.ink_secondary, 5.9),
                            (theme.palette.accent, 3.0),
                            (
                                theme.palette.accent_ink,
                                if mode == ThemeMode::Light { 5.9 } else { 9.0 },
                            ),
                        ] {
                            assert!(
                                ratio(ink, background) >= minimum,
                                "{mode:?}/{token:?}/{intensity:?}: {} < {minimum}",
                                ratio(ink, background)
                            );
                        }
                    }
                    assert_eq!(glass_tint(tint, &theme, false), tint);
                    let high = Theme::new(mode, ContrastMode::Increased, Density::Normal);
                    assert_eq!(glass_tint(tint, &high, true).a, 1.0);
                }
            }
        }
    }
}
