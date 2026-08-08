//! Downloads row detail (DESIGN.md §11.12): the secondary line and the
//! single contextual row action.
//!
//! Split out of `downloads_render` to keep both files within the ~500-line
//! structure gate. These are presentation only — the machine state and the
//! offered action are decided by the controller and arrive as a
//! `DownloadRowVisual`; nothing here derives or changes a transition.

use gpui::{div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use crate::theme::{Spacing, Theme};

use super::super::downloads::action_label;
use super::super::BbrApp;
use super::download_slot_id;

impl BbrApp {
    /// The §11.12 secondary line (`type:meta`, `tnum`): active rows — the
    /// 2 pt progress bar (track `hairline`, fill `color:accent`; a known
    /// total fills `fraction()`, an unknown length runs the §10.5
    /// indeterminate sweep with transferred bytes only) plus the
    /// transferred/total bytes; terminal rows — size + status copy naming
    /// the machine outcome (§11.8 tone) beside the bar (a succeeded row
    /// presents the full bar). The secondary line carries the
    /// `bbr-download-meta-<n>` id; the bar inside carries
    /// `bbr-download-progress-<n>`.
    pub(super) fn downloads_secondary_line(
        &self,
        theme: &Theme,
        visual: &super::super::downloads::DownloadRowVisual,
    ) -> Stateful<Div> {
        let p = theme.palette;
        let bar = match visual.progress_fraction {
            Some(fraction) => {
                // Known total: the track fills `fraction()` (a zero-byte
                // download reports `1.0` and renders a full bar). The fill
                // width is the `DefiniteLength::Fraction` of the track's
                // own width — no pixel size is invented.
                let fraction = fraction.clamp(0.0, 1.0) as f32;
                div()
                    .id(download_slot_id("progress", visual.index))
                    .flex_1()
                    .min_w_0()
                    .h(px(2.0))
                    .bg(p.hairline)
                    .child(
                        div()
                            .h_full()
                            .w(gpui::DefiniteLength::Fraction(fraction))
                            .bg(p.accent),
                    )
            }
            None => {
                // Unknown length: the §10.5 indeterminate sweep (a fixed
                // leading sweep segment in the v1 render; the sweep loop is
                // transient indication and not part of the asserted state).
                div()
                    .id(download_slot_id("progress", visual.index))
                    .flex_1()
                    .min_w_0()
                    .h(px(2.0))
                    .bg(p.hairline)
                    .child(
                        div()
                            .h_full()
                            .w(gpui::DefiniteLength::Fraction(0.30))
                            .bg(p.accent),
                    )
            }
        };
        let meta_text = div()
            .text_size(theme.type_.meta.size)
            .font_weight(theme.type_.meta.weight)
            .text_color(p.ink_secondary)
            .child(visual.meta.clone());
        // The §15 row id set (`-status`/`-name`/`-meta`/`-progress`/`-action`)
        // applies to every row, so the secondary line always carries the
        // progress slot — History rows included (a succeeded row renders the
        // full bar, matching the `bbr-download-progress-<n>` id the visible
        // ids derive for every rendered row).
        div()
            .id(download_slot_id("meta", visual.index))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::sp2())
            .child(bar)
            .child(meta_text)
    }

    /// The exactly-one contextual primary action per row (DESIGN.md §11.12):
    /// active → **Cancel**, failed → **Retry**, succeeded →
    /// **Reveal-in-Finder** (the conventional macOS "Show in Finder" copy;
    /// §10.8 `disabled` with a reason when the resolved path does not
    /// exist), cancelled → none in v1 (§17.8 open). The hit area is at
    /// least `density:hit-min`.
    pub(super) fn downloads_action(
        &self,
        theme: &Theme,
        visual: &super::super::downloads::DownloadRowVisual,
        index: usize,
        activate: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Stateful<Div> {
        let p = theme.palette;
        let density = theme.density;
        let label = action_label(visual.action);
        if label.is_empty() {
            // Cancelled rows render no primary action in v1 (§17.8 open) —
            // the trailing slot stays quiet.
            return div()
                .id(download_slot_id("action", index))
                .w(density.hit_min());
        }
        let enabled = visual.action_enabled;
        div()
            .id(download_slot_id("action", index))
            .h(density.hit_min())
            .min_w(density.hit_min())
            .px(Spacing::sp2())
            .rounded(density.density_radius())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .text_size(theme.type_.meta.size)
            .font_weight(theme.type_.meta.weight)
            .when(enabled, |el| {
                el.text_color(p.accent_ink).hover(|el| el.bg(p.wash_hover))
            })
            .when(!enabled, |el| el.text_color(p.ink_faint))
            // A disabled action (a vanished reveal target) is a no-op; the
            // reason rides the semantic tree / snapshot (§10.8; the tooltip
            // presentation is deferred with §17.2).
            .when(enabled, |el| el.on_mouse_down(MouseButton::Left, activate))
            .child(label)
    }
}
