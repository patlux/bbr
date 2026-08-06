//! Structural invariant validation for the browser model.

use crate::{Browser, InvariantViolation, TabLifecycle};

impl Browser {
    /// Public invariant checker (see crate docs). Every mutation keeps these
    /// by construction; tests also call it to prove no tampering path leaks.
    pub fn check_invariants(&self) -> Result<(), InvariantViolation> {
        let fail = |msg: String| InvariantViolation(msg);

        for profile in &self.profiles {
            if self.profile(profile.id) != Some(profile) {
                return Err(fail(format!(
                    "profile table inconsistent for {:?}",
                    profile.id
                )));
            }
        }

        for space in &self.spaces {
            if !self.profiles.iter().any(|p| p.id == space.profile) {
                return Err(fail(format!(
                    "space {:?} references unknown profile {:?}",
                    space.id, space.profile
                )));
            }
            if space.pinned_count > space.tabs.len() {
                return Err(fail(format!(
                    "space {:?} pinned_count {} exceeds tab count {}",
                    space.id,
                    space.pinned_count,
                    space.tabs.len()
                )));
            }
            let mut seen = std::collections::HashSet::new();
            for (i, tid) in space.tabs.iter().enumerate() {
                if !seen.insert(tid) {
                    return Err(fail(format!(
                        "space {:?} lists tab {:?} more than once at index {i}",
                        space.id, tid
                    )));
                }
                let tab = self.tab(*tid).ok_or_else(|| {
                    fail(format!(
                        "space {:?}[{i}] references unknown tab {:?}",
                        space.id, tid
                    ))
                })?;
                if tab.space != space.id {
                    return Err(fail(format!(
                        "tab {:?} claims space {:?} but is ordered in {:?}",
                        tid, tab.space, space.id
                    )));
                }
                if tab.pinned != (i < space.pinned_count) {
                    return Err(fail(format!(
                        "tab {:?} pinned={} but sits at index {i} with pinned_count {}",
                        tid, tab.pinned, space.pinned_count
                    )));
                }
            }
            if let Some(active_tab) = space.active_tab {
                if !space.tabs.contains(&active_tab) {
                    return Err(fail(format!(
                        "space {:?} active tab {:?} is not a member",
                        space.id, active_tab
                    )));
                }
            }
            for tid in &space.tabs {
                let tab = self.tab(*tid).expect("membership checked above");
                let visible = space.split.is_some_and(|split| split.contains(*tid))
                    || (space.split.is_none() && space.active_tab == Some(*tid));
                // A visible tab is either actively served (Active) or is a
                // crashed/recoverable entry (Discarded); it must never be
                // Suspended or Warm while shown. A hidden tab must never be
                // Active. Discarded hidden tabs are valid (recoverable).
                let agreement = if visible {
                    matches!(
                        tab.lifecycle,
                        TabLifecycle::Active | TabLifecycle::Discarded
                    )
                } else {
                    tab.lifecycle != TabLifecycle::Active
                };
                if !agreement {
                    return Err(fail(format!(
                        "tab {:?} lifecycle {:?} disagrees with visible={visible}",
                        tid, tab.lifecycle
                    )));
                }
            }
            if let Some(split) = &space.split {
                if !matches!(space.active_tab, Some(active) if split.contains(active)) {
                    return Err(fail(format!(
                        "space {:?} active tab {:?} is outside its split",
                        space.id, space.active_tab
                    )));
                }
                if split.primary == split.secondary {
                    return Err(fail(format!(
                        "space {:?} split panes are identical",
                        space.id
                    )));
                }
                for pane in [split.primary, split.secondary] {
                    if !space.tabs.contains(&pane) {
                        return Err(fail(format!(
                            "space {:?} split pane {:?} is not a member tab",
                            space.id, pane
                        )));
                    }
                }
            }
            for closed in &space.recently_closed {
                if space.tabs.contains(&closed.tab.id) {
                    return Err(fail(format!(
                        "space {:?} recently closed tab {:?} is still a member",
                        space.id, closed.tab.id
                    )));
                }
                if closed.tab.space != space.id {
                    return Err(fail(format!(
                        "space {:?} recently closed tab {:?} claims space {:?}",
                        space.id, closed.tab.id, closed.tab.space
                    )));
                }
            }
        }

        for tab in &self.tabs {
            let Some(sp) = self.space(tab.space) else {
                return Err(fail(format!(
                    "tab {:?} references unknown space {:?}",
                    tab.id, tab.space
                )));
            };
            if !sp.tabs.contains(&tab.id) {
                return Err(fail(format!(
                    "tab {:?} is not ordered in its space {:?}",
                    tab.id, tab.space
                )));
            }
        }
        // Every id is unique across all tables (the counter guarantees this,
        // but a structural pass keeps the guarantee independent).
        let mut ids: std::collections::HashSet<(&str, u64)> =
            std::collections::HashSet::from_iter(self.profiles.iter().map(|p| ("p", p.id.0)));
        for space in &self.spaces {
            if !ids.insert(("s", space.id.0)) {
                return Err(fail(format!("duplicate space id {:?}", space.id)));
            }
        }
        for tab in &self.tabs {
            if !ids.insert(("t", tab.id.0)) {
                return Err(fail(format!("duplicate tab id {:?}", tab.id)));
            }
        }
        Ok(())
    }
}
