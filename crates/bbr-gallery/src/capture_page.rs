//! Gallery-only framing. Never changes a production component's anatomy.
//! Each direct story child is indivisible; pagination follows measured layout,
//! not pixel polling. A child that cannot fit fails rather than being clipped.
use std::{cell::RefCell, ops::Range, rc::Rc, time::Instant};

use gpui::{div, prelude::*, px, AnyElement, Bounds, Pixels};

use crate::registry::StoryContext;

pub const WIDTH: f32 = 1200.0;
pub const HEIGHT: f32 = 1000.0;
pub const INSET: f32 = 48.0;
pub const GAP: f32 = 8.0;
pub const HEADER: f32 = 48.0;

#[derive(Clone, Default)]
pub struct PageCapture(pub Rc<RefCell<PageState>>);

#[derive(Default)]
pub struct PageState {
    pub range: Option<Range<usize>>,
    pub total: usize,
    pub bounds: Vec<Bounds<Pixels>>,
    pub first_prepaint: Option<Instant>,
    pub last_layout_start: Option<Instant>,
}

impl PageState {
    pub fn is_settled(&self, duration: std::time::Duration) -> bool {
        if self.bounds.is_empty() {
            return false;
        }
        if duration.is_zero() {
            return true;
        }
        match (self.first_prepaint, self.last_layout_start) {
            (Some(first), Some(last)) => last.saturating_duration_since(first) > duration,
            _ => false,
        }
    }
}

impl PageCapture {
    pub fn reset(&self, range: Option<Range<usize>>) {
        *self.0.borrow_mut() = PageState {
            range,
            ..Default::default()
        };
    }
}

pub fn render(ctx: &StoryContext, title: &str, children: Vec<AnyElement>) -> AnyElement {
    let capture = ctx.capture.expect("capture context").clone();
    let total = children.len();
    let range = capture.0.borrow().range.clone().unwrap_or(0..total);
    assert!(
        range.start < range.end && range.end <= total,
        "empty/invalid page"
    );
    let theme = ctx.theme;
    let selected = children
        .into_iter()
        .enumerate()
        .filter(|(i, _)| range.contains(i))
        .map(|(_, child)| div().flex_none().child(child));
    let observer = capture.clone();
    // This precedes child request_layout (where GPUI samples animations).
    // Prepaint alone is too late to prove that the last layout was settled.
    let layout_start = Instant::now();
    div()
        .id("bbr-gallery-capture-page")
        .size_full()
        .flex()
        .flex_col()
        .p(px(INSET))
        .bg(theme.palette.surface_canvas)
        .text_color(theme.palette.ink)
        .child(
            div()
                .h(px(HEADER))
                .flex_none()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .child(title.to_owned()),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_none()
                .gap(px(GAP))
                .on_children_prepainted(move |bounds, _, _| {
                    let mut state = observer.0.borrow_mut();
                    state.total = total;
                    state.bounds = bounds;
                    let now = Instant::now();
                    state.first_prepaint.get_or_insert(now);
                    state.last_layout_start = Some(layout_start);
                })
                .children(selected),
        )
        .into_any()
}

/// Greedy consecutive groups, never splitting or dropping a story child.
pub fn paginate(heights: &[f32]) -> Result<Vec<Range<usize>>, String> {
    if heights.is_empty() {
        return Err("story contains no children".into());
    }
    let available = HEIGHT - 2.0 * INSET - HEADER;
    let mut pages = Vec::new();
    let mut start = 0;
    let mut used = 0.0;
    for (i, height) in heights.iter().copied().enumerate() {
        if !height.is_finite() || height < 0.0 || height > available {
            return Err(format!(
                "child {i} height {height} exceeds framing capacity {available}"
            ));
        }
        let gap = if i > start { GAP } else { 0.0 };
        if used + gap + height > available {
            pages.push(start..i);
            start = i;
            used = height;
        } else {
            used += gap + height;
        }
    }
    pages.push(start..heights.len());
    Ok(pages)
}

pub fn validate_frame(state: &PageState, range: &Range<usize>) -> Result<(), String> {
    if state.range.as_ref() != Some(range) || state.bounds.len() != range.len() {
        return Err("rendered page does not match requested child range".into());
    }
    for (index, bounds) in state.bounds.iter().enumerate() {
        let x = f32::from(bounds.origin.x);
        let y = f32::from(bounds.origin.y);
        let right = x + f32::from(bounds.size.width);
        let bottom = y + f32::from(bounds.size.height);
        if ![x, y, right, bottom].iter().all(|v| v.is_finite())
            || x < INSET
            || y < INSET + HEADER
            || right > WIDTH - INSET
            || bottom > HEIGHT - INSET
        {
            return Err(format!(
                "child {} outside complete frame: {bounds:?}",
                range.start + index
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_requires_layout_after_production_duration() {
        let first = Instant::now();
        let duration = std::time::Duration::from_millis(90);
        let mut state = PageState {
            first_prepaint: Some(first),
            last_layout_start: Some(first - duration),
            bounds: vec![Bounds::default()],
            ..Default::default()
        };
        assert!(!state.is_settled(duration));
        state.last_layout_start = Some(first + duration);
        assert!(!state.is_settled(duration));
        state.last_layout_start = Some(first + duration + std::time::Duration::from_nanos(1));
        assert!(state.is_settled(duration));
        state.bounds.clear();
        assert!(!state.is_settled(std::time::Duration::ZERO));
    }

    #[test]
    fn pages_cover_every_child_once_without_splitting() {
        assert_eq!(
            paginate(&[100.0, 700.0, 100.0, 600.0]).unwrap(),
            vec![0..2, 2..4]
        );
        assert_eq!(paginate(&[856.0, 1.0]).unwrap(), vec![0..1, 1..2]);
    }

    #[test]
    fn oversized_or_invalid_children_fail_closed() {
        for heights in [vec![], vec![857.0], vec![f32::NAN], vec![-1.0]] {
            assert!(paginate(&heights).is_err());
        }
    }

    #[test]
    fn clipped_or_missing_children_fail_closed() {
        let mut state = PageState {
            range: Some(0..1),
            total: 1,
            bounds: vec![Bounds::new(
                gpui::point(px(INSET), px(INSET + HEADER)),
                gpui::size(px(300.0), px(100.0)),
            )],
            ..Default::default()
        };
        assert!(validate_frame(&state, &(0..1)).is_ok());
        state.bounds[0].size.height = px(1200.0);
        assert!(validate_frame(&state, &(0..1)).is_err());
        state.bounds.clear();
        assert!(validate_frame(&state, &(0..1)).is_err());
    }
}
