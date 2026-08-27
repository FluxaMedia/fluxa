use std::time::{Duration, Instant};

fn main() {
    let magnet = std::env::args().nth(1).unwrap_or_else(|| {
        "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny".to_string()
    });
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let cache = std::env::temp_dir().join(format!("fluxa-peer-probe-{unique}"));
    let Some(base) =
        fluxa_streaming_engine::start_torrent_server(cache.to_str().unwrap(), 0, "probe")
    else {
        eprintln!("engine did not start");
        std::process::exit(1);
    };
    println!("engine says {base}");
    let base: String = serde_json::from_str::<serde_json::Value>(&base)
        .ok()
        .and_then(|v| v.get("url").and_then(|u| u.as_str()).map(str::to_string))
        .unwrap_or(base);
    println!("engine at {base}");

    let client = reqwest::blocking::Client::new();
    let add = client
        .post(format!("{}/torrents", base.trim_end_matches('/')))
        .json(&serde_json::json!({ "action": "add", "link": magnet, "file_id": 2 }))
        .timeout(Duration::from_secs(60))
        .send();
    match add {
        Ok(r) => println!("add -> {} {}", r.status(), r.text().unwrap_or_default()),
        Err(e) => println!("add failed: {e}"),
    }
    if std::env::var_os("PROBE_PAUSE_FIRST").is_some() {
        std::thread::sleep(Duration::from_secs(8));
        let _ = client
            .post(format!("{}/torrents", base.trim_end_matches('/')))
            .json(&serde_json::json!({ "action": "deactivate", "link": magnet }))
            .timeout(Duration::from_secs(15))
            .send();
        println!("-- deactivated, re-adding like a second playback --");
        std::thread::sleep(Duration::from_secs(4));
        let re = client
            .post(format!("{}/torrents", base.trim_end_matches('/')))
            .json(&serde_json::json!({ "action": "add", "link": magnet, "file_id": 2 }))
            .timeout(Duration::from_secs(60))
            .send();
        println!(
            "re-add -> {}",
            re.map(|r| r.status().to_string())
                .unwrap_or_else(|e| e.to_string())
        );
    }
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(90) {
        let response = client
            .post(format!("{}/torrents", base.trim_end_matches('/')))
            .json(&serde_json::json!({ "action": "get", "link": magnet, "file_id": 2 }))
            .timeout(Duration::from_secs(5))
            .send();
        match response {
            Ok(r) => match r.json::<serde_json::Value>() {
                Ok(value) => println!("{:>5.1}s {}", started.elapsed().as_secs_f32(), value),
                Err(e) => println!(
                    "{:>5.1}s decode failed: {e}",
                    started.elapsed().as_secs_f32()
                ),
            },
            Err(e) => println!(
                "{:>5.1}s request failed: {e}",
                started.elapsed().as_secs_f32()
            ),
        }
        std::thread::sleep(Duration::from_secs(3));
    }
}
