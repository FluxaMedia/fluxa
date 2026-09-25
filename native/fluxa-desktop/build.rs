use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(fluxa_nvdec_ffmpeg)");
    println!("cargo:rerun-if-changed=src/nvdec_ffmpeg.c");
    println!("cargo:rerun-if-changed=src/nvdec_ffmpeg.h");

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let Some(repository_root) = manifest.parent().and_then(|path| path.parent()) else {
        return;
    };
    let dot_env = repository_root.join("apps/desktop/.env");
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

    // NVDEC is an optional acceleration path. Keep the ordinary CPU decoder
    // buildable on machines without FFmpeg's development files or NVIDIA's
    // public CUDA ABI headers.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        return;
    }
    let Ok(ffnvcodec) = pkg_config::Config::new().probe("ffnvcodec") else {
        return;
    };
    let Ok(avcodec) = pkg_config::Config::new()
        .atleast_version("58.0")
        .probe("libavcodec")
    else {
        return;
    };
    let Ok(avutil) = pkg_config::Config::new().probe("libavutil") else {
        return;
    };
    let mut build = cc::Build::new();
    build
        .file(manifest.join("src/nvdec_ffmpeg.c"))
        .include(manifest.join("src"))
        .warnings(true)
        .flag_if_supported("-std=c11");
    for path in avcodec
        .include_paths
        .iter()
        .chain(avutil.include_paths.iter())
        .chain(ffnvcodec.include_paths.iter())
    {
        build.include(path);
    }
    build.compile("fluxa_nvdec_ffmpeg");
    println!("cargo:rustc-cfg=fluxa_nvdec_ffmpeg");
    println!("cargo:rustc-link-lib=dl");
}
