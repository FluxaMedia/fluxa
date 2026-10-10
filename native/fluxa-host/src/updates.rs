use super::*;
use fluxa_ui::{NODE_SETTINGS_UPDATE_ACTION, NODE_UPDATE_INSTALL, NODE_UPDATE_LATER, UpdateState};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
const RECHECK: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub(crate) struct State {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
    updater: Option<fluxa_updater::Updater>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
    last_check: Option<Instant>,
    install_when_ready: bool,
    dismissed: bool,
    pub(crate) exit_requested: bool,
    pub(crate) install_apk: Option<String>,
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
pub(crate) fn start(state: &mut RendererState, data_dir: &std::path::Path) {
    state.updates.updater = Some(fluxa_updater::Updater::new(data_dir));
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
pub(crate) fn start(_state: &mut RendererState, _data_dir: &std::path::Path) {}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
pub(crate) fn poll(state: &mut RendererState) {
    use fluxa_updater::Status;
    state.settings.update.current = fluxa_updater::CURRENT_VERSION.to_owned();
    let Some(updater) = state.updates.updater.clone().filter(|_| !cfg!(debug_assertions)) else {
        return;
    };
    let automatic = state
        .settings
        .values
        .get("automaticUpdates")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let loaded = state.settings.values.is_object();
    let due = state
        .updates
        .last_check
        .is_none_or(|at| at.elapsed() >= RECHECK);
    if automatic && loaded && due {
        state.updates.last_check = Some(Instant::now());
        updater.check(variant(state));
    }
    state.settings.update.supported = true;
    state.settings.update.state = match updater.status() {
        Status::Idle => UpdateState::Idle,
        Status::Checking => UpdateState::Checking,
        Status::UpToDate => UpdateState::UpToDate,
        Status::Available { version } => UpdateState::Available(version),
        Status::Downloading { percent, .. } => UpdateState::Downloading(percent),
        Status::Ready { version } => UpdateState::Ready(version),
        Status::Failed(reason) => UpdateState::Failed(reason),
    };
    match state.settings.update.state {
        UpdateState::Ready(_) if state.updates.install_when_ready => {
            state.updates.install_when_ready = false;
            state.updates.dismissed = true;
            install(state, &updater);
        }
        UpdateState::Available(_) if state.updates.install_when_ready => updater.download(),
        _ => {}
    }
    state.settings.update.prompt = !state.updates.dismissed
        && !state.home.loading_screen()
        && match state.settings.update.state {
            UpdateState::Available(_) | UpdateState::Ready(_) => true,
            UpdateState::Downloading(_) | UpdateState::Failed(_) => {
                state.updates.install_when_ready
            }
            _ => false,
        };
    reveal(state);
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
fn reveal(state: &mut RendererState) {
    let Some(context) = state.gpu.as_ref().map(|gpu| gpu.egui_context.clone()) else {
        return;
    };
    let view = &mut state.settings.update;
    view.reveal = fluxa_ui::update_reveal(&context, view.prompt);
    view.top = if state.home.form_factor == fluxa_ui::UiFormFactorJson::Tv {
        state.safe_top.max(27.0)
    } else {
        state.safe_top
    };
    state.home.top_inset = view.reveal * fluxa_ui::update_banner_space(view.top);
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
fn install(state: &mut RendererState, updater: &fluxa_updater::Updater) {
    match updater.apply() {
        Ok(fluxa_updater::Applied::Restart) => state.updates.exit_requested = true,
        Ok(fluxa_updater::Applied::InstallApk(path)) => {
            state.updates.install_apk = Some(path.to_string_lossy().into_owned());
        }
        Err(error) => {
            host_log(format!("update could not be applied: {error}"));
            state.settings.update.state = UpdateState::Failed(error);
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
pub(crate) fn poll(_state: &mut RendererState) {}

pub(crate) fn revealing(state: &RendererState) -> bool {
    let view = &state.settings.update;
    view.reveal != if view.prompt { 1.0 } else { 0.0 }
}

pub(crate) fn busy(state: &RendererState) -> bool {
    matches!(
        state.settings.update.state,
        UpdateState::Checking | UpdateState::Downloading(_)
    )
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if node == NODE_UPDATE_LATER {
        state.updates.dismissed = true;
        state.updates.install_when_ready = false;
        if let Some(updater) = state.updates.updater.as_ref() {
            updater.cancel();
        }
        return true;
    }
    if node != NODE_SETTINGS_UPDATE_ACTION && node != NODE_UPDATE_INSTALL {
        return false;
    }
    let Some(updater) = state.updates.updater.clone() else {
        return true;
    };
    match state.settings.update.state {
        UpdateState::Ready(_) => install(state, &updater),
        UpdateState::Available(_) => {
            state.updates.install_when_ready = true;
            state.updates.dismissed = false;
            updater.download();
        }
        UpdateState::Downloading(_) => {}
        _ => {
            state.updates.install_when_ready = node == NODE_UPDATE_INSTALL;
            state.updates.last_check = Some(Instant::now());
            state.updates.dismissed = false;
            updater.check(variant(state));
        }
    }
    true
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
fn variant(state: &RendererState) -> &'static str {
    if cfg!(target_os = "android") && state.home.form_factor == fluxa_ui::UiFormFactorJson::Tv {
        "tv"
    } else if cfg!(target_os = "android") {
        "mobile"
    } else {
        ""
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
pub(crate) fn activate_node(_state: &mut RendererState, node: u64) -> bool {
    matches!(
        node,
        NODE_SETTINGS_UPDATE_ACTION | NODE_UPDATE_INSTALL | NODE_UPDATE_LATER
    )
}
