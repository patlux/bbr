//! Production theme-selection authority (task packet
//! `system-appearance-theme`).
//!
//! The shell derives its `ThemeMode` from the macOS system appearance —
//! exposed through GPUI's public [`gpui::WindowAppearance`] and
//! [`gpui::Window::appearance`] — through exactly one mapping in this
//! module. There is no second theme source: `BbrApp` stores the last
//! observed system appearance (updated live by the
//! [`gpui::Window::observe_window_appearance`] observer) and the render +
//! snapshot resolve the effective mode from it. Under the `testkit`
//! feature a deterministic appearance override wins over the observed
//! system appearance so real-app E2E can force light and dark regardless
//! of the host.
//!
//! A live appearance flip is a state change: the observer stores the new
//! mode and notifies, so the next presentation commit renders the complete
//! other token set — `dur:instant`, no cross-fade, no intermediate
//! mixed-token frame (DESIGN.md §6 commit-first).

use bbr_ui::theme::{ContrastMode, MotionMode, Theme, ThemeMode};
use gpui::{Context, Window, WindowAppearance};

use super::BbrApp;

/// The observed macOS accessibility display options the shell honours
/// (task packet `v2-system-prefs`): Increase Contrast selects the §3.4
/// token set, Reduce Motion the §6 reduced variants. Portable plain value;
/// the macOS adapter reads and observes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AccessibilityPrefs {
    pub(crate) increase_contrast: bool,
    pub(crate) reduce_motion: bool,
}

/// Testkit-only per-option override: `None` follows the system value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AccessibilityOverride {
    pub(crate) increase_contrast: Option<bool>,
    pub(crate) reduce_motion: Option<bool>,
}

/// The effective accessibility options: each overridden option wins over
/// the observed system value (overrides exist only under `testkit`). Pure
/// so the priority is unit-testable without a display server.
pub(crate) fn effective_accessibility(
    system: AccessibilityPrefs,
    _override: AccessibilityOverride,
) -> AccessibilityPrefs {
    #[cfg(feature = "testkit")]
    {
        AccessibilityPrefs {
            increase_contrast: _override
                .increase_contrast
                .unwrap_or(system.increase_contrast),
            reduce_motion: _override.reduce_motion.unwrap_or(system.reduce_motion),
        }
    }
    #[cfg(not(feature = "testkit"))]
    {
        system
    }
}

/// Map the effective accessibility options to the theme's contrast and
/// motion modes (the one mapping; DESIGN.md §3.4, §6).
pub(crate) fn theme_modes_for(prefs: AccessibilityPrefs) -> (ContrastMode, MotionMode) {
    let contrast = if prefs.increase_contrast {
        ContrastMode::Increased
    } else {
        ContrastMode::Standard
    };
    let motion = if prefs.reduce_motion {
        MotionMode::Reduced
    } else {
        MotionMode::Standard
    };
    (contrast, motion)
}

/// Map a GPUI window appearance to the production theme mode (DESIGN.md
/// §1): a dark system appearance presents the complete dark token set,
/// anything else presents the reference-calibrated light rail.
pub fn theme_mode_for_appearance(appearance: WindowAppearance) -> ThemeMode {
    match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeMode::Dark,
        WindowAppearance::Light | WindowAppearance::VibrantLight => ThemeMode::Light,
    }
}

/// The effective production mode: a testkit appearance override (when
/// present) wins over the observed system appearance. Portable and pure so
/// the priority is unit-testable without a display server.
pub(crate) fn effective_theme_mode(
    system: ThemeMode,
    _override_mode: Option<ThemeMode>,
) -> ThemeMode {
    #[cfg(feature = "testkit")]
    {
        _override_mode.unwrap_or(system)
    }
    #[cfg(not(feature = "testkit"))]
    {
        system
    }
}

impl BbrApp {
    /// The resolved theme tokens for the current shell mode. Density is
    /// automatic from the regular-tab collection size (DESIGN.md §8); the
    /// §15 mode fields are already reported by the snapshot. The mode
    /// comes from the one production theme-selection authority
    /// (system appearance, testkit-overridable) — never a hardcoded
    /// value (task packet `system-appearance-theme`).
    pub fn theme(&self) -> Theme {
        use crate::preferences::DensityChoice;
        use bbr_ui::theme::Density;
        let browser = self.controller.read().snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = browser.tabs.len().saturating_sub(pinned);
        let (contrast, motion) = theme_modes_for(self.effective_accessibility());
        let theme = Theme::for_collection(self.effective_theme_mode(), contrast, regular);
        // Blueprint D16: a density pinned in Settings wins over the
        // collection-size choice (DESIGN §8).
        let density = self.preferences.read().density;
        let theme = match density {
            DensityChoice::Automatic => theme,
            DensityChoice::Small => Theme::new(theme.mode, contrast, Density::Small),
            DensityChoice::Normal => Theme::new(theme.mode, contrast, Density::Normal),
            DensityChoice::Large => Theme::new(theme.mode, contrast, Density::ExtraLarge),
        };
        theme.with_motion_mode(motion)
    }

