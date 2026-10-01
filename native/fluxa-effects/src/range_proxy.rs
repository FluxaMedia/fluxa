use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

const CHUNK: u64 = 8 * 1024 * 1024;

pub fn range_proxy(url: &str) -> Option<String> {
    let length = content_length(url)?;
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let address = listener.local_addr().ok()?;
    let runtime = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok()?,
    );
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let upstream = url.to_owned();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(900);
        let _ = listener.set_nonblocking(true);
        while Instant::now() < deadline {
            let Ok((stream, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            };
            let _ = stream.set_nonblocking(false);
            let runtime = runtime.clone();
            let client = client.clone();
            let upstream = upstream.clone();
            std::thread::spawn(move || {
                let _ = serve(stream, &runtime, &client, &upstream, length);
            });
        }
    });
    Some(format!("http://{address}/trailer.mp4"))
}

fn content_length(url: &str) -> Option<u64> {
    url.split_once('?')?
        .1
        .split('&')
        .find_map(|pair| pair.strip_prefix("clen="))
        .and_then(|value| value.parse().ok())
}

fn serve(
    mut stream: TcpStream,
    runtime: &tokio::runtime::Runtime,
    client: &reqwest::Client,
    upstream: &str,
    length: u64,
) -> std::io::Result<()> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte)? == 0 || head.len() > 16 * 1024 {
            return Ok(());
        }
        head.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&head).to_lowercase();
    let is_head = head.starts_with("head ");
    let range = head
        .lines()
        .find_map(|line| line.strip_prefix("range: bytes="))
        .and_then(|value| value.split_once('-'))
        .map(|(start, end)| (start.trim().parse::<u64>().ok(), end.trim().parse::<u64>().ok()));
    let (start, end) = match range {
        Some((Some(start), end)) => (start, end.unwrap_or(length - 1).min(length - 1)),
        _ => (0, length - 1),
    };
    if start > end {
        return stream.write_all(
            format!(
                "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{length}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        );
    }
    let status = if range.is_some() { "206 Partial Content" } else { "200 OK" };
    let content_range = if range.is_some() {
        format!("Content-Range: bytes {start}-{end}/{length}\r\n")
    } else {
        String::new()
    };
    stream.write_all(
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: video/mp4\r\nAccept-Ranges: bytes\r\n{content_range}Content-Length: {}\r\nConnection: close\r\n\r\n",
            end - start + 1
        )
        .as_bytes(),
    )?;
    if is_head {
        return Ok(());
    }
    let mut position = start;
    while position <= end {
        let last = (position + CHUNK - 1).min(end);
        let mut response = runtime
            .block_on(
                client
                    .get(upstream)
                    .header("Range", format!("bytes={position}-{last}"))
                    .send(),
            )
            .map_err(std::io::Error::other)?;
        if !response.status().is_success() {
            return Ok(());
        }
        while let Some(bytes) = runtime
            .block_on(response.chunk())
            .map_err(std::io::Error::other)?
        {
            stream.write_all(&bytes)?;
        }
        position = last + 1;
    }
    Ok(())
}
