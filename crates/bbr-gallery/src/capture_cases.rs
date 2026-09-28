//! Capture case identity derives from the live registry, never a second catalog.
use bbr_ui::theme::{ContrastMode, Density, ThemeMode};

use crate::registry::{CatalogRegistry, StoryId};

#[derive(Clone, Debug)]
pub struct CaptureCase {
    pub story: StoryId,
    pub state: usize,
    pub state_label: &'static str,
    pub label: &'static str,
    pub mode: ThemeMode,
    pub contrast: ContrastMode,
    pub density: Density,
}

impl CaptureCase {
    /// Static endpoint policy for finite animations that remain under Reduce
    /// Motion. The IDs are append-only registry identities. Loops use their
    /// production Reduced Motion counterpart; this does not prove timing.
    pub fn settling_duration(&self) -> std::time::Duration {
        match self.story.0 {
            22 => {
                bbr_ui::theme::motion::Moment::AttentionPulse
                    .plan(bbr_ui::theme::MotionMode::Reduced)
                    .expect("attention retains its opacity pulse")
                    .duration
            }
            45 => bbr_ui::theme::Motion::fine(),
            _ => std::time::Duration::ZERO,
        }
    }

    pub fn id(&self) -> String {
        format!(
            "gallery-{:02}-state-{:02}-{}-{}--{}",
            self.story.0,
            self.state,
            match self.contrast {
                ContrastMode::Standard => "standard",
                ContrastMode::Increased => "increased",
            },
            match self.density {
                Density::Small => "small",
                Density::Normal => "normal",
                Density::ExtraLarge => "extralarge",
            },
            match self.mode {
                ThemeMode::Light => "light",
                ThemeMode::Dark => "dark",
            }
        )
    }
}

pub fn cases(registry: &CatalogRegistry) -> Vec<CaptureCase> {
    registry
        .stories
        .iter()
        .flat_map(|story| {
            let states = if story.state_options.is_empty() {
                &["default"][..]
            } else {
                story.state_options
            };
            states.iter().enumerate().flat_map(move |(state, label)| {
                [ContrastMode::Standard, ContrastMode::Increased]
                    .into_iter()
                    .flat_map(move |contrast| {
                        [Density::Small, Density::Normal, Density::ExtraLarge]
                            .into_iter()
                            .flat_map(move |density| {
                                [ThemeMode::Light, ThemeMode::Dark]
                                    .into_iter()
                                    .map(move |mode| CaptureCase {
                                        story: story.id,
                                        state,
                                        state_label: label,
                                        label: story.label,
                                        mode,
                                        contrast,
                                        density,
                                    })
                            })
                    })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_registry_state_has_all_twelve_appearance_cases() {
        let registry = crate::stories::catalog();
        let cases = cases(&registry);
        let ids: HashSet<_> = cases.iter().map(CaptureCase::id).collect();
        assert_eq!(ids.len(), cases.len());
        for story in &registry.stories {
            let own: Vec<_> = cases.iter().filter(|c| c.story == story.id).collect();
            assert_eq!(own.len(), story.state_options.len().max(1) * 12);
            for state in 0..story.state_options.len().max(1) {
                assert_eq!(own.iter().filter(|c| c.state == state).count(), 12);
            }
        }
    }

    #[test]
    fn appended_stories_cannot_silently_disappear() {
        let mut registry = crate::stories::catalog();
        let before = cases(&registry).len();
        registry.add_group(
            "New",
            vec![crate::registry::story(
                "New story",
                |ctx| crate::stories::story_page(ctx, "New", Vec::new()),
                &["empty", "populated"],
            )],
        );
        assert_eq!(cases(&registry).len(), before + 24);
    }
}
