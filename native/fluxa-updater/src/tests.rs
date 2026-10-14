use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::time::{Duration, Instant};

fn archive() -> Vec<u8> {
    let mut tar = tar::Builder::new(Vec::new());
    let body = b"new binary";
    let mut header = tar::Header::new_gnu();
    header.set_size(body.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append_data(&mut header, "fluxa/fluxa-desktop", &body[..])
        .unwrap();
    let raw = tar.into_inner().unwrap();
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    gz.write_all(&raw).unwrap();
    gz.finish().unwrap()
}

fn wait_for(updater: &Updater, done: impl Fn(&Status) -> bool) -> Status {
    let started = Instant::now();
    loop {
        let status = updater.status();
        if done(&status) || started.elapsed() > Duration::from_secs(20) {
            return status;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fluxa-updater-e2e-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

struct Fixture {
    updater: Updater,
    dir: PathBuf,
}

fn fixture(name: &str, version: &str, tamper_asset: bool) -> Fixture {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let asset = archive();
    let sha: String = Sha256::digest(&asset)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let listener_probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener_probe.local_addr().unwrap().port();
    drop(listener_probe);
    let base = format!("http://127.0.0.1:{port}");
    let key_name = platform("");
    let manifest = serde_json::json!({
        "version": version,
        "notes": "notes",
        "assets": { key_name: { "url": format!("{base}/fluxa-{version}.tar.gz"), "sha256": sha, "size": asset.len() } }
    })
    .to_string();
    let signature = STANDARD.encode(key.sign(manifest.as_bytes()).to_bytes());
    let served_asset = if tamper_asset {
        vec![0u8; asset.len()]
    } else {
        asset
    };
    let files = vec![
        ("/latest.json".to_owned(), manifest.into_bytes()),
        ("/latest.json.sig".to_owned(), signature.into_bytes()),
        (format!("/fluxa-{version}.tar.gz"), served_asset),
    ];
    let served = serve_on(port, files);
    let dir = scratch(name);
    let updater = Updater::with_source(
        &dir,
        Source {
            manifest_url: format!("{served}/latest.json"),
            public_key: STANDARD.encode(key.verifying_key().to_bytes()),
        },
    );
    Fixture { updater, dir }
}

fn serve_on(port: u16, files: Vec<(String, Vec<u8>)>) -> String {
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut request = [0u8; 2048];
            let read = stream.read(&mut request).unwrap_or(0);
            let head = String::from_utf8_lossy(&request[..read]).into_owned();
            let path = head.split_whitespace().nth(1).unwrap_or("/").to_owned();
            match files.iter().find(|(name, _)| *name == path) {
                Some((_, body)) => {
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(body);
                }
                None => {
                    let _ = stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                }
            }
        }
    });
    base
}

#[test]
fn newer_release_is_downloaded_verified_and_staged() {
    let Fixture { updater, dir } = fixture("ok", "9.9.9", false);
    updater.check("");
    let found = wait_for(&updater, |status| {
        matches!(status, Status::Available { .. } | Status::Failed(_))
    });
    assert_eq!(
        found,
        Status::Available {
            version: "9.9.9".to_owned()
        }
    );
    updater.download();
    let status = wait_for(&updater, |status| {
        matches!(status, Status::Ready { .. } | Status::Failed(_))
    });
    assert_eq!(
        status,
        Status::Ready {
            version: "9.9.9".to_owned()
        }
    );
    let staged = fs::read_to_string(dir.join("updates/staged/fluxa/fluxa-desktop")).unwrap();
    assert_eq!(staged, "new binary");
    let reopened = Updater::new(&dir);
    assert_eq!(
        reopened.status(),
        Status::Ready {
            version: "9.9.9".to_owned()
        }
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn corrupted_download_is_rejected() {
    let Fixture { updater, dir } = fixture("corrupt", "9.9.9", true);
    updater.check("");
    let found = wait_for(&updater, |status| {
        matches!(status, Status::Available { .. } | Status::Failed(_))
    });
    assert_eq!(
        found,
        Status::Available {
            version: "9.9.9".to_owned()
        }
    );
    updater.download();
    let status = wait_for(&updater, |status| {
        matches!(status, Status::Ready { .. } | Status::Failed(_))
    });
    assert!(matches!(status, Status::Failed(_)), "{status:?}");
    assert!(!dir.join("updates/staged").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn same_version_is_reported_up_to_date() {
    let Fixture { updater, dir } = fixture("same", CURRENT_VERSION, false);
    updater.check("");
    let status = wait_for(&updater, |status| {
        matches!(status, Status::UpToDate | Status::Failed(_))
    });
    assert_eq!(status, Status::UpToDate);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn manifest_built_by_the_release_script_is_accepted() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/update-manifest.mjs");
    let Ok(keys) = std::process::Command::new("node")
        .arg(&script)
        .arg("keygen")
        .output()
    else {
        return;
    };
    let keys = String::from_utf8(keys.stdout).unwrap();
    let key = |name: &str| {
        keys.lines()
            .find_map(|line| line.strip_prefix(name))
            .unwrap()
            .to_owned()
    };
    let dir = scratch("script");
    let asset = dir.join("fluxa-9.9.9-linux-x86_64.tar.gz");
    fs::write(&asset, archive()).unwrap();
    let out = dir.join("latest.json");
    let built = std::process::Command::new("node")
        .arg(&script)
        .args([
            "build",
            "--version",
            "v9.9.9",
            "--repo",
            "FluxaMedia/fluxa",
            "--asset",
        ])
        .arg(format!("linux-x86_64={}", asset.display()))
        .arg("--out")
        .arg(&out)
        .env("FLUXA_UPDATE_PRIVATE_KEY", key("FLUXA_UPDATE_PRIVATE_KEY="))
        .status()
        .unwrap();
    assert!(built.success());
    let release = update::check(
        &fs::read(&out).unwrap(),
        &fs::read_to_string(dir.join("latest.json.sig")).unwrap(),
        &key("FLUXA_UPDATE_PUBLIC_KEY="),
        CURRENT_VERSION,
        "linux-x86_64",
    )
    .unwrap()
    .unwrap();
    assert_eq!(release.version, "9.9.9");
    assert!(
        release
            .asset
            .url
            .ends_with("/v9.9.9/fluxa-9.9.9-linux-x86_64.tar.gz")
    );
    let _ = fs::remove_dir_all(dir);
}
