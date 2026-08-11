//! The gallery story catalog — the single declaration point (packet
//! `component-gallery-mvp.md`). Groups and stories appear in explicit
//! declaration order; the registry assigns stable, append-only ids.
//!
//! Every story renders **real production symbols** with plain fixture
//! models — never a gallery-local reimplementation of anatomy, never a
//! copied token. The Foundations pages are the gallery's own presentation
//! of the `bbr-ui` token API (swatch tables, type rows, density tables);
//! the primitives are the real `bbr-ui` components; the Core Controls
//! stories render the exact vendored `gpui-component` 0.5.1 production
//! controls (Button, Input, Checkbox, Switch, Radio, Slider, Progress)
//! initialized through the production integration pattern.
//!
//! Scope copy: the gallery ships the **Foundations / Primitives / Core
//! Controls / Browser Components** groups and points to the later Gallery
//! phase — Compositions (G3) — it never claims to be a complete component
//! explorer.

pub mod address;
pub mod auth;
pub mod browser_components;
pub mod command;
pub mod context_menu;
pub mod core_controls;
pub mod downloads;
pub mod error;
pub mod favorites;
pub mod find;
pub mod folders;
pub mod footer;
pub mod foundations;
pub mod library_composition;
pub mod menu;
pub mod primitives;
pub mod profile_chip;
pub mod saved_pages_library;
pub mod sidebar;

use gpui::{div, prelude::*, px, AnyElement, Rgba, SharedString};

use crate::library_fixtures::{
    LIBRARY_REMOVE_STATES, LIBRARY_ROW_STATES, LIBRARY_SEARCH_STATES, LIBRARY_STATES,
};
use crate::registry::{library_story, story, CatalogRegistry};

/// The catalog scope line under the title (finding 3).
pub const GALLERY_SCOPE: &str = "Foundations / Primitives / Browser Components / Compositions";

/// The honest continuation note: G2 browser components are complete and
/// the G3 Sidebar + Command compositions are integrated; the remaining G3
/// compositions (Split and the later post-packet stories) arrive in later
/// Gallery phases (finding 3 — never a complete-explorer claim).
pub const GALLERY_CONTINUATION: &str =
    "Browser Components (G2) are complete; Compositions (G3) — Split and later composition stories — arrive in later Gallery phases.";
