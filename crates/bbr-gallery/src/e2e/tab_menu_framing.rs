//! Prove fixture capacity against natural layout of the production card.
use std::{cell::RefCell, rc::Rc};

use bbr_ui::{
    surfaces::tab_menu::{tab_menu_card, TabMenuModel, TabMenuRect},
    theme::{ContrastMode, Density, Theme, ThemeMode},
};
use gpui::{div, point, prelude::*, px, size, AvailableSpace, Bounds, Context, Pixels, Window};

use crate::{
    harness::run_scenario,
    stories::tab_menu::fixture_rects,
    tab_menu_matrix::{build_tab_menu_fixture, TAB_MENU_VALID},
};

struct NaturalCard {
    theme: Theme,
    model: TabMenuModel,
    rect: TabMenuRect,
    sub: Option<TabMenuRect>,
    bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
}

impl Render for NaturalCard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let observed = self.bounds.clone();
        div()
            .flex()
            .flex_col()
            .items_start()
            .on_children_prepainted(move |b, _, _| *observed.borrow_mut() = b)
            .child(
                tab_menu_card(
                    &self.theme,
                    &self.model,
                    self.rect,
                    self.sub,
                    |_, _, _, _| {},
                    |_, _, _, _| {},
                    |_, _, _, _| {},
                    |_, _, _, _| {},
                )
                .relative()
                .left_0()
                .top_0()
                .h_auto()
                .flex_none(),
            )
    }
}

#[test]
#[ignore]
fn menu_fixture_fits_natural_production_rows_at_every_density() {
    run_scenario(|window, _| {
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let theme = Theme::new(ThemeMode::Light, ContrastMode::Standard, density);
            for case in TAB_MENU_VALID {
                let model = build_tab_menu_fixture(case).unwrap();
                let (rect, sub) = fixture_rects(&theme, &model);
                // Override only the test measurement card's positioning and
                // height to discover its natural row content. No production
                // row, font, padding or border is copied into this oracle.
                let bounds = Rc::new(RefCell::new(Vec::new()));
                let view = window.new(|_| NaturalCard {
                    theme,
                    model,
                    rect,
                    sub,
                    bounds: bounds.clone(),
                });
                window.draw(
                    point(px(0.0), px(0.0)),
                    size(
                        AvailableSpace::Definite(px(1200.0)),
                        AvailableSpace::MaxContent,
                    ),
                    |_, _| view.into_any_element(),
                );
                assert_eq!(bounds.borrow().len(), 1);
                let natural = bounds.borrow()[0].size.height;
                assert!(
                    natural <= px(rect.height),
                    "{} / {density:?}: natural {natural:?}, fixture {}",
                    case.label,
                    rect.height
                );
                if let Some(sub) = sub {
                    assert!(rect.x + sub.x + sub.width <= 480.0);
                    assert!(sub.y + sub.height <= rect.height + 48.0);
                }
            }
        }
    });
}
