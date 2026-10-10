use super::*;
use fluxa_ui::{NODE_SETTINGS_UPDATE_ACTION, UpdateState};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
const RECHECK: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub(crate) struct State {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
    updater: Option<fluxa_updater::Updater>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
    last_check: Option<Instant>,
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
        Status::Downloading { percent, .. } => UpdateState::Downloading(percent),
        Status::Ready { version } => UpdateState::Ready(version),
        Status::Failed(reason) => UpdateState::Failed(reason),
    };
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android")))]
pub(crate) fn poll(_state: &mut RendererState) {}

pub(crate) fn busy(state: &RendererState) -> bool {
    matches!(
        state.settings.update.state,
        UpdateState::Checking | UpdateState::Downloading(_)
    )
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "android"))]
pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if node != NODE_SETTINGS_UPDATE_ACTION {
        return false;
    }
    let Some(updater) = state.updates.updater.clone() else {
        return true;
    };
    if matches!(state.settings.update.state, UpdateState::Ready(_)) {
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
    } else {
        state.updates.last_check = Some(Instant::now());
        updater.check(variant(state));
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
    node == NODE_SETTINGS_UPDATE_ACTION
}