/// Build the G1b catalog: Foundations pages + the smallest valuable real
/// primitives + the Core Controls, in declaration order. The G2
/// browser-components group appends after the pinned G1b prefix — ids stay
/// append-only and never renumber; later G3 rows continue at later ids.
pub fn catalog() -> CatalogRegistry {
    let mut registry = CatalogRegistry::new();
    registry.add_group(
        "Foundations",
        vec![
            story("Tokens", foundations::render_tokens, &[]),
            story("Typography", foundations::render_typography, &[]),
            story("Spacing / Radius", foundations::render_spacing_radius, &[]),
            story("Motion", foundations::render_motion, &[]),
            story("Density", foundations::render_density, &[]),
            story("Elevation", foundations::render_elevation, &[]),
            story("Color", foundations::render_color, &[]),
            story(
                "States",
                foundations::render_states,
                &crate::fixtures::STATE_VOCABULARY,
            ),
            story("Iconography", foundations::render_iconography, &[]),
            story("Brand", foundations::render_brand, &[]),
        ],
    );
    registry.add_group(
        "Primitives",
        vec![
            story(
                "Favicon tile",
                primitives::render_favicon_tile,
                &crate::fixtures::FAVICON_STATES,
            ),
            story("Profile chip", primitives::render_profile_chip, &[]),
            story("Section label", primitives::render_section_label, &[]),
            story("Loading arc", primitives::render_loading_arc, &[]),
        ],
    );
    // The Core Controls stories (live-review finding 2): the established
    // vendored gpui-component 0.5.1 production controls the product stack
    // provides. Appended after the pinned G1b prefix — ids stay stable.
    registry.add_group(
        "Core controls",
        vec![
            story("Button", core_controls::render_button, &[]),
            story("Input", core_controls::render_input, &[]),
            story("Checkbox", core_controls::render_checkbox, &[]),
            story("Switch", core_controls::render_switch, &[]),
            story("Radio", core_controls::render_radio, &[]),
            story("Slider", core_controls::render_slider, &[]),
            story("Progress", core_controls::render_progress, &[]),
        ],
    );
    // The G2 browser-component stories (packet
    // `component-gallery-browser-components.md`): appended after the pinned
    // G1b prefix — ids stay stable and never renumber. Status Line is id 21,
    // the integrated production Tab Row is id 22, and B3's exact production
    // Saved Pages Library rows/controls append at ids 23–29.
    registry.add_group(
        "Browser components",
        vec![
            story("Status line", browser_components::render_status_line, &[]),
            story("Tab Row", browser_components::render_tab_row, &[]),
            library_story(
                "Library row",
                saved_pages_library::render_library_row,
                &LIBRARY_ROW_STATES,
            ),
            library_story(
                "Library collection row",
                saved_pages_library::render_library_collection_row,
                &[],
            ),
            library_story(
                "Library search",
                saved_pages_library::render_library_search,
                &LIBRARY_SEARCH_STATES,
            ),
            library_story(
                "Library sort",
                saved_pages_library::render_library_sort,
                &[],
            ),
            library_story(
                "Library count",
                saved_pages_library::render_library_count,
                &[],
            ),
            library_story(
                "Library remove confirm",
                saved_pages_library::render_library_remove_confirm,
                &LIBRARY_REMOVE_STATES,
            ),
            library_story(
                "Library state rows",
                saved_pages_library::render_library_state_rows,
                &[],
            ),
        ],
    );
    // The G3 composition stories (packet
    // `component-gallery-compositions.md`): the first composition is the
    // Saved Pages Library surface at id 30. Later G3 compositions append.
    registry.add_group(
        "Compositions",
        vec![library_story(
            "Library",
            library_composition::render_library,
            &LIBRARY_STATES,
        )],
    );
    // The G2 Group 3 Address/Input stories (packet
    // `component-gallery-browser-components.md`): the exact production
    // `address_control` pill and `staging_surface` staging families append
    // at ids 31–32 after the pinned 0–30 prefix — no renumbering, no
    // reservation.
    registry.add_group(
        "Address / Input",
        vec![
            story("Address control", address::render_address_control, &[]),
            story("Command staging", address::render_command_staging, &[]),
        ],
    );
    // Group 4 sidebar components append at ids 33–37.
    registry.add_group(
        "Sidebar components",
        vec![
            story("Favorites grid", favorites::render_favorites, &[]),
            story("Folders", folders::render_folders, &[]),
            story("Footer", footer::render_footer, &[]),
            story(
                "Profile chip (S/N/XL)",
                profile_chip::render_profile_chip,
                &[],
            ),
            story("Plus menu", menu::render_plus_menu, &[]),
        ],
    );
    // Group 5 Downloads appends at id 38.
    registry.add_group(
        "Downloads",
        vec![story(
            "Downloads panel",
            downloads::render_downloads_panel,
            &[],
        )],
    );
    // Group 6 Find appends at id 39.
    registry.add_group("Find", vec![story("Find bar", find::render_find_bar, &[])]);
    // Group 7 Error/Crash appends at ids 40–41.
    registry.add_group(
        "Error / Crash",
        vec![
            story("Error surface", error::render_error_surface, &[]),
            story("Crash / Empty panes", error::render_crash_empty_panes, &[]),
        ],
    );
    // The G2 Group 8 Auth sheet story (packet
    // `component-gallery-browser-components.md`): the exact production
    // `bbr_ui::surfaces::auth` credential sheet appends at id 42 after the
    // pinned 0–41 prefix — no renumbering, no reservation. Context Menu
    // appends later in its own group.
    registry.add_group(
        "Auth",
        vec![story("Auth sheet", auth::render_auth_sheet, &[])],
    );
    // The G2 Group 9 Context-menu story (packet
    // `component-gallery-browser-components.md`): the exact production
    // `bbr_ui::surfaces::menu::context_menu_card` renders every valid page-
    // menu family with recorder callbacks. Appended at canonical StoryId
    // 43 after the pinned 0–42 prefix — no renumbering or reservation.
    registry.add_group(
        "Context menu",
        vec![story(
            "Context menu",
            context_menu::render_context_menu,
            &[],
        )],
    );
    // The G3 Sidebar composition story (packet
    // `component-gallery-compositions.md`): the exact production
    // `bbr_ui::sidebar::sidebar_surface` renders the selected matrix family
    // with the one production `Callbacks` set filled by recorder callbacks.
    // Appended at canonical StoryId 44 after the pinned 0–43 prefix — no
    // renumbering or reservation. The remaining G3 compositions (Command,
    // Split) append at later ids in their own bounded cuts.
    registry.add_group(
        "Sidebar composition",
        vec![story(
            "Sidebar",
            sidebar::render_sidebar,
            sidebar::SIDEBAR_STATE_OPTIONS,
        )],
    );
    // The G3 Command composition story (packet
    // `component-gallery-compositions.md`): the exact production
    // `bbr_ui::surfaces::command::command_card` renders the selected matrix
    // family from the plain fixture model. The surface takes no callbacks —
    // activation stays app/Portal-owned, so the Gallery observes
    // presentation only. Appended at canonical StoryId 45 after the pinned
    // 0–44 prefix — no renumbering or reservation. Split composes after
    // Command in its own bounded cut.
    registry.add_group(
        "Command composition",
        vec![story(
            "Command palette / overlay",
            command::render_command,
            command::COMMAND_STATE_OPTIONS,
        )],
    );
    registry
}