    /// The effective accessibility display options: a testkit override
    /// per option wins over the observed macOS value (task packet
    /// `v2-system-prefs`).
    pub(crate) fn effective_accessibility(&self) -> AccessibilityPrefs {
        // Blueprint D16: the Settings choice (follow macOS, on, off) applies
        // to the system value first; a testkit override still wins.
        let system = AccessibilityPrefs {
            increase_contrast: self
                .preferences
                .read()
                .increase_contrast
                .resolve(self.system_accessibility.read().increase_contrast),
            reduce_motion: self
                .preferences
                .read()
                .reduce_motion
                .resolve(self.system_accessibility.read().reduce_motion),
        };
        #[cfg(feature = "testkit")]
        {
            effective_accessibility(system, self.accessibility_override.value())
        }
        #[cfg(not(feature = "testkit"))]
        {
            effective_accessibility(system, AccessibilityOverride::default())
        }
    }

    /// Register the macOS accessibility display observer (composition root,
    /// before the first presented frame): records Increase Contrast and
    /// Reduce Motion, then keeps them live through the workspace change
    /// notification — no polling, so an idle shell never wakes for them. A
    /// change is a state change: the next presentation commit renders the
    /// complete other token set (`dur:instant`, DESIGN.md §6).
    pub fn observe_system_accessibility(&mut self, cx: &mut Context<Self>) {
        #[cfg(target_os = "macos")]
        {
            use bbr_web_macos::accessibility_display as display;
            let Some(mtm) = objc2::MainThreadMarker::new() else {
                return;
            };
            self.system_accessibility
                .set(prefs_from(display::current_options()));
            let (tx, rx) = smol::channel::unbounded::<()>();
            self.accessibility_observer = Some(display::AccessibilityDisplayObserver::install(
                mtm,
                Box::new(move || {
                    let _ = tx.try_send(());
                }),
            ));
            self.accessibility_task = Some(cx.spawn(async move |this, cx| {
                while rx.recv().await.is_ok() {
                    let prefs = prefs_from(display::current_options());
                    let updated = this.update(cx, |view, cx| {
                        view.set_system_accessibility(prefs, cx);
                    });
                    if updated.is_err() {
                        break;
                    }
                }
            }));
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = cx;
        }
    }

    /// Live accessibility-change sink: store the new options and re-present.
    pub(crate) fn set_system_accessibility(
        &mut self,
        prefs: AccessibilityPrefs,
        cx: &mut Context<Self>,
    ) {
        if self.system_accessibility.value() != prefs {
            self.system_accessibility.set(prefs);
            cx.notify();
        }
    }

    /// Testkit seam: force Increase Contrast and/or Reduce Motion regardless
    /// of the host (`None` returns that option to the observed system value).
    /// The §15 `contrast` and `reduced_motion` snapshot fields report the
    /// effective values.
    #[cfg(feature = "testkit")]
    pub fn set_accessibility_override(
        &mut self,
        increase_contrast: Option<bool>,
        reduce_motion: Option<bool>,
        cx: &mut Context<Self>,
    ) {
        self.accessibility_override.set(AccessibilityOverride {
            increase_contrast,
            reduce_motion,
        });
        cx.notify();
    }

    /// The effective production `ThemeMode`: a testkit appearance override
    /// (when present) wins over the observed macOS system appearance
    /// (task packet `system-appearance-theme`).
    pub fn effective_theme_mode(&self) -> ThemeMode {
        // Blueprint D16: Settings › Appearance (System follows macOS).
        let appearance = self.preferences.read().appearance;
        let system = match appearance {
            crate::preferences::AppearanceChoice::System => self.system_appearance,
            crate::preferences::AppearanceChoice::Light => ThemeMode::Light,
            crate::preferences::AppearanceChoice::Dark => ThemeMode::Dark,
        };
        #[cfg(feature = "testkit")]
        {
            effective_theme_mode(system, self.appearance_override.value())
        }
        #[cfg(not(feature = "testkit"))]
        {
            effective_theme_mode(system, None)
        }
    }

