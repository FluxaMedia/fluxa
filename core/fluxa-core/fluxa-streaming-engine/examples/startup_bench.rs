use std::io::Read;
use std::time::{Duration, Instant};

fn main() {
    let magnet = std::env::args().nth(1).expect("usage: startup_bench <magnet> [file_index]");
    let index = std::env::args().nth(2).unwrap_or_else(|| "0".into());
    let cache = std::env::temp_dir().join(format!("fluxa-bench-{}", std::process::id()));
    let server = fluxa_streaming_engine::start_torrent_server(cache.to_str().unwrap(), 0, "bench").unwrap();
    let server: serde_json::Value = serde_json::from_str(&server).unwrap();
    let base = server["url"].as_str().unwrap();
    let link: String = url::form_urlencoded::byte_serialize(magnet.as_bytes()).collect();

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(600)).build().unwrap();
    let start = Instant::now();
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
