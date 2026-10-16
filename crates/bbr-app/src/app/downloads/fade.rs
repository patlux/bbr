//! Persistent interpolation state with a fresh GPUI cache for each panel draw.
use gpui::{App, AppContext, Entity, Transition, TransitionState, Window};

pub struct DownloadsFade {
    state: Entity<TransitionState<f32>>,
    frame: Transition<f32>,
}

impl DownloadsFade {
    pub fn new(cx: &mut App) -> Self {
        let state = cx.new(|_| TransitionState::new(0.0));
        let frame = Transition::new(state.clone(), bbr_ui::theme::Motion::fine());
        Self { state, frame }
    }

    pub fn set_open(&self, open: bool, cx: &mut App) {
        self.frame.update(cx, |value, cx| {
            *value = if open { 1.0 } else { 0.0 };
            cx.notify();
        });
    }

    pub fn evaluate(&mut self, window: &mut Window, cx: &mut App) -> f32 {
        // Transition::evaluate caches within its wrapper, not the window's
        // frame lifetime. Keep goals/timestamps/last_delta in the same entity,
        // but do not retain the previous draw's cached interpolated value.
        // Only the browser's owning render may schedule animation frames;
        // floating rail composition reads the resulting presentation cache.
        self.frame = Transition::new(self.state.clone(), bbr_ui::theme::Motion::fine());
        *self.frame.evaluate(window, cx)
    }

    pub fn rendered_value(&self) -> f32 {
        self.frame
            .read_cache()
            .expect("primed by the browser render")
    }

    #[cfg(feature = "visual-capture")]
    pub fn rendered_sample(&self, cx: &mut App) -> (Option<f32>, f32, f32) {
        let cached = *self.frame.read_cache();
        let goal = *self.frame.read_goal(cx);
        (cached, goal, self.frame.evaluate_delta(cx))
    }
}

#[cfg(all(test, feature = "visual-capture"))]
mod tests {
    use super::*;

    struct FadeView(DownloadsFade);

    impl gpui::Render for FadeView {
        fn render(
            &mut self,
            window: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            self.0.evaluate(window, cx);
            gpui::div()
        }
    }

    #[gpui::test]
    fn drawing_refreshes_only_the_cached_value(cx: &mut gpui::TestAppContext) {
        // Mock TestPlatform: no native host window. Frame requests require a
        // rendering Entity, not merely an arbitrary element's draw callback.
        let (view, cx) = cx.add_window_view(|_, cx| FadeView(DownloadsFade::new(cx)));
        let identity = view.read_with(cx, |view, _| view.0.frame.entity_id());
        assert_eq!(draw(&view, cx), 0.0);
        view.update(cx, |view, cx| {
            view.0.set_open(true, cx);
            assert_eq!(*view.0.frame.read_cache(), Some(0.0));
            assert_eq!(*view.0.frame.read_goal(cx), 1.0);
        });
        let alpha = draw(&view, cx);
        assert!(alpha > 0.0 && alpha <= 1.0, "stale closed cache: {alpha}");
        // The floating composition may read during an ordinary window update,
        // even mid-fade: it must not evaluate or schedule a frame here.
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).0.rendered_value(), alpha);
            assert_eq!(view.read(cx).0.rendered_value(), alpha);
        });
        // Retarget through the same entity, preserving the current value.
        view.update(cx, |view, cx| view.0.set_open(false, cx));
        let closing = draw(&view, cx);
        assert!(closing >= 0.0 && closing <= alpha);
        view.update(cx, |view, cx| view.0.set_open(true, cx));
        let reopening = draw(&view, cx);
        assert!(reopening >= closing && reopening <= 1.0);
        assert_eq!(
            view.read_with(cx, |view, _| view.0.frame.entity_id()),
            identity
        );
    }

    fn draw(view: &Entity<FadeView>, cx: &mut gpui::VisualTestContext) -> f32 {
        cx.update(|window, cx| {
            window.refresh();
            let _clear = window.draw(cx);
        });
        view.read_with(cx, |view, _| view.0.frame.read_cache().expect("rendered"))
    }
}
