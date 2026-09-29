//! The G3 Sidebar composition story (packet
//! `component-gallery-compositions.md`): the **exact production
//! `bbr_ui::sidebar::sidebar_surface`** renders the selected matrix family
//! from the one matrix authority [`crate::sidebar_matrix`] with the one
//! production [`bbr_ui::sidebar::Callbacks`] set filled by recorder
//! callbacks proving which production callback fired. The story's state
//! selector switches the matrix family (one rail at a time, so every
//! production id is unique and the E2E can target any element directly);
//! the rejected combinations render as value rows with their reasons. The
//! three app-owned child slots (command field anchor, command results,
//! downloads panel) are **Gallery-owned placeholders at the same render
//! boundary** — never placed into the model, never faking the app's
//! surfaces. No gallery-local reimplementation of anatomy, no copied
//! tokens, no fake engine/controller/portal/ranking/persistence.

use bbr_ui::sidebar::sidebar_surface;
use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, SharedString};

use crate::registry::StoryContext;
use crate::sidebar_fixture::build_sidebar_fixture_with_theme;
use crate::sidebar_matrix::{sidebar_case_valid, SIDEBAR_REJECTED, SIDEBAR_VALID};
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector for the rendered rail
/// (`sidebar-family` — one family at a time via the story's state
/// selector). Test target only — never a product id.
pub const SIDEBAR_FAMILY_SELECTOR: &str = "sidebar-family";

/// The story's state options: the matrix family labels in declaration
/// order (the state selector switches the rendered family). Pinned by a
/// matrix test so the options can never drift from [`SIDEBAR_VALID`].
pub const SIDEBAR_STATE_OPTIONS: &[&str] = &[
    "N open rail — full composition",
    "S dense 500-tab windowed rail",
    "compact window — content floor clamp",
    "wide rail — full 400 pt",
    "collapsed rail — width 0",
    "empty rail — no tabs/favorites/folders",
    "long-title + missing-icon rows",
    "cross-profile — profile hint + strip identity",
    "plus menu open — selection 1",
    "command staging + results child",
    "downloads panel child + running count",
    "hover/pressed/focus/drag/key paths",
    "reduced-motion + increased contrast",
    "titlebar nav — full history + live session",
    "titlebar nav — no history, no live session",
    "favorites drop band — empty grid during tab drag",
];

