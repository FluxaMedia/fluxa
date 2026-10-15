use super::*;
use fluxa_ui::localized;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Entry {
    Download,
    CopyUrl,
    CopyMagnet,
    External,
}

fn http_url(stream: &Value) -> Option<&str> {
    stream
        .get("url")
        .and_then(Value::as_str)
        .filter(|url| url.starts_with("http"))
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn magnet(stream: &Value) -> Option<String> {
    let hash = stream
        .get("infoHash")
        .and_then(Value::as_str)
        .filter(|hash| !hash.is_empty())?;
    let mut link = format!("magnet:?xt=urn:btih:{hash}");
    let name = stream
        .pointer("/behaviorHints/filename")
        .and_then(Value::as_str)
        .or_else(|| stream.get("title").and_then(Value::as_str))
        .and_then(|name| name.lines().next());
    if let Some(name) = name {
        link.push_str(&format!("&dn={}", encode(name)));
    }
    for source in stream
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if let Some(tracker) = source.strip_prefix("tracker:") {
            link.push_str(&format!("&tr={}", encode(tracker)));
        }
    }
    Some(link)
}

pub(crate) fn entries(stream: &Value) -> Vec<Entry> {
    let mut entries = Vec::new();
    if http_url(stream).is_some() {
        entries.push(Entry::Download);
        entries.push(Entry::CopyUrl);
        entries.push(Entry::External);
    } else if magnet(stream).is_some() {
        entries.push(Entry::Download);
        entries.push(Entry::CopyMagnet);
        entries.push(Entry::External);
    }
    entries
}

pub(crate) fn item(entry: Entry, language: &str) -> (&'static str, String) {
    let (icon, key) = match entry {
        Entry::Download => ("Downloads", "sources.download_stream"),
        Entry::CopyUrl => ("Copy", "sources.copy_stream_url"),
        Entry::CopyMagnet => ("Copy", "sources.copy_magnet_link"),
        Entry::External => ("PlayFilled", "common.external_player"),
    };
    (icon, localized(key, language))
}

pub(crate) fn title(stream: &Value) -> String {
    ["name", "title"]
        .into_iter()
        .filter_map(|key| stream.get(key).and_then(Value::as_str))
        .flat_map(str::lines)
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("Stream")
        .to_owned()
}

pub(crate) fn run(state: &mut RendererState, stream: &Value, index: usize, entry: Entry) {
    match entry {
        Entry::Download => crate::player::download_stream(state, index),
        Entry::CopyUrl => {
            state.copied = http_url(stream).map(ToOwned::to_owned);
            crate::sync::show_notice(state, "sources.copied");
        }
        Entry::CopyMagnet => {
            state.copied = magnet(stream);
            crate::sync::show_notice(state, "sources.copied");
        }
        Entry::External => match http_url(stream) {
            Some(url) => play_external(state, url, stream),
            None => resolve_torrent(state, stream),
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn resolve_torrent(state: &mut RendererState, _stream: &Value) {
    crate::sync::show_notice(state, "toast.external_player_not_found");
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_torrent(state: &mut RendererState, stream: &Value) {
    let Some(executor) = state.session.as_ref().map(|session| session.executor()) else {
        return;
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    let (stream, title) = (stream.clone(), title(stream));
    std::thread::spawn(move || {
        let _ = sender.send(executor.torrent_http_url(&stream, &title));
    });
    state.external_pending = Some(receiver);
}

pub(crate) fn tick(state: &mut RendererState) {
    let Some(receiver) = state.external_pending.as_ref() else {
        return;
    };
    match receiver.try_recv() {
        Ok(Ok(url)) => {
            state.external_pending = None;
            play_external(state, &url, &Value::Null);
        }
        Ok(Err(error)) => {
            state.external_pending = None;
            host_log(format!("torrent external play failed: {error}"));
            crate::sync::show_notice(state, "toast.external_player_not_found");
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => state.external_pending = None,
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn play_external(state: &mut RendererState, url: &str, stream: &Value) {
    let headers = stream
        .pointer("/behaviorHints/proxyHeaders/request")
        .and_then(Value::as_object)
        .map(|headers| {
            headers
                .iter()
                .filter_map(|(name, value)| Some(format!("{name}: {}", value.as_str()?)))
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|headers| !headers.is_empty());
    let mut mpv = std::process::Command::new("mpv");
    mpv.arg(url);
    if let Some(headers) = headers {
        mpv.arg(format!("--http-header-fields={headers}"));
    }
    let mut vlc = std::process::Command::new("vlc");
    vlc.arg(url);
    let started = [mpv, vlc].into_iter().any(|mut command| {
        command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|mut child| {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
            })
            .is_ok()
    });
    if !started {
        crate::sync::show_notice(state, "toast.external_player_not_found");
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn play_external(state: &mut RendererState, url: &str, _stream: &Value) {
    state.external_play = Some(url.to_owned());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn torrent_streams_offer_download_magnet_and_external_player() {
        let stream = json!({"infoHash": "abc123", "title": "Show S01E01\nextra", "sources": ["tracker:udp://t.example:80", "dht:abc"]});
        assert_eq!(
            entries(&stream),
            vec![Entry::Download, Entry::CopyMagnet, Entry::External]
        );
        assert_eq!(
            magnet(&stream).unwrap(),
            "magnet:?xt=urn:btih:abc123&dn=Show%20S01E01&tr=udp%3A%2F%2Ft.example%3A80"
        );
    }

    #[test]
    fn http_streams_offer_download_copy_and_external_player() {
        let stream = json!({"url": "https://cdn.test/a.mkv"});
        assert_eq!(
            entries(&stream),
            vec![Entry::Download, Entry::CopyUrl, Entry::External]
        );
        assert!(entries(&json!({"externalUrl": "x"})).is_empty());
    }
}
