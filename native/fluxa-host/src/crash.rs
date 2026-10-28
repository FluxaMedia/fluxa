use super::*;

#[derive(Default)]
pub(crate) struct State {
    data_dir: Option<std::path::PathBuf>,
    applied: Option<bool>,
}

const OPT_OUT_MARKER: &str = "crash-reports-off";

pub(crate) fn start(state: &mut RendererState, data_dir: &std::path::Path) {
    state.crash.data_dir = Some(data_dir.to_owned());
    #[cfg(not(target_arch = "wasm32"))]
    if !data_dir.join(OPT_OUT_MARKER).exists() {
        fluxa_crash::configure(true, data_dir, "");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn poll(state: &mut RendererState) {
    let Some(data_dir) = state.crash.data_dir.clone() else {
        return;
    };
    if !state.settings.values.is_object() {
        return;
    }
    let enabled = state
        .settings
        .values
        .get("crashReports")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if state.crash.applied == Some(enabled) {
        return;
    }
    state.crash.applied = Some(enabled);
    let marker = data_dir.join(OPT_OUT_MARKER);
    if enabled {
        let _ = std::fs::remove_file(&marker);
    } else {
        let _ = std::fs::write(&marker, b"");
    }
    let variant = if state.home.form_factor == fluxa_ui::UiFormFactorJson::Tv {
        "tv"
    } else {
        ""
    };
    fluxa_crash::configure(enabled, &data_dir, variant);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn poll(_state: &mut RendererState) {}
