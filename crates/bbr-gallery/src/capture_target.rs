//! Capture-only observation of the existing Checkbox child; never input or paint proof.
use gpui::{Bounds, Div, Pixels};

use crate::capture_page::{PageCapture, HEADER, HEIGHT, INSET, WIDTH};

/// Direct Checkbox story child: heading, control, heading, value, source.
pub const CHECKBOX_CHILD: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawToken(u64);

#[derive(Default)]
pub struct Target {
    generation: u64,
    active: Option<DrawToken>,
    observation: Option<Result<Bounds<Pixels>, String>>,
}
impl Target {
    pub fn invalidate(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("draw generation exhausted");
        self.active = None;
        self.observation = None;
    }
    pub fn begin_draw(&mut self) -> DrawToken {
        self.invalidate();
        let token = DrawToken(self.generation);
        self.active = Some(token);
        token
    }
    pub fn token(&self) -> Option<DrawToken> {
        self.active
    }

    pub fn observe(
        &mut self,
        token: Option<DrawToken>,
        children: Vec<Bounds<Pixels>>,
        clip: Bounds<Pixels>,
    ) {
        // Never let a later good callback overwrite ambiguity or a bad result.
        if self.observation.is_some() {
            self.observation = Some(Err("duplicate Checkbox target observation".into()));
        } else if token.is_none() || token != self.active {
            self.observation = Some(Err("stale/unarmed Checkbox target observation".into()));
        } else {
            self.observation = Some(validate(&children, clip));
        }
    }

    pub fn resolve(
        &self,
        token: DrawToken,
        expected: bool,
    ) -> Result<Option<Bounds<Pixels>>, String> {
        if self.active != Some(token) {
            return Err("stale Checkbox target draw".into());
        }
        match (&self.observation, expected) {
            (Some(Err(error)), _) => Err(error.clone()),
            (None, true) => Err("missing Checkbox target observation".into()),
            (Some(Ok(_)), false) => Err("unexpected Checkbox target on page".into()),
            (Some(Ok(bounds)), true) => Ok(Some(*bounds)),
            (None, false) => Ok(None),
        }
    }
}

fn edges(bounds: Bounds<Pixels>) -> Result<[f32; 4], String> {
    let [x, y, w, h] = [
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
    ]
    .map(f32::from);
    let right = x + w;
    let bottom = y + h;
    if ![x, y, w, h, right, bottom].iter().all(|n| n.is_finite()) || w <= 0.0 || h <= 0.0 {
        return Err("nonfinite/nonpositive Checkbox target or clip".into());
    }
    Ok([x, y, right, bottom])
}
fn validate(children: &[Bounds<Pixels>], clip: Bounds<Pixels>) -> Result<Bounds<Pixels>, String> {
    let [child] = children else {
        return Err("Checkbox target must have exactly one child".into());
    };
    let [x, y, right, bottom] = edges(*child)?;
    let [left, top, clip_right, clip_bottom] = edges(clip)?;
    if x < left
        || y < top
        || right > clip_right
        || bottom > clip_bottom
        || x < INSET
        || y < INSET + HEADER
        || right > WIDTH - INSET
        || bottom > HEIGHT - INSET
    {
        return Err("Checkbox target is clipped or outside capture frame".into());
    }
    Ok(*child)
}

/// Decorate the existing hug-child Div. No new wrapper, hitbox, input listener or style.
/// GPUI supplies its child's bounds and current ancestor-mask intersection after
/// child prepaint. This is not the inner hitbox, full clip ancestry or painted extent.
pub fn observe_checkbox(slot: Div, capture: &PageCapture) -> Div {
    let token = capture.0.borrow().checkbox.token();
    let capture = capture.clone();
    slot.on_children_prepainted(move |children, window, _| {
        capture
            .0
            .borrow_mut()
            .checkbox
            .observe(token, children, window.content_mask().bounds);
    })
}

#[cfg(test)]
#[path = "capture_target_tests.rs"]
mod tests;
