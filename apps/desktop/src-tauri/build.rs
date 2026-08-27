fn main() {
    load_dot_env();
    build_macos_avplayer_bridge();
    copy_bundled_libmpv_files();
    tauri_build::build()
}

#[cfg(target_os = "macos")]
fn build_macos_avplayer_bridge() {
    use std::process::Command;

    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = manifest.join("../../../apps/apple/FluxaPlayerKit/Sources/FluxaPlayerKit");
    let mut sources = Vec::new();
    let mut pending = vec![source_dir.clone()];
    while let Some(path) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "swift") {
                println!("cargo:rerun-if-changed={}", path.display());
                sources.push(path);
            }
        }
    }
    sources.sort();
    if sources.is_empty() {
        panic!("FluxaPlayerKit Swift sources were not found");
    }

    let target = std::env::var("TARGET").unwrap_or_default();
    let swift_target = if target.starts_with("aarch64-") {
        "arm64-apple-macosx13.0"
    } else {
        "x86_64-apple-macosx13.0"
    };
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let profile_dir = out_dir
        .ancestors()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "debug" || name == "release")
        })
        .unwrap_or(out_dir.as_path())
        .to_path_buf();
    let lib_dir = profile_dir.join("lib");
    std::fs::create_dir_all(&lib_dir).expect("create Swift bridge library directory");
    let output = lib_dir.join("libFluxaDesktopPlayer.a");

    let mut command = Command::new("swiftc");
    command.args([
        "-parse-as-library",
        "-swift-version",
        "5",
        "-emit-library",
        "-static",
        "-module-name",
        "FluxaDesktopPlayer",
        "-target",
        swift_target,
        "-o",
    ]);
    command.arg(&output);
    command.args([
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "AVFoundation",
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "AppKit",
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "QuartzCore",
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "Combine",
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "WebKit",
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "MediaPlayer",
        "-Xlinker",
        "-framework",
        "-Xlinker",
        "Foundation",
    ]);
    command.args(sources);
    let status = command.status().expect("run swiftc for FluxaPlayerKit");
    if !status.success() {
        panic!("Swift AVPlayer bridge compilation failed");
    }
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=FluxaDesktopPlayer");
    let swiftc = std::env::var("SWIFT_EXEC").ok().or_else(|| {
        Command::new("xcrun")
            .args(["--find", "swiftc"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    });
    if let Some(swiftc) = swiftc {
        for swift_runtime in swift_runtime_paths(&swiftc) {
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,{}",
                swift_runtime.display()
            );
        }
    }
    for framework in [
        "AVFoundation",
        "AppKit",
        "QuartzCore",
        "Combine",
        "WebKit",
        "MediaPlayer",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}

#[cfg(target_os = "macos")]
fn swift_runtime_paths(swiftc: &str) -> Vec<std::path::PathBuf> {
    let Ok(output) = std::process::Command::new(swiftc)
        .args(["-print-target-info"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let Some(start) = text.find("\"runtimeLibraryPaths\"") else {
        return Vec::new();
    };
    let Some(end) = text[start..].find(']') else {
        return Vec::new();
    };
    let section = &text[start..start + end];
    section
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|path| path.starts_with('/'))
        .map(std::path::PathBuf::from)
        .collect()
}

#[cfg(not(target_os = "macos"))]
fn build_macos_avplayer_bridge() {}

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
