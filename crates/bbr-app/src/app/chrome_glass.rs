//! One chrome background per browser window; solid controls and content are
//! above it. Never change the component theme or the native web views.

use super::BbrApp;
use bbr_ui::theme::{motion::Moment, ContrastMode, Theme};
use gpui::{div, prelude::*, AnimationExt, AnyElement, Context, Rgba, SharedString, Window};

/// Coverage of the space tint over AppKit's Sidebar material. The rest of
/// the frosted desktop stays visible: the original near-opaque compensation
/// (91.6–95.5 %) left nothing to see. These are the smallest coverages that
/// keep primary labels (`ink`) at WCAG AA 4.5:1 for every space tint even
/// over a raw black or white desktop, i.e. before the material's own frosting
/// moves the backdrop towards the rail. Secondary text and accents rely on
/// that frosting; Increase Contrast and Reduce Transparency restore the
/// opaque rail with its full D3 floors.
const LIGHT_TINT_ALPHA: f32 = 0.55;
const DARK_TINT_ALPHA: f32 = 0.66;

/// The tint keeps its own hue and luminance; over the similarly toned native
/// material it reads as the original rail with the desktop blurred into it.
pub(super) fn glass_tint(mut color: Rgba, theme: &Theme, enabled: bool) -> Rgba {
    if enabled && theme.contrast != ContrastMode::Increased {
        color.a = match theme.mode {
            bbr_ui::theme::ThemeMode::Light => LIGHT_TINT_ALPHA,
            bbr_ui::theme::ThemeMode::Dark => DARK_TINT_ALPHA,
        };
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

    fn every_tint(mut check: impl FnMut(ThemeMode, &Theme, SpaceTheme, TintIntensity, Rgba)) {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
            for token in SpaceTheme::ALL {
                for intensity in TintIntensity::ALL {
                    let tint = space_tint(token, intensity, &theme).chrome;
                    check(mode, &theme, token, intensity, tint);
                }
            }
        }
    }

    /// Regression for "ich seh davon nichts": 91.6–95.5 % coverage hid the
    /// native material. At least a third of it must remain visible.
    #[test]
    fn glass_tint_leaves_the_native_material_visible() {
        every_tint(|mode, theme, token, intensity, tint| {
            let fill = glass_tint(tint, theme, true);
            assert!(
                fill.a <= 0.67,
                "{mode:?}/{token:?}/{intensity:?}: {} hides the material",
                fill.a
            );
            assert_eq!([fill.r, fill.g, fill.b], [tint.r, tint.g, tint.b]);
        });
    }

    #[test]
    fn primary_labels_stay_legible_over_any_raw_desktop() {
        every_tint(|mode, theme, token, intensity, tint| {
            let fill = glass_tint(tint, theme, true);
            // Channel-wise extremes bound every possible desktop RGB; the
            // native frosting only moves the backdrop towards the rail.
            for backdrop in [gpui::rgb(0), gpui::rgb(0xFFFFFF)] {
                let background = bbr_ui::palette::over(fill, backdrop);
                let contrast = ratio(theme.palette.ink, background);
                assert!(
                    contrast >= 4.5,
                    "{mode:?}/{token:?}/{intensity:?}: {contrast} < 4.5"
                );
            }
        });
        // The coverage is the smallest step that holds the bar for every
        // tint, not an arbitrary opacity: one percent less fails somewhere.
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
            let fails_thinner = SpaceTheme::ALL.iter().any(|&token| {
                TintIntensity::ALL.iter().any(|&intensity| {
                    let tint = space_tint(token, intensity, &theme).chrome;
                    let fill = glass_tint(tint, &theme, true);
                    let thinner = Rgba {
                        a: fill.a - 0.01,
                        ..fill
                    };
                    [gpui::rgb(0), gpui::rgb(0xFFFFFF)]
                        .into_iter()
                        .any(|backdrop| {
                            ratio(theme.palette.ink, bbr_ui::palette::over(thinner, backdrop)) < 4.5
                        })
                })
            });
            assert!(fails_thinner, "{mode:?}: coverage is larger than needed");
        }
    }

    #[test]
    fn accessibility_and_unsupported_modes_restore_the_opaque_rail() {
        every_tint(|mode, theme, _, _, tint| {
            assert_eq!(glass_tint(tint, theme, false), tint);
            let high = Theme::new(mode, ContrastMode::Increased, Density::Normal);
            let high_tint = Rgba { a: 1.0, ..tint };
            assert_eq!(glass_tint(high_tint, &high, true).a, 1.0);
        });
    }
}
