//! The gallery story-context types (packet `component-gallery-mvp.md`).
//!
//! A bounded split of `registry.rs` (the G3 Sidebar story adds a context
//! family; the registry stays under the ~500-line module budget): every
//! story receives a [`StoryContext`] with the resolved theme, the
//! reduced-motion flag, the story's own state, and the recorder/observer
//! bundle of the story family it belongs to. The registry re-exports these
//! types so the declaration and the render functions keep one import path.

use gpui::{Entity, FocusHandle};
use gpui_component::input::InputState;
use gpui_component::slider::SliderState;

use crate::address_recorder::{AddressObserver, AddressRecorder};
use crate::auth_recorder::{AuthObserver, AuthRecorder};
use crate::command_recorder::CommandRecorder;
use crate::context_menu_recorder::{ContextMenuObserver, ContextMenuRecorder};
use crate::downloads_recorder::{DownloadsObserver, DownloadsRecorder};
use crate::error_recorder::{ErrorObserver, ErrorRecorder};
use crate::find_recorder::{FindObserver, FindRecorder};
use crate::group4_recorder::{Group4Observer, Group4Recorder};
use crate::library_recorder::{LibraryObserver, LibraryRecorder};
use crate::recorder::{ControlObserver, ControlRecorder, TabRowObserver, TabRowRecorder};
use crate::sidebar_observer::SidebarObserver;
use crate::sidebar_recorder::SidebarRecorder;
use crate::space_config_recorder::{SpaceConfigObserver, SpaceConfigRecorder};

/// The render context a story receives: the resolved theme (mode/contrast/
/// density), the reduced-motion flag (§15 mode field), and the story's own
/// selected state. G1b stories are render-only (the G1b `bbr-ui` primitives
/// take no callbacks); the Core Controls stories additionally receive the
/// real gpui-component state entities and the recorder/observer bundle
/// through [`StoryContext::controls`]; the Saved Pages Library stories
/// receive the recorder/observer bundle through [`StoryContext::library`].
pub struct StoryContext<'a> {
    pub theme: &'a bbr_ui::theme::Theme,
    pub reduced_motion: bool,
    pub state: &'a str,
    /// `Some` only for the Core Controls stories: the real gpui-component
    /// state entities, the current recorder snapshot, and the observer
    /// callbacks real control events route through. `None` for every other
    /// story.
    pub controls: Option<ControlsStory<'a>>,
    /// `Some` only for the Tab Row story: the Tab Row recorder snapshot
    /// and the observer callbacks the production `tab_row` callbacks route
    /// through (packet `component-gallery-browser-components.md`). `None`
    /// for every other story.
    pub tab_row: Option<TabRowStory<'a>>,
    /// `Some` only for the Saved Pages Library stories: the current
    /// recorder snapshot and the observer callbacks real Library events
    /// route through. `None` for every other story.
    pub library: Option<LibraryStory<'a>>,
    /// `Some` only for the Address control / Command staging stories: the
    /// current recorder snapshot and the observer callbacks the production
    /// `address_control`/`staging_surface` callbacks route through (packet
    /// `component-gallery-browser-components.md`, Group 3). `None` for
    /// every other story.
    pub address: Option<AddressStory<'a>>,
    /// `Some` only for the five Group 4 sidebar-component stories.
    pub group4: Option<Group4Story<'a>>,
    /// `Some` only for the Downloads panel story.
    pub downloads: Option<DownloadsStory<'a>>,
    /// `Some` only for the Find bar story.
    pub find: Option<FindStory<'a>>,
    /// `Some` only for the Error surface / Crash & Empty panes stories.
    pub error: Option<ErrorStory<'a>>,
    /// `Some` only for the Auth sheet story: the current recorder snapshot
    /// and the observer callbacks the production `bbr_ui::surfaces::auth`
    /// `auth_sheet` callbacks route through (packet
    /// `component-gallery-browser-components.md`, Group 8). `None` for
    /// every other story.
    pub auth: Option<AuthStory<'a>>,
    /// `Some` only for the Context-menu story: the current recorder
    /// snapshot and the observer callbacks the production
    /// `bbr_ui::surfaces::menu::context_menu_card` row callbacks route
    /// through (packet `component-gallery-browser-components.md`, Group
    /// 9). `None` for every other story.
    pub context_menu: Option<ContextMenuStory<'a>>,
    /// `Some` only for the G3 Sidebar composition story: the current
    /// recorder snapshot and the observer callbacks the production
    /// `bbr_ui::sidebar::sidebar_surface` `Callbacks` route through
    /// (packet `component-gallery-compositions.md`). `None` for every
    /// other story.
    pub sidebar: Option<SidebarStory<'a>>,
    /// `Some` only for the G3 Command composition story: the current
    /// recorder snapshot and the fixture model the production
    /// `bbr_ui::surfaces::command::command_card` renders (packet
    /// `component-gallery-compositions.md`). The surface takes no
    /// callbacks — activation stays app/Portal-owned, so the Gallery
    /// observes presentation only. `None` for every other story.
    pub command: Option<CommandStory<'a>>,
    /// `Some` only for the Space-configuration story: the current recorder
    /// snapshot and the observer callbacks the production
    /// `bbr_ui::surfaces::space_config` surface callbacks route through
    /// (packet `space-creation-appearance-ui.md`). `None` for every other
    /// story.
    pub space_config: Option<SpaceConfigStory<'a>>,
}

