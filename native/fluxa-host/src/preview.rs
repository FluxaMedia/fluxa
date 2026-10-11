use super::*;

fn setting(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|_| cfg!(debug_assertions))
}

pub fn size() -> Option<(f64, f64)> {
    let size = setting("FLUXA_PREVIEW_SIZE")?;
    let (width, height) = size.split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

pub(crate) fn form_factor(requested: &str) -> String {
    setting("FLUXA_PREVIEW_FORM").unwrap_or_else(|| requested.to_owned())
}

pub(crate) fn start(state: &mut RendererState) {
    let Some(target) = setting("FLUXA_PREVIEW") else {
        return;
    };
    let (route, page) = target.split_once('/').unwrap_or((&target, ""));
    let destination = Route::parse(route);
    if destination != Route::Home {
        state
            .pending_native_actions
            .push(NativeAction::Navigate { destination });
    }
    if let Some(scroll) = setting("FLUXA_PREVIEW_SCROLL").and_then(|value| value.parse().ok()) {
        state.screen_scroll_offsets.insert(destination, scroll);
    }
    if let Ok(page) = page.parse() {
        state.settings.active_section = page;
        state.settings.section_open = true;
        state.settings.page_open = true;
    }
}