    /// Register the production system-appearance observer (composition
    /// root, before the first presented frame): records the current
    /// appearance, then keeps the stored mode live — a system appearance
    /// flip re-presents the complete other token set on the next
    /// presentation commit (`dur:instant`, commit-first, DESIGN.md §6).
    pub fn observe_system_appearance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.system_appearance = theme_mode_for_appearance(window.appearance());
        let app = cx.entity();
        self.appearance_subscription = Some(window.observe_window_appearance(move |window, cx| {
            app.update(cx, |view, cx| {
                view.set_system_appearance(theme_mode_for_appearance(window.appearance()), cx);
            });
        }));
    }

    /// Live appearance-change sink (the GPUI observer callback): stores the
    /// newly observed mode and marks the shell dirty so the next frame
    /// presents the complete token set with no cross-fade and no
    /// intermediate mixed-token frame (task packet `system-appearance-theme`).
    fn set_system_appearance(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.system_appearance = mode;
        cx.notify();
    }

    /// Testkit seam: force the effective production mode regardless of the
    /// host appearance (`None` clears the override and returns to the
    /// observed system appearance). The §15 `theme` snapshot field reports
    /// the effective mode (task packet `system-appearance-theme`).
    #[cfg(feature = "testkit")]
    pub fn set_appearance_override(&mut self, mode: Option<ThemeMode>, cx: &mut Context<Self>) {
        self.appearance_override.set(mode);
        cx.notify();
    }
}

#[cfg(target_os = "macos")]
fn prefs_from(
    options: bbr_web_macos::accessibility_display::AccessibilityDisplayOptions,
) -> AccessibilityPrefs {
    AccessibilityPrefs {
        increase_contrast: options.increase_contrast,
        reduce_motion: options.reduce_motion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessibility_prefs_map_to_theme_modes() {
        let both_off = AccessibilityPrefs::default();
        assert_eq!(
            theme_modes_for(both_off),
            (ContrastMode::Standard, MotionMode::Standard)
        );
        let contrast = AccessibilityPrefs {
            increase_contrast: true,
            reduce_motion: false,
        };
        assert_eq!(
            theme_modes_for(contrast),
            (ContrastMode::Increased, MotionMode::Standard)
        );
        let motion = AccessibilityPrefs {
            increase_contrast: false,
            reduce_motion: true,
        };
        assert_eq!(
            theme_modes_for(motion),
            (ContrastMode::Standard, MotionMode::Reduced)
        );
    }

    #[test]
    fn accessibility_override_wins_per_option() {
        let system = AccessibilityPrefs {
            increase_contrast: true,
            reduce_motion: false,
        };
        // No override: the system decides.
        assert_eq!(
            effective_accessibility(system, AccessibilityOverride::default()),
            system
        );
        #[cfg(feature = "testkit")]
        {
            // Each option is overridden independently; `None` keeps the
            // system value for that option.
            let forced = effective_accessibility(
                system,
                AccessibilityOverride {
                    increase_contrast: None,
                    reduce_motion: Some(true),
                },
            );
            assert_eq!(
                forced,
                AccessibilityPrefs {
                    increase_contrast: true,
                    reduce_motion: true,
                }
            );
            let cleared = effective_accessibility(
                system,
                AccessibilityOverride {
                    increase_contrast: Some(false),
                    reduce_motion: None,
                },
            );
            assert!(!cleared.increase_contrast);
            assert!(!cleared.reduce_motion);
        }
    }

    #[test]
    fn appearance_maps_deterministically_to_theme_mode() {
        use gpui::WindowAppearance as A;
        // The production mapping: dark (and vibrant dark) → Dark; light
        // (and vibrant light) → Light. Every GPUI appearance value is
        // covered so the authority never falls through.
        assert_eq!(
            theme_mode_for_appearance(A::Dark),
            ThemeMode::Dark,
            "dark system → dark token set"
        );
        assert_eq!(
            theme_mode_for_appearance(A::VibrantDark),
            ThemeMode::Dark,
            "vibrant dark is a dark system"
        );
        assert_eq!(
            theme_mode_for_appearance(A::Light),
            ThemeMode::Light,
            "light system → calibrated light rail"
        );
        assert_eq!(
            theme_mode_for_appearance(A::VibrantLight),
            ThemeMode::Light,
            "vibrant light is a light system"
        );
    }

    #[test]
    fn effective_mode_prefers_the_override_then_the_system() {
        // No override: the observed system appearance decides.
        assert_eq!(
            effective_theme_mode(ThemeMode::Light, None),
            ThemeMode::Light
        );
        assert_eq!(effective_theme_mode(ThemeMode::Dark, None), ThemeMode::Dark);
        // An override wins regardless of the host appearance (E2E forces
        // light and dark deterministically).
        #[cfg(feature = "testkit")]
        {
            assert_eq!(
                effective_theme_mode(ThemeMode::Light, Some(ThemeMode::Dark)),
                ThemeMode::Dark
            );
            assert_eq!(
                effective_theme_mode(ThemeMode::Dark, Some(ThemeMode::Light)),
                ThemeMode::Light
            );
        }
    }
}
