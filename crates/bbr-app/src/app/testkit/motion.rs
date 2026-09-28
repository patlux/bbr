//! §6/§18.5 motion evidence: the transient moments the render presents,
//! resolved through the same `bbr_ui::theme::motion` plans and the same
//! row states / address progress the render paints with, plus the root
//! render counter for the idle proof.

use bbr_testkit::protocol::{MotionMomentSnapshot, MotionSnapshot};
use bbr_ui::components::{AddressProgress, RowVisuals};
use bbr_ui::state::TabVisualState;
use bbr_ui::theme::motion::Moment;

use crate::app::BbrApp;

impl BbrApp {
    /// The typed motion evidence of the main window.
    pub(super) fn motion_snapshot(&self, rows: &[RowVisuals]) -> MotionSnapshot {
        let mode = self.theme().motion_mode;
        let dense = self.dense_window.get();
        let pinned = crate::shell::pinned_count(&self.controller.snapshot().tabs);
        let rendered = |index: usize| {
            index < pinned
                || (dense.first_rendered..dense.first_rendered + dense.rendered_rows)
                    .contains(&index)
        };
        let moment = |moment: Moment, target: String| MotionMomentSnapshot {
            moment: moment.name().to_string(),
            target,
            variant: moment.variant(mode).to_string(),
        };
        let mut moments = Vec::new();
        if self.sidebar_visible {
            for row in rows.iter().filter(|row| rendered(row.index)) {
                match row.state {
                    TabVisualState::Loading => moments.push(moment(
                        Moment::LoadingArc,
                        format!("bbr-tab-favicon-{}", row.index),
                    )),
                    TabVisualState::Attention => moments.push(moment(
                        Moment::AttentionPulse,
                        format!("bbr-tab-state-{}", row.index),
                    )),
                    _ => {}
                }
            }
        }
        let target = bbr_ui::components::address::ADDRESS_PROGRESS_ID.to_string();
        match self.rendered_address_progress() {
            AddressProgress::Loading => moments.push(moment(Moment::AddressProgress, target)),
            AddressProgress::Finishing => {
                moments.push(moment(Moment::AddressProgressFinish, target))
            }
            AddressProgress::Idle => {}
        }
        MotionSnapshot {
            moments,
            root_renders: self.motion.root_renders,
            animated_elements: bbr_ui::theme::motion::animated_elements_built(),
        }
    }
}
