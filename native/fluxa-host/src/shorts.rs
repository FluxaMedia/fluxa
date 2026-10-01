use fluxa_ui::HomeHero;

use crate::{RendererState, Route};

pub(crate) fn tick(state: &mut RendererState) {
    if state.route != Route::Shorts {
        if !state.shorts.items.is_empty() {
            state.shorts = Default::default();
        }
        return;
    }
    if state.shorts.items.is_empty() {
        state.shorts.items = fluxa_ui::shorts_feed(&state.home);
        state.shorts.index = 0;
    }
    state.shorts.loading = state.home.is_loading;
    state.shorts.language = state.home.language.clone();
}

pub(crate) fn step(state: &mut RendererState, delta: i32) {
    let last = state.shorts.items.len().saturating_sub(1) as i32;
    state.shorts.index = (state.shorts.index as i32 + delta).clamp(0, last) as usize;
}

pub(crate) fn current(state: &RendererState) -> Option<&HomeHero> {
    state.shorts.items.get(state.shorts.index)
}
