//! U2 sidebar benchmark drivers (DESIGN.md §11.4/§11.10/§11.11): the plus
//! menu toggle, the favorites tile add/remove, and the folder disclosure,
//! each through the production app path the pointer listeners call. A sample
//! commits only when the presented rail shows the expected semantic state —
//! the in-app probe guard rejects a fast-but-wrong presentation (missing
//! menu, wrong tile count, wrong expanded value) before it leaves the app.

use bbr_browser::FolderId;
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the §11.4 plus menu through the production footer `[+]`
    /// path: each sample toggles the menu (open on odd, close on even — the
    /// alternating toggle path the `[+]` control drives). The sample commits
    /// only when the presented rail shows the menu in the expected open
    /// state (the in-app probe guard rejects a fast-but-wrong presentation
    /// without the menu surface).
    pub fn benchmark_plus_menu(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarPlusMenu,
            )
            .is_err()
        {
            return false;
        }
        let open = !self.u2.plus_menu.open;
        self.performance.expect_plus_menu_open(open);
        // The production toggle path (the footer `[+]` pointer listener and
        // the keyboard toggle share this method).
        self.toggle_plus_menu(window, cx);
        self.performance.observe_plus_menu(self.u2.plus_menu.open);
        self.performance.observe_render_state(
            self.controller.read().snapshot().active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );
        self.performance.expect_browser_state(
            self.controller.read().snapshot().active_view,
            self.sidebar_visible,
        );
        self.performance.commit_current_presentation();
        true
    }

    /// Benchmark the §11.10 favorites toggle through the production path
    /// (the exact app method the drag-into-grid drop commits through): the
    /// sample commits only when the grid renders the expected tile count
    /// (guard), and the toggled tab keeps its space membership and session.
    pub fn benchmark_favorite_toggle(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarFavorites,
            )
            .is_err()
        {
            return false;
        }
        let before = self.controller.read().snapshot();
        let active_tab = before
            .all_tabs
            .iter()
            .filter(|tab| tab.space == before.active_space)
            .nth(index)
            .map(|tab| tab.id);
        let already_favorite = active_tab.is_some_and(|tab| {
            self.organization
                .get()
                .organization_snapshot(&self.controller.read())
                .favorites
                .iter()
                .any(|f| f.tab == tab)
        });
        let expected_len = if already_favorite {
            self.organization
                .get()
                .organization_snapshot(&self.controller.read())
                .favorites
                .len()
                .saturating_sub(1)
        } else {
            self.organization
                .get()
                .organization_snapshot(&self.controller.read())
                .favorites
                .len()
                + 1
        };
        // The active tab count bounds the toggle target; an out-of-range
        // index is a setup error, not a sample.
        self.favorite_toggle_tab(index, window, cx);
        let favorites_len = self
            .organization
            .get()
            .organization_snapshot(&self.controller.read())
            .favorites
            .len();
        if favorites_len != expected_len {
            self.performance.cancel();
            return false;
        }
        self.performance.expect_favorites_len(favorites_len);
        self.performance.observe_favorites(favorites_len);
        self.performance.observe_render_state(
            self.controller.read().snapshot().active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );
        self.performance.expect_browser_state(
            self.controller.read().snapshot().active_view,
            self.sidebar_visible,
        );
        self.performance.commit_current_presentation();
        true
    }

    /// Benchmark the §11.11 folder disclosure through the production toggle
    /// path: the sample commits only when the folder's expanded/collapsed
    /// value matches the toggle (guard — a presentation without the nested
    /// rows is rejected). The folder is keyed by its stable domain id.
    pub fn benchmark_folder_toggle(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        id: FolderId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarFolderToggle,
            )
            .is_err()
        {
            return false;
        }
        let expected = !self.u2.folder_presentation.is_expanded(id);
        // The production disclosure path (pointer toggle and keyboard
        // Enter/Space share this method).
        self.toggle_folder(id, cx);
        let expanded = self.u2.folder_presentation.is_expanded(id);
        if expanded != expected {
            self.performance.cancel();
            return false;
        }
        self.performance.expect_folder_expanded(expanded);
        self.performance.observe_folder_expanded(expanded);
        self.performance.observe_render_state(
            self.controller.read().snapshot().active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );
        self.performance.expect_browser_state(
            self.controller.read().snapshot().active_view,
            self.sidebar_visible,
        );
        self.performance.commit_current_presentation();
        true
    }
}
