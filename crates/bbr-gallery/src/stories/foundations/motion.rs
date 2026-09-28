//! The v2 Motion Foundations page (DESIGN §6, §18.5): the duration and
//! easing tokens, every transient moment with its standard plan and its
//! Reduce Motion variant — all read from `bbr_ui::theme::motion`, the one
//! timing authority — and the D10 pane illustrations played live from the
//! production `components::illustration`. Nothing is copied or restyled.

use bbr_ui::components::illustration::{illustration, Illustration};
use bbr_ui::theme::motion::{Easing, Moment, MomentPlan};
use bbr_ui::theme::{Motion, MotionMode};
use gpui::{div, prelude::*, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The §6 duration tokens, read from `bbr_ui::theme::Motion`.
fn durations() -> [(&'static str, std::time::Duration, &'static str); 6] {
    [
        ("dur:instant", Motion::instant(), "state changes snap"),
        ("dur:micro", Motion::micro(), "hover, cursor glide"),
        ("dur:fine", Motion::fine(), "fades, reduced flashes"),
        ("dur:standard", Motion::standard(), "tint cross-fade, fill"),
        ("dur:emphasis", Motion::emphasis(), "single-shot ceiling"),
        ("load:indeterminate", Motion::load_loop(), "progress sweep"),
    ]
}

fn easing_name(easing: Easing) -> &'static str {
    match easing {
        Easing::Linear => "linear",
        Easing::Standard => "ease:standard",
        Easing::Accelerate => "ease:accelerate",
        Easing::Decelerate => "ease:decelerate",
    }
}

fn plan_text(plan: Option<MomentPlan>) -> String {
    match plan {
        None => "static — no frames".into(),
        Some(plan) => format!(
            "{} ms · {} · {}{}",
            plan.duration.as_millis(),
            easing_name(plan.easing),
            if plan.repeat { "loop" } else { "once" },
            if plan.reduced { " · opacity only" } else { "" },
        ),
    }
}

/// One `(moment, standard plan, reduced plan)` row per §18.5 moment.
pub(crate) fn moment_rows() -> Vec<(&'static str, String, String)> {
    Moment::ALL
        .into_iter()
        .map(|moment| {
            (
                moment.name(),
                plan_text(moment.plan(MotionMode::Standard)),
                plan_text(moment.plan(MotionMode::Reduced)),
            )
        })
        .collect()
}

/// §6/§18.5 motion tokens and moments, plus the live D10 illustrations.
pub fn render_motion(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let p = theme.palette;
    let mut children = vec![section_heading(theme, "Durations")];
    children.extend(durations().into_iter().map(|(name, duration, note)| {
        value_row(
            theme,
            name,
            &format!("{} ms · {note}", duration.as_millis()),
        )
    }));
    children.push(section_heading(theme, "Easing"));
    children.extend(
        [
            Easing::Standard,
            Easing::Accelerate,
            Easing::Decelerate,
            Easing::Linear,
        ]
        .into_iter()
        .map(|easing| {
            let (x1, y1, x2, y2) = easing.control_points();
            value_row(
                theme,
                easing_name(easing),
                &format!("cubic-bezier({x1}, {y1}, {x2}, {y2})"),
            )
        }),
    );
    children.push(section_heading(
        theme,
        "Moments (§18.5) — standard / Reduce Motion",
    ));
    for (name, standard, reduced) in moment_rows() {
        children.push(value_row(theme, name, &format!("{standard}  |  {reduced}")));
    }
    children.push(section_heading(
        theme,
        "Pane illustrations (D10) — two cycles, then rest",
    ));
    // Under the gallery's Reduce Motion control the illustrations render
    // their rest frame; otherwise each plays its two cycles once when the
    // page first draws.
    let key = (!ctx.reduced_motion).then_some(1);
    children.push(
        div()
            .flex()
            .flex_row()
            .items_end()
            .gap_6()
            .p_4()
            .rounded(theme.radius.large())
            .bg(p.surface_canvas)
            .border_1()
            .border_color(p.hairline)
            .children(
                [
                    Illustration::EmptySpace,
                    Illustration::Crashed,
                    Illustration::Offline,
                ]
                .map(|kind| {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(illustration(theme, kind, key))
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(p.ink_secondary)
                                .child(kind.name()),
                        )
                }),
            )
            .into_any(),
    );
    children.push(section_heading(theme, "Reduce Motion"));
    children.push(
        div()
            .text_size(theme.type_.meta.size)
            .text_color(p.ink_secondary)
            .child(if ctx.reduced_motion {
                "Reduce Motion is ON: every moment shows its right-hand variant; illustrations rest."
            } else {
                "Reduce Motion is OFF: moments decorate a committed state, never delay it."
            })
            .into_any(),
    );
    story_page(theme, "Motion", children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_moment_table_covers_every_moment_once_from_the_one_authority() {
        let rows = moment_rows();
        assert_eq!(rows.len(), Moment::ALL.len());
        let names: std::collections::HashSet<_> = rows.iter().map(|row| row.0).collect();
        assert_eq!(names.len(), rows.len(), "unique moment names");
        for moment in Moment::ALL {
            let row = rows
                .iter()
                .find(|row| row.0 == moment.name())
                .expect("row per moment");
            assert_eq!(row.1, plan_text(moment.plan(MotionMode::Standard)));
            assert_eq!(row.2, plan_text(moment.plan(MotionMode::Reduced)));
        }
        // The Reduce Motion column never loops (loops are static there).
        assert!(rows.iter().all(|row| !row.2.contains("loop")));
    }

    #[test]
    fn durations_read_the_motion_tokens() {
        let table = durations();
        assert_eq!(table[0].1, Motion::instant());
        assert_eq!(table[4].1, Motion::emphasis());
        assert_eq!(table[5].1, Motion::load_loop());
    }
}
