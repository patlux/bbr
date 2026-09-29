//! Blueprint D19: keep the menu bar and the Dock menu in step with the
//! space list (see `crate::menus`).

use super::BbrApp;

impl BbrApp {
    /// Converge the menu bar and the Dock menu with the space list (the
    /// render calls this; a change installs the menus after the frame).
    pub(crate) fn sync_menus(&mut self, cx: &mut gpui::Context<Self>) {
        let browser = self.controller.snapshot();
        let spaces: Vec<crate::menus::MenuSpace> = browser
            .spaces
            .iter()
            .map(|space| crate::menus::MenuSpace {
                id: space.id.0,
                title: space.title.clone(),
                active: space.active,
            })
            .collect();
        if self.menu_spaces.as_ref() != Some(&spaces) {
            self.menu_spaces = Some(spaces.clone());
            cx.defer(move |cx| crate::menus::install(cx, &spaces));
        }
    }

    /// The spaces the installed menus list (empty before the first render).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub(crate) fn menu_spaces(&self) -> &[crate::menus::MenuSpace] {
        self.menu_spaces.as_deref().unwrap_or(&[])
    }
}
