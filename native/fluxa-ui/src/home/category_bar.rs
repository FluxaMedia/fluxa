use super::*;
use crate::navigation::{nav_bar_padding, nav_bar_top, nav_item_height};

fn bar_visible(viewport: Viewport, home: &HomeModel) -> bool {
    home.category_bar && home.categories.len() > 1 && viewport.side_inset > 0.0
}

pub(crate) fn category_bar_space(viewport: Viewport, home: &HomeModel) -> f32 {
    if bar_visible(viewport, home) && !home_shows_hero(home) {
        nav_bar_top()
            + nav_item_height()
            + nav_bar_padding() * 2.0
            + Space::S8.px(&crate::metrics::active())
    } else {
        0.0
    }
}
