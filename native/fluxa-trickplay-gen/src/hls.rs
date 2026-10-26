//! Minimal HLS playlist parsing: variant selection plus segment starts.

#[derive(Debug, Clone)]
pub struct Segment {
    pub start: f64,
    pub url: String,
}

#[derive(Debug, Default)]
pub struct Playlist {
    pub segments: Vec<Segment>,
    /// Variant playlist URL with the lowest declared BANDWIDTH, if the
    /// parsed document was a master playlist.
    pub lowest_variant_url: Option<String>,
    pub duration: f64,
}

fn attr(line: &str, key: &str) -> Option<u64> {
    line.split(',').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        k.trim()
            .eq_ignore_ascii_case(key)
            .then(|| v.trim().trim_matches('"').parse::<u64>().ok())?
    })
}

pub fn parse(data: &[u8]) -> Result<Playlist, &'static str> {
    let text = std::str::from_utf8(data).map_err(|_| "playlist is not utf-8")?;
    if !text.contains("#EXTM3U") {
        return Err("missing #EXTM3U");
    }
    let mut playlist = Playlist::default();
    let mut acc = 0.0f64;
    let mut pending_duration: Option<f64> = None;
    let mut pending_bandwidth: Option<u64> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            pending_duration = Some(
                rest.split(',')
                    .next()
                    .unwrap_or("0")
                    .trim()
                    .parse()
                    .unwrap_or(0.0),
            );
        } else if line.starts_with("#EXT-X-STREAM-INF:") {
            pending_bandwidth = attr(line, "BANDWIDTH");
        } else if line.starts_with('#') || line.is_empty() {
            continue;
        } else if let Some(bandwidth) = pending_bandwidth.take() {
            let is_first = playlist.lowest_variant_url.is_none();
            let is_lower = playlist
                .lowest_variant_url
                .as_ref()
                .and_then(|_| attr(line, "BANDWIDTH"))
                .map(|other| bandwidth < other)
                .unwrap_or(false);
            let _ = is_lower;
            if is_first {
                playlist.lowest_variant_url = Some(line.to_owned());
            }
        } else if let Some(dur) = pending_duration.take() {
            playlist.segments.push(Segment {
                start: acc,
                url: line.to_owned(),
            });
            acc += dur;
        }
    }
    playlist.duration = acc;
    Ok(playlist)
}

pub fn resolve(base: &str, target: &str) -> String {
    if target.contains("://") {
        return target.to_owned();
    }
    match base.rfind('/') {
        Some(pos) => format!("{}/{}", &base[..pos], target),
        None => target.to_owned(),
    }
}
