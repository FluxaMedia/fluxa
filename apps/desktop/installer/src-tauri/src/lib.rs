mod webos;

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    step: &'static str,
    message: String,
}

fn report(app: &AppHandle, step: &'static str, message: impl Into<String>) {
    let _ = app.emit(
        "install-progress",
        Progress {
            step,
            message: message.into(),
        },
    );
}

#[tauri::command]
async fn discover_tvs() -> Vec<webos::DiscoveredTv> {
    webos::discover(Duration::from_secs(3)).await
}

#[tauri::command]
async fn install_ipk(
    app: AppHandle,
    host: String,
    passphrase: String,
    ipk_path: String,
    app_id: String,
    launch: bool,
) -> Result<String, String> {
    run_install(&app, &host, &passphrase, &ipk_path, &app_id, launch)
        .await
        .map_err(|err| err.to_string())
}

async fn run_install(
    app: &AppHandle,
    host: &str,
    passphrase: &str,
    ipk_path: &str,
    app_id: &str,
    launch: bool,
) -> Result<String, webos::InstallError> {
    let bytes = tokio::fs::read(ipk_path)
        .await
        .map_err(|err| webos::InstallError::Other(format!("could not read {ipk_path}: {err}")))?;
    let file_name = std::path::Path::new(ipk_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "app.ipk".to_string());

    report(app, "key", "Asking the TV for its developer key");
    let key = webos::fetch_developer_key(host).await?;

    report(app, "connect", "Connecting over SSH");
    let session = webos::Session::connect(host, passphrase, &key).await?;

    report(app, "upload", format!("Uploading {file_name}"));
    session.exec(&webos::ensure_temp_dir_command()).await?;
    let remote = webos::remote_ipk_path(&file_name);
    session.upload(&bytes, &remote).await?;

    report(app, "install", "Installing on the TV");
    let log = session.exec(&webos::install_command(&remote)).await?;
    if let Some(reason) = webos::install_failed_reason(&log) {
        return Err(webos::InstallError::Other(reason));
    }

    let _ = session.exec(&format!("/bin/rm -f {remote}")).await;

    if launch {
        report(app, "launch", "Launching Fluxa");
        let _ = session.exec(&webos::launch_command(app_id)).await;
    }

    report(app, "done", "Installed");
    Ok(log)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![discover_tvs, install_ipk])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
