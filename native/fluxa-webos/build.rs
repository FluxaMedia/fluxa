fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("armv7") && target.contains("linux") {
        cc::Build::new().file("shim/getauxval.c").compile("getauxval_shim");
    }
}
