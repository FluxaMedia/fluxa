fn main() {
    #[cfg(target_os = "macos")]
    {
        match fluxa_desktop_lib::macos_player_surface::smoke_check_placement() {
            Ok(report) => println!("macos surface smoke: {report}"),
            Err(error) => {
                eprintln!("macos surface smoke FAILED: {error}");
                std::process::exit(1);
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    println!("macos surface smoke: skipped on this platform");
}
