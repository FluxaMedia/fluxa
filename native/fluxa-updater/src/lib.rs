mod install;
mod rollback;

pub use install::Applied;

use fluxa_core::update;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const HEALTHY_AFTER: Duration = Duration::from_secs(30);

const MANIFEST_URL: &str =
    "https://github.com/FluxaMedia/fluxa/releases/latest/download/latest.json";
const PUBLIC_KEY: &str = include_str!("../../../shared/update-public-key.txt");

#[derive(Clone)]
struct Source {
    manifest_url: String,
    public_key: String,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            manifest_url: MANIFEST_URL.to_owned(),
            public_key: PUBLIC_KEY.to_owned(),
        }
    }
}

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Checking,
    UpToDate,
    Available { version: String },
    Downloading { version: String, percent: u8 },
    Ready { version: String },
    Failed(String),
}

#[derive(Serialize, Deserialize)]
struct Staged {
    version: String,
}

#[derive(Clone)]
pub struct Updater {
    data_dir: PathBuf,
    dir: PathBuf,
    started: Instant,
    confirmed: Arc<AtomicBool>,
    source: Source,
    status: Arc<Mutex<Status>>,
    release: Arc<Mutex<Option<update::Release>>>,
    cancelled: Arc<AtomicBool>,
}

impl Updater {
    pub fn new(data_dir: &Path) -> Self {
        Self::with_source(data_dir, Source::default())
    }

    fn with_source(data_dir: &Path, source: Source) -> Self {
        let dir = data_dir.join("updates");
        rollback::on_start(data_dir);
        let status = match staged_version(&dir) {
            Some(version) => Status::Ready { version },
            None => {
                clear_stale(&dir);
                Status::Idle
            }
        };
        Self {
            data_dir: data_dir.to_owned(),
            started: Instant::now(),
            confirmed: Arc::default(),
            dir,
            source,
            status: Arc::new(Mutex::new(status)),
            release: Arc::default(),
            cancelled: Arc::default(),
        }
    }

    pub fn status(&self) -> Status {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or(Status::Idle)
    }

    pub fn check(&self, variant: &str) {
        {
            let Ok(mut status) = self.status.lock() else {
                return;
            };
            if matches!(
                *status,
                Status::Checking | Status::Downloading { .. } | Status::Ready { .. }
            ) {
                return;
            }
            *status = Status::Checking;
        }
        let source = self.source.clone();
        let status = self.status.clone();
        let found = self.release.clone();
        let variant = variant.to_owned();
        let skipped = rollback::skipped(&self.data_dir);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| find(&source, &variant, skipped.as_deref()))
                .unwrap_or_else(|_| Err("update check crashed".to_owned()));
            let outcome = match result {
                Ok(Some(release)) => {
                    let version = release.version.clone();
                    if let Ok(mut found) = found.lock() {
                        *found = Some(release);
                    }
                    Status::Available { version }
                }
                Ok(None) => Status::UpToDate,
                Err(error) => Status::Failed(error),
            };
            set(&status, outcome);
        });
    }

    pub fn download(&self) {
        let Some(release) = self.release.lock().ok().and_then(|release| release.clone()) else {
            return;
        };
        {
            let Ok(mut status) = self.status.lock() else {
                return;
            };
            if !matches!(*status, Status::Available { .. }) {
                return;
            }
            *status = Status::Downloading {
                version: release.version.clone(),
                percent: 0,
            };
        }
        self.cancelled.store(false, Ordering::Relaxed);
        let dir = self.dir.clone();
        let status = self.status.clone();
        let cancelled = self.cancelled.clone();
        std::thread::spawn(move || {
            let outcome = match stage(&dir, &release, &status, &cancelled) {
                Ok(ready) => ready,
                Err(_) if cancelled.load(Ordering::Relaxed) => Status::Available {
                    version: release.version.clone(),
                },
                Err(error) => Status::Failed(error),
            };
            set(&status, outcome);
        });
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn mandatory(&self) -> bool {
        self.release
            .lock()
            .ok()
            .and_then(|release| release.as_ref().map(|release| release.mandatory))
            .unwrap_or(false)
    }

    pub fn confirm_healthy(&self) {
        if self.started.elapsed() >= HEALTHY_AFTER && !self.confirmed.swap(true, Ordering::Relaxed)
        {
            rollback::confirm(&self.data_dir);
        }
    }

    pub fn apply(&self) -> Result<Applied, String> {
        let Status::Ready { version } = self.status() else {
            return Err("no update is ready".to_owned());
        };
        rollback::before_apply(&self.data_dir, &version);
        let applied = match install::apply(&self.dir.join("staged")) {
            Ok(applied) => applied,
            Err(error) => {
                rollback::abandon(&self.data_dir);
                return Err(error);
            }
        };
        if matches!(applied, Applied::Restart) {
            let _ = fs::remove_dir_all(&self.dir);
        }
        Ok(applied)
    }
}

