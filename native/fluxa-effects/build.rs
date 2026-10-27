use std::path::PathBuf;

const KEYS: [&str; 11] = [
    "FLUXA_NUVIO_SUPABASE_URL",
    "FLUXA_NUVIO_SUPABASE_KEY",
    "FLUXA_SYNC_URL",
    "FLUXA_SYNC_KEY",
    "FLUXA_TRAKT_CLIENT_ID",
    "FLUXA_TRAKT_CLIENT_SECRET",
    "FLUXA_SIMKL_CLIENT_ID",
    "FLUXA_SIMKL_CLIENT_SECRET",
    "FLUXA_MDBLIST_CLIENT_ID",
    "FLUXA_ANILIST_CLIENT_ID",
    "FLUXA_ANILIST_CLIENT_SECRET",
];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let Some(repository_root) = manifest.parent().and_then(|path| path.parent()) else {
        return;
    };
    let dot_env = repository_root.join(".env");
    println!("cargo:rerun-if-changed={}", dot_env.display());
    for key in KEYS {
        println!("cargo:rerun-if-env-changed={key}");
    }
    let content = std::fs::read_to_string(dot_env).unwrap_or_default();
    let file_values = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let key = key.trim();
            KEYS.contains(&key)
                .then(|| (key, value.trim().trim_matches('"').trim_matches('\'')))
        })
        .collect::<std::collections::HashMap<_, _>>();
    for key in KEYS {
        if let Ok(value) = std::env::var(key) {
            println!("cargo:rustc-env={key}={value}");
        } else if let Some(value) = file_values.get(key) {
            println!("cargo:rustc-env={key}={value}");
        }
    }
}
