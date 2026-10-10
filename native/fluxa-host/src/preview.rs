use super::*;

pub(crate) fn start(state: &mut RendererState) {
    if !cfg!(debug_assertions) {
        return;
    }
    let Ok(target) = std::env::var("FLUXA_PREVIEW") else {
        return;
    };
    let (route, page) = target.split_once('/').unwrap_or((&target, ""));
    state.pending_native_actions.push(NativeAction::Navigate {
        destination: Route::parse(route),
    });
    if let Ok(page) = page.parse() {
        state.settings.active_section = page;
        state.settings.section_open = true;
        state.settings.page_open = true;
    }
}
