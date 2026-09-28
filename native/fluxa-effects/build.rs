use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let Some(repository_root) = manifest.parent().and_then(|path| path.parent()) else {
        return;
    };
    let dot_env = repository_root.join(".env");
    println!("cargo:rerun-if-changed={}", dot_env.display());
    for key in ["FLUXA_NUVIO_SUPABASE_URL", "FLUXA_NUVIO_SUPABASE_KEY"] {
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
            ["FLUXA_NUVIO_SUPABASE_URL", "FLUXA_NUVIO_SUPABASE_KEY"]
                .contains(&key)
                .then(|| (key, value.trim().trim_matches('"').trim_matches('\'')))
        })
        .collect::<std::collections::HashMap<_, _>>();
    for key in ["FLUXA_NUVIO_SUPABASE_URL", "FLUXA_NUVIO_SUPABASE_KEY"] {
        if let Ok(value) = std::env::var(key) {
            println!("cargo:rustc-env={key}={value}");
        } else if let Some(value) = file_values.get(key) {
            println!("cargo:rustc-env={key}={value}");
        }
    }
}
