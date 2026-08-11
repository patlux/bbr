//! The gallery canvas surface (packet `component-gallery-mvp.md`): renders
//! the selected story with the resolved theme (mode/contrast/density) and
//! the story's own state. The canvas is the semantic root of the story
//! (`bbr-gallery-canvas`, `bbr-gallery-canvas-story-<id>`); the E2E asserts
//! the §15 mode fields through the app snapshot, never pixels.

use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, AnyElement, SharedString};

use crate::registry::{
    AddressStory, AuthStory, CatalogRegistry, ControlsStory, DownloadsStory, ErrorStory, FindStory,
    Group4Story, LibraryStory, StoryContext, StoryId, TabRowStory,
};

/// Everything the canvas needs to render the selected story.
pub struct CanvasModel<'a> {
    pub registry: &'a CatalogRegistry,
    pub selected: StoryId,
    pub theme: &'a Theme,
    pub reduced_motion: bool,
    pub story_state: &'a str,
    /// The Core Controls context (`Some` only when a Core Controls story is
    /// selected) — see [`ControlsStory`].
    pub controls: Option<ControlsStory<'a>>,
    /// The Tab Row context (`Some` only when the Tab Row story is
    /// selected) — see [`TabRowStory`].
    pub tab_row: Option<TabRowStory<'a>>,
    /// The Saved Pages Library context (`Some` only when a Library story is
    /// selected) — see [`LibraryStory`].
    pub library: Option<LibraryStory<'a>>,
    /// The Address/Input context (`Some` only when the Address control or
    /// Command staging story is selected) — see [`AddressStory`].
    pub address: Option<AddressStory<'a>>,
    /// The Group 4 sidebar-component context (`Some` only when one of the
    /// five Group 4 stories is selected) — see [`Group4Story`].
    pub group4: Option<Group4Story<'a>>,
    /// The Downloads context (`Some` only when the Downloads panel story
    /// is selected) — see [`DownloadsStory`].
    pub downloads: Option<DownloadsStory<'a>>,
    /// The Find context (`Some` only when the Find bar story is selected)
    /// — see [`FindStory`].
    pub find: Option<FindStory<'a>>,
    /// The Error/Crash context (`Some` only when the Error surface or
    /// Crash / Empty panes story is selected) — see [`ErrorStory`].
    pub error: Option<ErrorStory<'a>>,
    /// The Auth sheet context (`Some` only when the Auth sheet story is
    /// selected) — see [`AuthStory`].
    pub auth: Option<AuthStory<'a>>,
}

/// The canvas surface: a header (story label + state) above the story.
pub fn canvas_surface(model: CanvasModel) -> AnyElement {
    let story = model
        .registry
        .story(model.selected)
        .expect("selected story is registered");
    let context = StoryContext {
        theme: model.theme,
        reduced_motion: model.reduced_motion,
        state: model.story_state,
        controls: model.controls,
        tab_row: model.tab_row,
        library: model.library,
        address: model.address,
        group4: model.group4,
        downloads: model.downloads,
        find: model.find,
        error: model.error,
        auth: model.auth,
    };
    let content = (story.render)(&context);
    div()
        .id("bbr-gallery-canvas")
        .flex()
        .flex_col()
        .size_full()
        .bg(model.theme.palette.surface_canvas)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(model.theme.palette.hairline)
                .child(
                    div()
                        .text_size(model.theme.type_.title.size)
                        .font_weight(model.theme.type_.title.weight)
                        .text_color(model.theme.palette.ink)
                        .child(story.label),
                )
                .child(
                    div()
                        .text_size(model.theme.type_.meta.size)
                        .text_color(model.theme.palette.ink_faint)
                        .child(SharedString::from(format!(
                            "story {} · state {}",
                            story.id.0, model.story_state
                        ))),
                ),
        )
        .child(
            div()
                .id(SharedString::from(format!(
                    "bbr-gallery-canvas-story-{}",
                    story.id.0
                )))
                .flex()
                .flex_1()
                .size_full()
                .min_w_0()
                .child(content),
        )
        .into_any()
}
