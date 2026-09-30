//! Blueprint D19 (several windows, task packet `multi-window.md`): building
//! a window's shell. The first window builds the shared state; every
//! further window's shell shares it ([`BbrApp::sibling`]) and starts its own
//! presentation state fresh. [`BbrApp::assemble`] lists every field, so a new
//! field has to decide here whether it is shared or per window.

use super::*;
use bbr_browser::WindowKey;
use gpui::ScrollHandle;

/// The state a window's shell shares with the other windows (handles and
/// settled configuration), handed to [`BbrApp::assemble`].
pub(crate) struct SharedParts {
    pub(crate) windows: crate::shared::Shared<crate::browser_windows::Registry>,
    #[cfg(target_os = "macos")]
    pub(crate) find_tx: smol::channel::Sender<bbr_web_macos::find_mapping::FindCompletion>,
    pub(crate) engine: EngineRef,
    pub(crate) engine_kind: &'static str,
    pub(crate) controller: crate::shared::ControllerHandle,
    pub(crate) media: crate::shared::Shared<media::MediaState>,
    pub(crate) split_layouts_saved:
        crate::shared::Shared<Option<crate::split_layouts::SpaceGroups>>,
    pub(crate) menu_spaces: crate::shared::Shared<Option<Vec<crate::menus::MenuSpace>>>,
    pub(crate) recovery_dismissed: crate::shared::Shared<bool>,
    pub(crate) about_window:
        crate::shared::Shared<Option<gpui::WindowHandle<crate::about::AboutView>>>,
    pub(crate) error_surface: crate::shared::Shared<error_surface::ErrorSurfaceState>,
    pub(crate) auth: crate::shared::Shared<auth::AuthCoordinator>,
    #[cfg(target_os = "macos")]
    pub(crate) external_urls:
        crate::shared::Shared<bbr_web_macos::external_urls_adapter::ExternalUrlAdapterCore>,
    pub(crate) store: crate::shared::Shared<crate::store::AppStore>,
    #[cfg(feature = "testkit")]
    pub(crate) appearance_override: crate::shared::Shared<Option<bbr_ui::theme::ThemeMode>>,
    pub(crate) system_accessibility: crate::shared::Shared<appearance::AccessibilityPrefs>,
    #[cfg(feature = "testkit")]
    pub(crate) accessibility_override: crate::shared::Shared<appearance::AccessibilityOverride>,
    pub(crate) preferences: crate::shared::Shared<crate::preferences::Preferences>,
    pub(crate) settings: crate::shared::Shared<settings::SettingsState>,
    pub(crate) unresponsive: crate::shared::Shared<unresponsive::UnresponsiveState>,
    pub(crate) organization: crate::shared::Shared<bbr_browser::OrganizationController>,
    pub(crate) new_window: crate::shared::Shared<new_window::NewWindowState>,
    pub(crate) content_gesture: crate::shared::Shared<content_gesture::ContentGestureState>,
    pub(crate) favicon_images: crate::shared::Shared<favicon_presentation::FaviconImageMirror>,
    pub(crate) favicon_mirror_version: crate::shared::Shared<u64>,
    pub(crate) split_layouts_path: Option<std::path::PathBuf>,
    pub(crate) preferences_path: Option<std::path::PathBuf>,
    pub(crate) import_home: Option<std::path::PathBuf>,
    pub(crate) windows_hidden: bool,
    pub(crate) restore_evidence: RestoreEvidence,
    pub(crate) saved_pages: saved_pages::SavedPageCommand,
    pub(crate) downloads: downloads::DownloadsState,
    pub(crate) permissions: permissions::PermissionsState,
    pub(crate) certificates: client_certificates::CertificateState,
    pub(crate) zoom: zoom::ZoomController,
    #[cfg(target_os = "macos")]
    pub(crate) download_web_view: bbr_web_macos::downloads_adapter::DownloadWebViewSlot,
    #[cfg(target_os = "macos")]
    pub(crate) downloads_adapter:
        Option<Rc<bbr_web_macos::downloads_adapter::NativeDownloadAdapter>>,
    #[cfg(target_os = "macos")]
    pub(crate) download_engine: Option<crate::engine::NativeEngineRef>,
    #[cfg(target_os = "macos")]
    pub(crate) find_engine: Option<crate::engine::NativeEngineRef>,
    #[cfg(target_os = "macos")]
    pub(crate) zoom_engine: Option<crate::engine::NativeEngineRef>,
    #[cfg(target_os = "macos")]
    pub(crate) menu_engine: Option<crate::engine::NativeEngineRef>,
    #[cfg(target_os = "macos")]
    pub(crate) geometry_engine: Option<crate::engine::NativeEngineRef>,
    #[cfg(all(target_os = "macos", feature = "testkit"))]
    pub(crate) web_content_probe: Rc<
        RefCell<
            Option<(
                bbr_web::WebViewId,
                bbr_web_macos::content_probe::ContentProbe,
            )>,
        >,
    >,
    pub(crate) focus_sink: Rc<RefCell<Option<FocusSink>>>,
    pub(crate) browser_sink: Rc<RefCell<Option<BrowserSink>>>,
    #[cfg(feature = "testkit")]
    pub(crate) performance: PerformanceProbe,
}