fn platform(variant: &str) -> String {
    #[cfg(target_os = "android")]
    {
        let abi = match std::env::consts::ARCH {
            "aarch64" => "arm64-v8a",
            "arm" => "armeabi-v7a",
            other => other,
        };
        format!("android-{variant}-{abi}")
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = variant;
        let mut key = update::platform_key(std::env::consts::OS, std::env::consts::ARCH);
        if install::appimage_path().is_some() {
            key.push_str("-appimage");
        } else if let Some(kind) = install::system_package() {
            key.push('-');
            key.push_str(kind);
        }
        key
    }
}

fn clear_stale(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name() != "download" {
            let path = entry.path();
            let _ = fs::remove_dir_all(&path).or_else(|_| fs::remove_file(&path));
        }
    }
}

fn staged_version(dir: &Path) -> Option<String> {
    let staged: Staged = serde_json::from_slice(&fs::read(dir.join("ready.json")).ok()?).ok()?;
    let newer = update::compare_versions(&staged.version, CURRENT_VERSION).ok()?
        == std::cmp::Ordering::Greater;
    (newer && dir.join("staged").is_dir()).then_some(staged.version)
}

fn set(status: &Mutex<Status>, value: Status) {
    if let Ok(mut status) = status.lock() {
        *status = value;
    }
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .tls_certs_only(
            webpki_root_certs::TLS_SERVER_ROOT_CERTS
                .iter()
                .filter_map(|der| reqwest::Certificate::from_der(der).ok()),
        )
        .user_agent(concat!("fluxa/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(15))        .build()
        .map_err(|error| error.to_string())
}

fn find(
    source: &Source,
    variant: &str,
    skipped: Option<&str>,
) -> Result<Option<update::Release>, String> {
    let client = client()?;
    let manifest = fetch(&client, &source.manifest_url)?;
    let signature = fetch(&client, &format!("{}.sig", source.manifest_url))?;
    let signature = String::from_utf8_lossy(&signature).into_owned();
    let release = update::check(
        &manifest,
        &signature,
        &source.public_key,
        CURRENT_VERSION,
        &platform(variant),
    )
    .map_err(|error| error.to_string())?;
    Ok(release.filter(|release| skipped != Some(release.version.as_str())))
}

fn stage(
    dir: &Path,
    release: &update::Release,
    status: &Mutex<Status>,
    cancelled: &AtomicBool,
) -> Result<Status, String> {
    let client = client()?;
    install::check_writable()?;
    clear_stale(dir);
    let downloads = dir.join("download");
    fs::create_dir_all(&downloads).map_err(|error| error.to_string())?;
    let name = release.asset.url.rsplit('/').next().unwrap_or("update");
    let archive = downloads.join(name);
    let partial = downloads.join(format!("{name}.part"));
    download(&client, release, &partial, cancelled, |percent| {
        set(
            status,
            Status::Downloading {
                version: release.version.clone(),
                percent,
            },
        );
    })?;
    fs::rename(&partial, &archive).map_err(|error| error.to_string())?;

    let staged = dir.join("staged");
    install::extract(&archive, &staged)?;
    let _ = fs::remove_dir_all(&downloads);
    let marker = serde_json::to_vec(&Staged {
        version: release.version.clone(),
    })
    .map_err(|error| error.to_string())?;
    fs::write(dir.join("ready.json"), marker).map_err(|error| error.to_string())?;
    Ok(Status::Ready {
        version: release.version.clone(),
    })
}

fn fetch(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?;
    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| error.to_string())
}

fn download(
    client: &reqwest::blocking::Client,
    release: &update::Release,
    path: &Path,
    cancelled: &AtomicBool,
    progress: impl Fn(u8),
) -> Result<(), String> {
    let existing = fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    let resume = existing > 0 && existing < release.asset.size;
    let mut request = client.get(&release.asset.url);
    if resume {
        request = request.header(reqwest::header::RANGE, format!("bytes={existing}-"));
    }
    let mut response = request
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut written = 0u64;
    let mut file = if resume && response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
        let mut kept = fs::File::open(path).map_err(|error| error.to_string())?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = kept.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        written = existing;
        fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|error| error.to_string())?
    } else {
        fs::File::create(path).map_err(|error| error.to_string())?
    };
    let mut buffer = [0u8; 64 * 1024];
    let mut last = 255u8;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err("cancelled".to_owned());
        }
        let read = response
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        let Some(chunk) = buffer.get(..read).filter(|chunk| !chunk.is_empty()) else {
            break;
        };
        hasher.update(chunk);
        file.write_all(chunk).map_err(|error| error.to_string())?;
        written += read as u64;
        let percent = (written * 100 / release.asset.size.max(1)).min(100) as u8;
        if percent != last {
            last = percent;
            progress(percent);
        }
    }
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if written != release.asset.size || !update::sha256_matches(&digest, &release.asset) {
        let _ = fs::remove_file(path);
        return Err("downloaded update does not match the signed checksum".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