/// Everything a Saved Pages Library story needs to render the **exact**
/// production `bbr-ui` Library rows/controls/surface with fixture models:
/// the current recorder snapshot and the observer callbacks that re-enter
/// the app on real Library events.
pub struct LibraryStory<'a> {
    pub recorder: &'a LibraryRecorder,
    pub observer: &'a LibraryObserver,
}

/// Everything a Core Controls story needs to render the **exact** vendored
/// gpui-component production symbols with fixture state: the real
/// `InputState`/`SliderState` entities (the controls render them directly —
/// never a gallery-local copy), the current recorder snapshot (checked
/// states, click counts), and the observer callbacks that re-enter the app
/// on real control events.
pub struct ControlsStory<'a> {
    pub input: Option<&'a Entity<InputState>>,
    pub slider: Option<&'a Entity<SliderState>>,
    pub recorder: &'a ControlRecorder,
    pub observer: &'a ControlObserver,
    /// The real `InputState::value()` snapshot (read by the app each
    /// render) — the story's own value row.
    pub input_value: String,
    /// The real `SliderState::value().start()` snapshot (read by the app
    /// each render) — the story's value row and the Progress mirror.
    pub slider_value: f32,
}

/// Everything a Tab Row story needs to render the **exact** production
/// `bbr_ui::components::tab_row` with recorder callbacks: the current
/// recorder snapshot (select/close/pin/drag counts) and the observer
/// callbacks that re-enter the app on real pointer interactions (packet
/// `component-gallery-browser-components.md`). `Some` only while the Tab
/// Row story is selected.
pub struct TabRowStory<'a> {
    pub recorder: &'a TabRowRecorder,
    pub observer: &'a TabRowObserver,
}

/// Everything an Address control / Command staging story needs to render
/// the **exact** production `bbr_ui::components::address` symbols
/// (`address_control`, `staging_surface`) with recorder callbacks: the
/// current recorder snapshot (hover states, open count) and the observer
/// callbacks that re-enter the app on real pointer interactions (packet
/// `component-gallery-browser-components.md`, Group 3). `Some` only while
/// one of the two Address stories is selected.
pub struct AddressStory<'a> {
    pub recorder: &'a AddressRecorder,
    pub observer: &'a AddressObserver,
}

/// Group 4 sidebar-component story context.
pub struct Group4Story<'a> {
    pub recorder: &'a Group4Recorder,
    pub observer: &'a Group4Observer,
    pub downloads_focus: FocusHandle,
}

