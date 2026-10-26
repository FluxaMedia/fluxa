fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !(target.starts_with("armv7") && target.contains("linux")) {
        return;
    }
    cc::Build::new()
        .file("shim/getauxval.c")
        .compile("getauxval_shim");
    if std::env::var_os("CARGO_FEATURE_FFMPEG_STATIC").is_some() {
        println!("cargo:rerun-if-env-changed=WEBOS_SYSROOT");
        if let Ok(sysroot) = std::env::var("WEBOS_SYSROOT") {
            println!("cargo:rustc-link-search=native={sysroot}/usr/lib");
        }
        println!("cargo:rustc-link-arg-bins=-Wl,-Bstatic");
        println!("cargo:rustc-link-arg-bins=-lssl");
        println!("cargo:rustc-link-arg-bins=-lcrypto");
        println!("cargo:rustc-link-arg-bins=-Wl,-Bdynamic");
        println!("cargo:rustc-link-arg-bins=-l:libc_nonshared.a");
        println!("cargo:rustc-link-arg-bins=-lz");
    }
}
