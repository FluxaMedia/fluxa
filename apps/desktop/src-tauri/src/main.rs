#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--fluxa-thumbnail-helper") {
        if let Err(error) = fluxa_desktop_lib::run_thumbnail_helper() {
            eprintln!("thumbnail helper failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    fluxa_desktop_lib::run();
}