impl BbrApp {
    /// The wiring constructor: engine, controller, organization, event
    /// channels, native find channel (macOS), and the testkit performance
    /// probe are all injected together.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: EngineRef,
        controller: BrowserController,
        organization: bbr_browser::OrganizationController,
        engine_kind: &'static str,
        evt_tx: smol::channel::Sender<WebEvent>,
        #[cfg(target_os = "macos")] find_tx: smol::channel::Sender<
            bbr_web_macos::find_mapping::FindCompletion,
        >,
        #[cfg(feature = "testkit")] performance: PerformanceProbe,
        store: crate::store::AppStore,
        cx: &mut Context<Self>,
    ) -> Self {
        // Engine → app events: the sink runs on the main thread (delegate
        // callbacks); the channel is drained by the pump task that mirrors
        // into `shell` and (under testkit) broadcasts to E2E clients.
        let sink_tx = evt_tx.clone();
        engine
            .borrow_mut()
            .set_events(Box::new(move |ev: WebEvent| {
                let _ = sink_tx.try_send(ev);
            }));

        // Startup restore of the terminal download history (DESIGN.md §17.10
        // settled: terminal history persists through restarts): the persisted
        // terminal records are mapped onto the controller **before the first
        // frame**, so the panel and the footer badge render the restored rows
        // from the first user-observable paint. Live attempts are never
        // persisted and never resurrected; the repository validates outcomes
        // on load, so a non-terminal record cannot reach the restore seam.
        let downloads_state = downloads::DownloadsState::new();
        match crate::store::downloads::from_persisted(store.restored_downloads())
            .into_iter()
            .map(bbr_browser::DownloadRecord::restored_terminal)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(records) => {
                let restored = downloads_state
                    .controller
                    .get()
                    .restore_terminal_history(records);
                if let Err(error) = restored {
                    log::warn!("downloads history restore failed: {error:?}");
                }
            }
            Err(error) => log::warn!("downloads history restore failed: {error:?}"),
        }
        let parts = SharedParts {
            windows: Default::default(),
            #[cfg(target_os = "macos")]
            find_tx,
            engine: engine.clone(),
            engine_kind,
            controller: crate::shared::ControllerHandle::new(controller),
            media: Default::default(),
            split_layouts_saved: Default::default(),
            menu_spaces: Default::default(),
            recovery_dismissed: Default::default(),
            about_window: Default::default(),
            error_surface: Default::default(),
            auth: crate::shared::Shared::new(crate::app::auth::AuthCoordinator::new()),
            #[cfg(target_os = "macos")]
            external_urls: crate::shared::Shared::new(
                bbr_web_macos::external_urls_adapter::ExternalUrlAdapterCore::new(),
            ),
            store: crate::shared::Shared::new(store),
            #[cfg(feature = "testkit")]
            appearance_override: Default::default(),
            system_accessibility: Default::default(),
            #[cfg(feature = "testkit")]
            accessibility_override: Default::default(),
            preferences: Default::default(),
            settings: Default::default(),
            unresponsive: crate::shared::Shared::new(unresponsive::UnresponsiveState::new()),
            organization: crate::shared::Shared::new(organization),
            new_window: Default::default(),
            content_gesture: Default::default(),
            favicon_images: Default::default(),
            favicon_mirror_version: Default::default(),
            split_layouts_path: None,
            preferences_path: None,
            import_home: None,
            windows_hidden: false,
            restore_evidence: RestoreEvidence::Fresh,
            saved_pages: saved_pages::SavedPageCommand,
            downloads: downloads_state,
            permissions: permissions::PermissionsState::default(),
            certificates: client_certificates::CertificateState::default(),
            zoom: crate::app::zoom::ZoomController::new(crate::app::zoom::noop_target()),
            #[cfg(target_os = "macos")]
            download_web_view: bbr_web_macos::downloads_adapter::DownloadWebViewSlot::new(),
            #[cfg(target_os = "macos")]
            downloads_adapter: None,
            #[cfg(target_os = "macos")]
            download_engine: None,
            #[cfg(target_os = "macos")]
            find_engine: None,
            #[cfg(target_os = "macos")]
            zoom_engine: None,
            #[cfg(target_os = "macos")]
            menu_engine: None,
            #[cfg(target_os = "macos")]
            geometry_engine: None,
            #[cfg(all(target_os = "macos", feature = "testkit"))]
            web_content_probe: std::rc::Rc::new(std::cell::RefCell::new(None)),
            focus_sink: Rc::new(RefCell::new(None)),
            browser_sink: Rc::new(RefCell::new(None)),
            #[cfg(feature = "testkit")]
            performance,
        };
        Self::assemble(parts, cx)
    }

    /// A further window's shell over the same shared state: its own
    /// presentation state, the controller in `window`'s context, and its
    /// own find-completion channel.
    pub(crate) fn sibling_parts(
        &self,
        window: WindowKey,
        #[cfg(target_os = "macos")] find_tx: smol::channel::Sender<
            bbr_web_macos::find_mapping::FindCompletion,
        >,
    ) -> SharedParts {
        SharedParts {
            windows: self.windows.clone(),
            #[cfg(target_os = "macos")]
            find_tx,
            engine: self.engine.clone(),
            engine_kind: self.engine_kind,
            controller: self.controller.for_window(window),
            media: self.media.clone(),
            split_layouts_saved: self.split_layouts_saved.clone(),
            menu_spaces: self.menu_spaces.clone(),
            recovery_dismissed: self.recovery_dismissed.clone(),
            about_window: self.about_window.clone(),
            error_surface: self.error_surface.clone(),
            auth: self.auth.clone(),
            #[cfg(target_os = "macos")]
            external_urls: self.external_urls.clone(),
            store: self.store.clone(),
            #[cfg(feature = "testkit")]
            appearance_override: self.appearance_override.clone(),
            system_accessibility: self.system_accessibility.clone(),
            #[cfg(feature = "testkit")]
            accessibility_override: self.accessibility_override.clone(),
            preferences: self.preferences.clone(),
            settings: self.settings.clone(),
            unresponsive: self.unresponsive.clone(),
            organization: self.organization.clone(),
            new_window: self.new_window.clone(),
            content_gesture: self.content_gesture.clone(),
            favicon_images: self.favicon_images.clone(),
            favicon_mirror_version: self.favicon_mirror_version.clone(),
            split_layouts_path: self.split_layouts_path.clone(),
            preferences_path: self.preferences_path.clone(),
            import_home: self.import_home.clone(),
            windows_hidden: self.windows_hidden,
            restore_evidence: self.restore_evidence,
            saved_pages: self.saved_pages,
            downloads: self.downloads.sibling(),
            permissions: self.permissions.sibling(),
            certificates: self.certificates.sibling(),
            zoom: self.zoom.sibling(),
            #[cfg(target_os = "macos")]
            download_web_view: self.download_web_view.clone(),
            #[cfg(target_os = "macos")]
            downloads_adapter: self.downloads_adapter.clone(),
            #[cfg(target_os = "macos")]
            download_engine: self.download_engine.clone(),
            #[cfg(target_os = "macos")]
            find_engine: self.find_engine.clone(),
            #[cfg(target_os = "macos")]
            zoom_engine: self.zoom_engine.clone(),
            #[cfg(target_os = "macos")]
            menu_engine: self.menu_engine.clone(),
            #[cfg(target_os = "macos")]
            geometry_engine: self.geometry_engine.clone(),
            #[cfg(all(target_os = "macos", feature = "testkit"))]
            web_content_probe: self.web_content_probe.clone(),
            focus_sink: self.focus_sink.clone(),
            browser_sink: self.browser_sink.clone(),
            #[cfg(feature = "testkit")]
            performance: self.performance.clone(),
        }
    }

    /// A window's shell from its shared parts, with fresh presentation
    /// state.
    pub(crate) fn assemble(parts: SharedParts, cx: &mut Context<Self>) -> Self {
        BbrApp {
            windows: parts.windows,
            seen_generation: 0,
            window_watch: Vec::new(),
            #[cfg(feature = "testkit")]
            window_refreshes: 0,
            engine: parts.engine,
            controller: parts.controller,
            engine_kind: parts.engine_kind,
            chrome_handle: cx.focus_handle(),
            space_strip_handle: cx.focus_handle(),
            tab_list_handle: cx.focus_handle(),
            divider_handle: cx.focus_handle(),
            dense_scroll: ScrollHandle::new(),
            space_strip_scroll: ScrollHandle::new(),
            dense_window: std::cell::Cell::new(crate::dense::DenseRenderState::default()),
            sidebar: bbr_ui::sidebar::SidebarGeometry::new(),
            tab_drag: drag::TabDrag::default(),
            native_portal: None,
            transient: transient::TransientOverlays::default(),
            permissions: parts.permissions,
            media: parts.media,
            split_view: split_view::SplitView::default(),
            split_layouts_path: parts.split_layouts_path,
            split_layouts_saved: parts.split_layouts_saved,
            certificates: parts.certificates,
            open_app: open_app::OpenAppState::default(),
            space_chooser: None,
            onboarding: None,
            import_home: parts.import_home,
            menu_spaces: parts.menu_spaces,
            recovery_dismissed: parts.recovery_dismissed,
            about_window: parts.about_window,
            windows_hidden: parts.windows_hidden,
            portal: PortalState::new(crate::portal::probe::PORTAL_ID),
            portal_probe: Rc::new(RefCell::new(PortalProbeState::new())),
            hovered_row: None,
            hovered_folder: None,
            pressed_row: None,
            address_hovered: false,
            motion: motion::MotionState::default(),
            moments: moments::Moments::default(),
            titlebar_hovered: None,
            titlebar_pressed: None,
            nav_history: Default::default(),
            nav_press: None,
            nav_press_generation: 0,
            nav_hold_timer: None,
            command_bar_mode: CommandBarMode::Inactive,
            command_bar_return: None,
            command_surface: CommandSurface::None,
            find: crate::app::find::FindController::new(
                #[cfg(target_os = "macos")]
                parts.find_tx,
            ),
            error_surface: parts.error_surface,
            find_handle: cx.focus_handle(),
            auth: parts.auth,
            auth_handle: cx.focus_handle(),
            space_delete: None,
            space_delete_handle: cx.focus_handle(),
            downloads: parts.downloads,
            downloads_panel_handle: cx.focus_handle(),
            downloads_control_handle: cx.focus_handle(),
            downloads_press_inside: std::cell::Cell::new(false),
            #[cfg(target_os = "macos")]
            download_web_view: parts.download_web_view,
            #[cfg(target_os = "macos")]
            downloads_adapter: parts.downloads_adapter,
            #[cfg(target_os = "macos")]
            download_engine: parts.download_engine,
            #[cfg(target_os = "macos")]
            find_engine: parts.find_engine,
            #[cfg(all(target_os = "macos", feature = "testkit"))]
            web_content_probe: parts.web_content_probe,
            #[cfg(target_os = "macos")]
            external_urls: parts.external_urls,
            zoom: parts.zoom,
            #[cfg(target_os = "macos")]
            zoom_engine: parts.zoom_engine,
            context_menu: crate::app::context_menu::ContextMenuController::new(),
            tab_menu: crate::app::tab_menu::TabMenuController::new(),
            menu_handle: cx.focus_handle(),
            #[cfg(target_os = "macos")]
            menu_engine: parts.menu_engine,
            #[cfg(target_os = "macos")]
            geometry_engine: parts.geometry_engine,
            command_field_rect: std::rc::Rc::new(std::cell::RefCell::new(None)),
            favorites_band_rect: std::rc::Rc::new(std::cell::RefCell::new(None)),
            drag_offer_rects: drag::DragOfferRects::default(),
            command_text: CommandText::new(),
            command_line: bbr_ui::components::EditLineSink::new(),
            floating_rail: rail_float::FloatingRail::default(),
            preferences: parts.preferences,
            preferences_path: parts.preferences_path,
            settings: parts.settings,
            browser_window: None,
            #[cfg(feature = "testkit")]
            text_input_readback: None,
            command_ranking: CommandRanking::default(),
            saved_pages: parts.saved_pages,
            library: crate::app::library::LibraryState::default(),
            library_handle: cx.focus_handle(),
            unresponsive: parts.unresponsive,
            unresponsive_wait_handle: cx.focus_handle(),
            unresponsive_reload_handle: cx.focus_handle(),
            unresponsive_close_handle: cx.focus_handle(),
            command_profile_hint: None,
            store: parts.store,
            restore_evidence: parts.restore_evidence,
            // The observed system appearance is corrected by
            // `observe_system_appearance` at composition (before the first
            // presented frame); Light is the deterministic pre-observation
            // default.
            system_appearance: bbr_ui::theme::ThemeMode::Light,
            #[cfg(feature = "testkit")]
            appearance_override: parts.appearance_override,
            system_accessibility: parts.system_accessibility,
            #[cfg(feature = "testkit")]
            accessibility_override: parts.accessibility_override,
            #[cfg(target_os = "macos")]
            accessibility_observer: None,
            accessibility_task: None,
            #[cfg(feature = "testkit")]
            full_screen_override: None,
            #[cfg(feature = "testkit")]
            auth_presented_theme: std::cell::Cell::new(None),
            #[cfg(feature = "testkit")]
            peek_presented_theme: std::cell::Cell::new(None),
            #[cfg(feature = "testkit")]
            peek_presented_frame: std::cell::Cell::new(None),
            appearance_subscription: None,
            window_geometry: std::cell::RefCell::new(None),
            sidebar_visible: true,
            organization: parts.organization,
            u2: u2::U2State::new(cx),
            space_config: space_config::SpaceConfigState::new(cx),
            swipe: swipe::SwipeState::new(),
            focus_sink: parts.focus_sink,
            browser_sink: parts.browser_sink,
            new_window: parts.new_window,
            content_gesture: parts.content_gesture,
            peek: peek::PeekState::default(),
            favicon_images: parts.favicon_images,
            favicon_mirror_version: parts.favicon_mirror_version,
            #[cfg(feature = "testkit")]
            performance: parts.performance,
        }
    }
}
