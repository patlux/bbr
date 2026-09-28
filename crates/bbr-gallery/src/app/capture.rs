//! Opt-in capture selection still uses the Gallery's production control path.
use super::*;
use crate::{capture_cases::CaptureCase, capture_page::PageCapture, tree::NodeKind};

impl GalleryApp {
    pub(crate) fn select_capture(
        &mut self,
        case: &CaptureCase,
        page: PageCapture,
        cx: &mut Context<Self>,
    ) {
        let node = self
            .tree
            .nodes
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Story(id) if id == case.story))
            .expect("registered story has tree node")
            .id;
        self.activate_node(node, cx);
        self.apply_control(ControlAction::Mode(case.mode), cx);
        self.apply_control(ControlAction::Contrast(case.contrast), cx);
        self.apply_control(ControlAction::Density(case.density), cx);
        self.apply_control(ControlAction::ReducedMotion(true), cx);
        self.apply_control(ControlAction::State(case.state), cx);
        self.capture = Some(page);
        assert_eq!(self.tree.selected_story, case.story);
        assert_eq!(self.story_state_label(case.story), case.state_label);
        cx.notify();
    }
}