/// A story page scaffold: a scrollable column with a `type:title` header
/// and the story content.
pub fn story_page(
    theme: &bbr_ui::theme::Theme,
    title: &str,
    children: Vec<AnyElement>,
) -> AnyElement {
    div()
        .id("bbr-gallery-story-page")
        .debug_selector(|| "bbr-gallery-story-page".to_string())
        .flex()
        .flex_col()
        .size_full()
        .min_w_0()
        .overflow_x_hidden()
        .overflow_y_scroll()
        .p_4()
        .gap_2()
        .child(
            div()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(theme.palette.ink)
                .child(SharedString::from(title)),
        )
        .children(children)
        .into_any()
}

/// A labeled color swatch: the token name, the rendered color tile, and
/// the hex value.
pub fn swatch(theme: &bbr_ui::theme::Theme, label: &str, color: Rgba) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .size(px(24.0))
                .rounded(theme.radius.small())
                .bg(color)
                .border_1()
                .border_color(theme.palette.hairline),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .font_weight(theme.type_.label.weight)
                        .text_color(theme.palette.ink)
                        .child(SharedString::from(label)),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_secondary)
                        .child(hex_of(color)),
                ),
        )
        .into_any()
}

/// A labeled value row: a name and a `type:mono` value. The value is
/// shrinkable (`min_w_0` + ellipsis) so a long value never forces the
/// story page wider than the window — the real-pointer E2E targets stay
/// within the clickable viewport.
pub fn value_row(theme: &bbr_ui::theme::Theme, name: &str, value: &str) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .justify_between()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_size(theme.type_.label.size)
                .text_color(theme.palette.ink)
                .child(SharedString::from(name)),
        )
        .child(
            div()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.mono.size)
                .text_color(theme.palette.ink_secondary)
                .child(SharedString::from(value)),
        )
        .into_any()
}

/// The hex string of an `Rgba` (the §15 display form).
pub fn hex_of(color: Rgba) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
    )
}

/// WCAG 2.x relative luminance of an `Rgba` (derived from the tokens, never
/// a copied ratio).
fn luminance(color: Rgba) -> f64 {
    let channel = |c: f32| {
        let v = f64::from(c);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

/// WCAG 2.x contrast ratio between two token colors.
pub fn contrast_ratio(a: Rgba, b: Rgba) -> f64 {
    let (hi, lo) = if luminance(a) >= luminance(b) {
        (luminance(a), luminance(b))
    } else {
        (luminance(b), luminance(a))
    };
    (hi + 0.05) / (lo + 0.05)
}

/// A `type:badge` section label inside a story (the gallery's own page
/// header, distinct from the real `bbr-ui` `section_label` primitive).
pub fn section_heading(theme: &bbr_ui::theme::Theme, text: &str) -> AnyElement {
    div()
        .mt_2()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(theme.palette.ink_secondary)
        .child(SharedString::from(text.to_uppercase()))
        .into_any()
}