/// §11.4 Sidebar: the production `bbr_ui::sidebar::sidebar_surface` renders
/// the selected matrix family with the one production `Callbacks` set
/// filled by recorder callbacks. The state selector switches the family;
/// the rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_sidebar(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let sidebar = ctx
        .sidebar
        .as_ref()
        .expect("Sidebar story renders with the recorder");
    let callbacks = sidebar.observer.callbacks();
    let handles = sidebar.handles;
    // The selected family (the story's state selector; a leftover state
    // from another story falls back to the first family).
    let case = SIDEBAR_VALID
        .iter()
        .find(|c| c.label == ctx.state)
        .unwrap_or(&SIDEBAR_VALID[0]);
    let family_theme = crate::sidebar_fixture::fixture_theme(case, theme);
    let model =
        build_sidebar_fixture_with_theme(case, handles, theme).expect("valid matrix family builds");
    let field_anchor = field_anchor_placeholder();
    let favorites_band_anchor = band_anchor_placeholder();
    let command_results = case
        .command_results
        .then(|| child_placeholder(theme, "command results (app-owned child)"));
    let downloads_panel = case
        .downloads_panel
        .then(|| child_placeholder(theme, "downloads panel (app-owned child)"));
    let rail = sidebar_surface(
        &family_theme,
        &model,
        &callbacks,
        bbr_ui::sidebar::SidebarChildren {
            command_field_anchor: field_anchor,
            favorites_band_anchor,
            // The ghost-row and proxy rect recorders are app-owned.
            pin_offer_anchor: None,
            folder_anchors: Vec::new(),
            drag_proxy_anchor: None,
            command_results,
            downloads_panel,
            // The §11.3 Space-configuration panel is an app-owned child;
            // the dedicated Space configuration story owns that surface.
            space_config_panel: None,
        },
    );

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `sidebar_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = SIDEBAR_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !sidebar_case_valid(&r.case),
                "rejected sidebar {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.4 sidebar rail (production bbr_ui::sidebar::sidebar_surface)"),
        value_row(
            theme,
            "ids",
            "bbr-sidebar · bbr-titlebar · bbr-titlebar-sidebar-toggle · bbr-nav-back · bbr-nav-forward · bbr-nav-reload · bbr-brand · bbr-address · bbr-favorites · bbr-tab-row-<n> · bbr-folder-row-<n> · bbr-status · bbr-footer · bbr-space-strip · bbr-plus · bbr-sidebar-divider",
        ),
        value_row(
            theme,
            "callbacks",
            "the one 28-slot Callbacks set (recorder) — every production listener, including the §11.14 titlebar row",
        ),
        value_row(
            theme,
            "children",
            "three app-owned child slots at the composition boundary: command field anchor (invisible), command results, downloads panel — Gallery-owned placeholders here, never in the model",
        ),
        value_row(theme, "family", case.label),
        value_row(
            theme,
            "effective width",
            &format!(
                "{:.0} pt (requested {:.0} in a {:.0} pt window)",
                model.sidebar_width, case.requested_width, case.window_width
            ),
        ),
        value_row(
            theme,
            "dense window",
            &format!(
                "{} pinned + {} window rows ({} regular tabs)",
                model.tabs.pinned.len(),
                model.tabs.window.len(),
                case.tab_count
            ),
        ),
        value_row(
            theme,
            "mode fields",
            &format!(
                "{} density · {} contrast · reduced motion {}",
                match family_theme.density {
                    bbr_ui::theme::Density::Small => "S",
                    bbr_ui::theme::Density::Normal => "N",
                    bbr_ui::theme::Density::ExtraLarge => "XL",
                },
                match family_theme.contrast {
                    bbr_ui::theme::ContrastMode::Standard => "standard",
                    bbr_ui::theme::ContrastMode::Increased => "increased",
                },
                if family_theme.motion_mode == bbr_ui::theme::MotionMode::Reduced { "on" } else { "off" },
            ),
        ),
    ];
    // The family viewport is tall enough to show the whole rail: the
    // production composition's fixed regions (brand, command slot,
    // favorites, pinned block, folders, status, toggle, footer) exceed
    // 560 pt, and the regular-region scroll view is `flex_1` — a short
    // wrapper would clip the rows below the scroll view's bottom edge and
    // the real-pointer E2E could not target them. The wrapper must also
    // keep the footer above the story page's scroll viewport bottom (the
    // page is 1080 pt tall in the test window), so 700 pt fits both the
    // rows and the footer. The story page scrolls, so the viewport height
    // costs nothing visually.
    children.push(
        div()
            .debug_selector(|| SIDEBAR_FAMILY_SELECTOR.to_string())
            .h(px(700.0))
            .child(rail)
            .into_any(),
    );
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(ctx, "Sidebar", children)
}

/// The Gallery-owned §11.4 field anchor placeholder: an invisible element
/// inside the command slot (the app's is an invisible rect-recorder
/// element; the gallery's placeholder occupies no layout space and paints
/// nothing — the slot is proven at the same render boundary).
fn field_anchor_placeholder() -> AnyElement {
    div()
        .id("bbr-gallery-sidebar-field-anchor")
        .size_0()
        .into_any()
}

/// The Gallery-owned §11.10 favorites-band anchor placeholder: an invisible
/// element inside the favorites region (the app's is an invisible
/// rect-recorder element; the gallery's placeholder occupies no layout
/// space and paints nothing — the band is proven at the same render
/// boundary).
fn band_anchor_placeholder() -> AnyElement {
    div()
        .id("bbr-gallery-sidebar-band-anchor")
        .size_0()
        .into_any()
}

/// A Gallery-owned child-slot placeholder (command results / downloads
/// panel): a clearly-labeled bordered box that marks the app-owned slot at
/// the composition boundary — never a fake of the app's surface.
fn child_placeholder(theme: &Theme, label: &str) -> AnyElement {
    div()
        .mx(Spacing::sp2())
        .my(Spacing::sp1())
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .rounded(theme.radius.small())
        .border_1()
        .border_color(theme.palette.hairline)
        .bg(theme.palette.surface_overlay)
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_secondary)
        .child(SharedString::from(label))
        .into_any()
}

use bbr_ui::theme::Spacing;