/// Downloads panel story context.
pub struct DownloadsStory<'a> {
    pub recorder: &'a DownloadsRecorder,
    pub observer: &'a DownloadsObserver,
    pub scroll: &'a gpui::ScrollHandle,
    pub focus: &'a gpui::FocusHandle,
}

/// Find bar story context.
pub struct FindStory<'a> {
    pub recorder: &'a FindRecorder,
    pub observer: &'a FindObserver,
}

/// Error surface / Crash & Empty panes story context.
pub struct ErrorStory<'a> {
    pub recorder: &'a ErrorRecorder,
    pub observer: &'a ErrorObserver,
}

/// Everything the Auth sheet story needs to render the **exact** production
/// `bbr_ui::surfaces::auth::auth_sheet` with recorder callbacks: the
/// current recorder snapshot (submit/cancel counts) and the observer
/// callbacks that re-enter the app on real pointer interactions (packet
/// `component-gallery-browser-components.md`, Group 8). `Some` only while
/// the Auth sheet story is selected.
pub struct AuthStory<'a> {
    pub recorder: &'a AuthRecorder,
    pub observer: &'a AuthObserver,
}

/// Everything the Context-menu story needs to render the **exact**
/// production `bbr_ui::surfaces::menu::context_menu_card` with recorder
/// callbacks: the current recorder snapshot (hovered/pressed row indexes)
/// and the observer callbacks that re-enter the app on real pointer
/// interactions (packet `component-gallery-browser-components.md`, Group
/// 9). `Some` only while the Context-menu story is selected.
pub struct ContextMenuStory<'a> {
    pub recorder: &'a ContextMenuRecorder,
    pub observer: &'a ContextMenuObserver,
}

/// Everything the Space-configuration story needs to render the **exact**
/// production `bbr_ui::surfaces::space_config` surface with recorder
/// callbacks: the current recorder snapshot (selected theme/identity, done/
/// cancel/inside-press events) and the observer callbacks that re-enter
/// the app on real pointer interactions (packet
/// `space-creation-appearance-ui.md`). `Some` only while the
/// Space-configuration story is selected.
pub struct SpaceConfigStory<'a> {
    pub recorder: &'a SpaceConfigRecorder,
    pub observer: &'a SpaceConfigObserver,
    /// The fixture focus handle the production surface tracks (§9 ring +
    /// keyboard domain; the gallery never focuses it).
    pub focus: &'a gpui::FocusHandle,
}

/// Everything the G3 Sidebar composition story needs to render the **exact**
/// production `bbr_ui::sidebar::sidebar_surface` with fixture models: the
/// current recorder snapshot (what the production `Callbacks` slots
/// reported), the observer callbacks that re-enter the app on real
/// pointer/keyboard interactions, and the fixture handles the plain model
/// needs (packet `component-gallery-compositions.md`). `Some` only while
/// the Sidebar story is selected.
pub struct SidebarStory<'a> {
    pub recorder: &'a SidebarRecorder,
    pub observer: &'a SidebarObserver,
    /// The fixture handles the plain [`bbr_ui::sidebar::SidebarModel`]
    /// needs (GPUI handles cannot be created without a context; the app
    /// state creates them once and shares them across every family).
    pub handles: &'a crate::sidebar_fixture::SidebarFixtureHandles,
}

/// Everything the G3 Command composition story needs to render the **exact**
/// production `bbr_ui::surfaces::command::command_card` with a fixture
/// model: the current recorder snapshot (the presentation summary the story
/// observed) and the model itself. The production surface takes no
/// callbacks, so no callback/observer bundle exists — the Gallery observes
/// presentation only (packet `component-gallery-compositions.md`). `Some`
/// only while the Command story is selected.
pub struct CommandStory<'a> {
    pub recorder: &'a CommandRecorder,
    pub model: Option<&'a bbr_ui::surfaces::command::CommandModel>,
}
