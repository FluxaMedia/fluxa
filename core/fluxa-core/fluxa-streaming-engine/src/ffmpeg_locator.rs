use std::path::PathBuf;

fn platform_dir() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux-x64",
        ("macos", "x86_64") => "macos-x64",
        ("macos", "aarch64") => "macos-arm64",
        ("windows", "x86_64") => "windows-x64",
        _ => "unknown",
    }
}

fn bundled_path(name: &str) -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let exe_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    [
        exe_dir
            .join("resources/ffmpeg")
            .join(platform_dir())
            .join(&exe_name),
        // cargo run / cargo test layout: target/<profile>/ -> crate root/resources
        exe_dir
            .join("../../resources/ffmpeg")
            .join(platform_dir())
            .join(&exe_name),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

/// Resolves the ffmpeg/ffprobe binary to run: the bundled static build next to
/// the companion server executable if present, otherwise whatever's on PATH.
pub fn resolve(name: &str) -> PathBuf {
    bundled_path(name).unwrap_or_else(|| PathBuf::from(name))
}
