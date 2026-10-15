use fluxa_webos::diagnostics::Report;
use fluxa_webos::demux::FfmpegDemuxer;
use fluxa_webos::player::Demuxer;
use fluxa_webos::{LoadOptions, load_payload};

fn main() {
    let Some(arg) = std::env::args().nth(1) else {
        eprintln!("usage: probe <url|--report>");
        std::process::exit(2);
    };
    if arg == "--report" {
        let report = Report::collect();
        for (key, value) in &report.tags {
            println!("{key} = {value}");
        }
        for (key, value) in &report.extras {
            println!("--- {key}\n{value}");
        }
        return;
    }
    let mut demuxer = FfmpegDemuxer::default();
    let info = match demuxer.open(&arg) {
        Ok(info) => info,
        Err(error) => {
            eprintln!("open failed: {error}");
            std::process::exit(1);
        }
    };
    let options = LoadOptions {
        app_id: "com.fluxa.app".into(),
        webos_major: 5,
        start_pts_ns: 0,
        window_id: None,
        max_resolution: None,
    };
    println!("duration {:.3}s", info.duration_ns as f64 / 1e9);
    println!("{}", load_payload(&info.video, info.audio.as_ref(), &options));
}
