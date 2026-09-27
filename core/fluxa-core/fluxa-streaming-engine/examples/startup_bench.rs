use std::io::Read;
use std::time::{Duration, Instant};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    let magnet = std::env::args().nth(1).expect("usage: startup_bench <magnet> [file_index]");
    let index = std::env::args().nth(2).unwrap_or_else(|| "0".into());
    let cache = std::env::temp_dir().join(format!("fluxa-bench-{}", std::process::id()));
    let server = fluxa_streaming_engine::start_torrent_server(cache.to_str().unwrap(), 0, "bench").unwrap();
    let server: serde_json::Value = serde_json::from_str(&server).unwrap();
    let base = server["url"].as_str().unwrap();
    let link: String = url::form_urlencoded::byte_serialize(magnet.as_bytes()).collect();

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(600)).build().unwrap();
    let start = Instant::now();
    let status_url = format!("{base}/torrents");
    let status_magnet = magnet.clone();
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let Ok(res) = client
                .post(&status_url)
                .json(&serde_json::json!({ "action": "get", "link": status_magnet }))
                .send()
            else {
                continue;
            };
            let Ok(s) = res.json::<serde_json::Value>() else { continue };
            eprintln!(
                "{:5.1} {} seen={} connecting={} live={} speed={:.0}KB/s loaded={}",
                start.elapsed().as_secs_f64(),
                s["stat_string"].as_str().unwrap_or("?"),
                s["total_peers"],
                s["connecting_peers"],
                s["active_peers"],
                s["download_speed"].as_f64().unwrap_or(0.0) / 1024.0,
                s["loaded_size"],
            );
        }
    });
    let mut out = serde_json::Map::new();
    let mut body = client
        .get(format!("{base}/stream/fname?link={link}&index={index}&play"))
        .header("Range", "bytes=0-")
        .send()
        .unwrap();
    out.insert("meta".into(), start.elapsed().as_secs_f64().into());
    let mut buf = vec![0u8; 1 << 16];
    let mut got = 0;
    for (key, mark) in [("first", 64 << 10), ("mb16", 16 << 20)] {
        while got < mark {
            match body.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => got += n,
            }
        }
        if got >= mark {
            out.insert(key.into(), start.elapsed().as_secs_f64().into());
        }
    }
    println!("{}", serde_json::Value::Object(out));
    let _ = std::fs::remove_dir_all(cache);
}
