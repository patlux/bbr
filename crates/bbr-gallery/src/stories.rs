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
pub mod browser_components;
pub mod core_controls;
pub mod find;
pub mod foundations;
pub mod library_composition;
pub mod primitives;
pub mod saved_pages_library;

use gpui::{div, prelude::*, px, AnyElement, Rgba, SharedString};

use crate::library_fixtures::{
    LIBRARY_REMOVE_STATES, LIBRARY_ROW_STATES, LIBRARY_SEARCH_STATES, LIBRARY_STATES,
};
use crate::registry::{library_story, story, CatalogRegistry};

/// The catalog scope line under the title (finding 3).
pub const GALLERY_SCOPE: &str = "Foundations / Primitives / Browser Components / Compositions";

/// The honest continuation note: later Gallery phases bring the remaining
/// browser components and compositions (finding 3 — never a
/// complete-explorer claim).
pub const GALLERY_CONTINUATION: &str =
    "More Browser Components (G2) and Compositions (G3) — Sidebar, Command, Downloads, Find, Error, Auth, Menu, Split — arrive in later Gallery phases.";

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
    // The G2 Group 6 Find story (packet
    // `component-gallery-browser-components.md`): the exact production
    // `bbr-ui::surfaces::find` bar renders every valid matrix family from
    // the one matrix authority with recorder callbacks. Appended after
    // the pinned prefix — ids stay append-only; the Orchestrator merges
    // groups 4–9 in canonical order.
    registry.add_group("Find", vec![story("Find bar", find::render_find_bar, &[])]);
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
        .flex()
        .flex_col()
        .size_full()
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

/// A labeled value row: a name and a `type:mono` value.
pub fn value_row(theme: &bbr_ui::theme::Theme, name: &str, value: &str) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .justify_between()
        .items_center()
        .child(
            div()
                .text_size(theme.type_.label.size)
                .text_color(theme.palette.ink)
                .child(SharedString::from(name)),
        )
        .child(
            div()
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
