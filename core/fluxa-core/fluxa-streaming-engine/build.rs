use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=FLUXA_FFMPEG_BRIDGE_INCLUDE");
    println!("cargo:rerun-if-env-changed=FLUXA_FFMPEG_BRIDGE_LIB");

    let Ok(include_dir) = env::var("FLUXA_FFMPEG_BRIDGE_INCLUDE") else {
        return;
    };
    let Some(lib_dir) = env::var_os("FLUXA_FFMPEG_BRIDGE_LIB") else {
        return;
    };

    cc::Build::new()
        .file("bridge/fluxa_ffmpeg_remux.c")
        .include(&include_dir)
        .warnings(true)
        .compile("fluxa_ffmpeg_remux");

    println!("cargo:rustc-link-search=native={}", PathBuf::from(lib_dir).display());
    println!("cargo:rustc-link-lib=static=CFFmpeg");
}
