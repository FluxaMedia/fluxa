use super::*;
use fluxa_ui::localized;

#[derive(Default)]
pub(crate) struct State {
    data_dir: Option<std::path::PathBuf>,
    applied: Option<bool>,
}

const OPT_OUT_MARKER: &str = "crash-reports-off";

pub(crate) fn start(state: &mut RendererState, data_dir: &std::path::Path) {
    state.crash.data_dir = Some(data_dir.to_owned());
    #[cfg(not(target_arch = "wasm32"))]
    fluxa_crash::open_session_log(data_dir);
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

pub(crate) fn open_report(state: &mut RendererState) {
    host_log(format!(
        "problem report opened route={}",
        state.route.as_str()
    ));
    state.settings.report_open = true;
}

pub(crate) fn close_report(state: &mut RendererState) {
    state.settings.report_open = false;
}

fn notify(state: &mut RendererState, key: &str) {
    let language = state.settings.language().to_owned();
    state.notice = Some((localized(key, &language), Instant::now()));
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn submit_report(state: &mut RendererState) {
    let title = state.settings.report_title.trim().to_owned();
    let body = state.settings.report_body.trim().to_owned();
    if title.is_empty() || body.is_empty() {
        return;
    }
    if !fluxa_crash::reporting_available() {
        notify(state, "settings.about.report_unavailable");
        return;
    }
    let route = state.route.as_str();
    std::thread::spawn(move || {
        fluxa_crash::report_problem(&title, &body, &[("report.route", route)]);
    });
    state.settings.report_open = false;
    state.settings.report_title.clear();
    state.settings.report_body.clear();
    notify(state, "settings.about.report_sent");
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn submit_report(state: &mut RendererState) {
    notify(state, "settings.about.report_unavailable");
}
