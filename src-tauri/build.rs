fn main() {
    load_dot_env();
    copy_bundled_libmpv_files();
    tauri_build::build()
}

fn load_dot_env() {
    // Pass .env values to the crate via cargo:rustc-env.
    // CI secrets set as real env vars take priority over .env file.
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let dot_env = manifest.parent().unwrap_or(&manifest).join(".env");
    if let Ok(content) = std::fs::read_to_string(&dot_env) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                // If already set in the real environment (e.g. GitHub Actions secret), use that.
                let effective = std::env::var(key).unwrap_or_else(|_| val.to_string());
                println!("cargo:rustc-env={key}={effective}");
                println!("cargo:rerun-if-env-changed={key}");
            }
        }
    }
    println!("cargo:rerun-if-changed=../.env");
}

fn copy_bundled_libmpv_files() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(value) => std::path::PathBuf::from(value),
        Err(_) => return,
    };
    let source_dir = manifest_dir.join("lib");
    if !source_dir.exists() {
        return;
    }
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(value) => std::path::PathBuf::from(value),
        Err(_) => return,
    };
    let profile_dir = match out_dir.ancestors().find(|path| {
        path.file_name()
            .is_some_and(|name| name == "debug" || name == "release")
    }) {
        Some(path) => path.join("lib"),
        None => return,
    };
    if std::fs::create_dir_all(&profile_dir).is_err() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(source_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let target = profile_dir.join(entry.file_name());
            let _ = std::fs::copy(path, target);
        }
    }
}
