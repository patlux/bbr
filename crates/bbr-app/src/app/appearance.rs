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

use bbr_ui::theme::{ContrastMode, Theme, ThemeMode};
use gpui::{Context, Window, WindowAppearance};

use super::BbrApp;

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
    #[cfg(feature = "testkit")] override_mode: Option<ThemeMode>,
) -> ThemeMode {
    #[cfg(feature = "testkit")]
    {
        override_mode.unwrap_or(system)
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
        let browser = self.controller.snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = browser.tabs.len().saturating_sub(pinned);
        Theme::for_collection(self.effective_theme_mode(), ContrastMode::Standard, regular)
    }

    /// The effective production `ThemeMode`: a testkit appearance override
    /// (when present) wins over the observed macOS system appearance
    /// (task packet `system-appearance-theme`).
    pub fn effective_theme_mode(&self) -> ThemeMode {
        #[cfg(feature = "testkit")]
        {
            effective_theme_mode(self.system_appearance, self.appearance_override)
        }
        #[cfg(not(feature = "testkit"))]
        {
            effective_theme_mode(self.system_appearance, None)
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
        self.appearance_override = mode;
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
