use regex::RegexBuilder;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum Source {
    Sheets {
        url: String,
        interval: u64,
        tile_width: u64,
        tile_height: u64,
        count: u64,
    },
    Bif {
        url: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub sheet: u64,
    pub column: u64,
    pub row: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BifFrame {
    pub time_ms: u64,
    pub start: usize,
    pub end: usize,
}

pub fn source_from_stream(stream: &Value) -> Option<Source> {
    serde_json::from_value(stream.get("trickplay")?.clone()).ok()
}

pub fn sheet_url(url: &str, sheet: u64) -> String {
    url.replace("{sheet}", &sheet.to_string())
}

pub fn tile_at(
    time_ms: u64,
    interval: u64,
    tile_width: u64,
    tile_height: u64,
    count: u64,
) -> Option<Tile> {
    let per_sheet = tile_width * tile_height;
    if interval == 0 || per_sheet == 0 || count == 0 {
        return None;
    }
    let index = (time_ms / interval).min(count - 1);
    let within = index % per_sheet;
    Some(Tile {
        sheet: index / per_sheet,
        column: within % tile_width,
        row: within / tile_width,
    })
}

const BIF_MAGIC: [u8; 8] = [0x89, b'B', b'I', b'F', 0x0D, 0x0A, 0x1A, 0x0A];
const BIF_HEADER: usize = 64;

pub fn parse_bif(bytes: &[u8]) -> Option<Vec<BifFrame>> {
    if bytes.len() < BIF_HEADER || bytes[..8] != BIF_MAGIC {
        return None;
    }
    let word = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
    };
    let count = word(12)? as usize;
    let multiplier = match word(16)? {
        0 => 1000,
        value => value as u64,
    };
    let mut frames = Vec::with_capacity(count);
    for index in 0..count {
        let at = BIF_HEADER + index * 8;
        let (time, start) = (word(at)?, word(at + 4)? as usize);
        let end = word(at + 12)? as usize;
        if start > end || end > bytes.len() {
            return None;
        }
        frames.push(BifFrame {
            time_ms: time as u64 * multiplier,
            start,
            end,
        });
    }
    Some(frames)
}

pub fn frame_at(frames: &[BifFrame], time_ms: u64) -> Option<&BifFrame> {
    let after = frames.partition_point(|frame| frame.time_ms <= time_ms);
    frames.get(after.saturating_sub(1))
}

pub fn plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let settings = args.get("settings").unwrap_or(&Value::Null);
    let text = |key: &str| {
        settings
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
    };
    let flag = |key: &str, default: bool| {
        settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    };
    let (kind, id) = match text("trickplaySource").split_once(':') {
        Some((kind @ ("addon" | "plugin"), id)) if !id.is_empty() => (kind, id),
        _ if text("trickplaySource") == "server" && !text("trickplayServerUrl").is_empty() => {
            ("server", text("trickplayServerUrl"))
        }
        _ => ("stream", ""),
    };
    let interval = text("trickplayInterval")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map_or(10.0, |value| value.clamp(5.0, 3600.0));
    let sheet = flag("trickplaySheet", true);
    Some(
        json!({
            "enabled": flag("seekThumbnailEnabled", true),
            "source": {"kind": kind, "id": id},
            "regex": text("trickplayRegex"),
            "interval": interval,
            "sheet": sheet,
            "pregenerate": sheet && text("trickplayMode") == "pregenerate",
        })
        .to_string(),
    )
}

fn server_base(base: &str, video_id: &str) -> Option<String> {
    let base = base.trim().trim_end_matches('/');
    if !base.starts_with("http") || video_id.is_empty() {
        return None;
    }
    let path = video_id
        .split(':')
        .map(|part| part.trim())
        .collect::<Vec<_>>()
        .join("/");
    Some(format!("{base}/{path}"))
}

pub fn server_manifest_url_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let base = server_base(args.get("base")?.as_str()?, args.get("videoId")?.as_str()?)?;
    Some(json!(format!("{base}/manifest.json")).to_string())
}

pub fn server_source_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let base = server_base(args.get("base")?.as_str()?, args.get("videoId")?.as_str()?)?;
    let manifest = args.get("manifest")?;
    let number = |key: &str| manifest.get(key).and_then(Value::as_u64).filter(|n| *n > 0);
    Some(
        json!({
            "kind": "sheets",
            "url": format!("{base}/{{sheet}}.jpg"),
            "interval": number("intervalMs")?,
            "tileWidth": number("columns")?,
            "tileHeight": number("rows")?,
            "count": number("count")?,
        })
        .to_string(),
    )
}

fn headers_of(stream: &Value) -> Vec<(String, String)> {
    let hints = stream.get("behaviorHints");
    [
        stream.get("headers"),
        hints.and_then(|hints| hints.get("requestHeaders")),
        hints.and_then(|hints| hints.pointer("/proxyHeaders/request")),
    ]
    .into_iter()
    .flatten()
    .filter_map(Value::as_object)
    .flatten()
    .filter_map(|(name, value)| Some((name.clone(), value.as_str()?.to_owned())))
    .collect()
}

pub fn pick_source_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let pattern = args
        .get("pattern")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let matcher = if pattern.is_empty() {
        None
    } else {
        Some(
            RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .ok()?,
        )
    };
    let text = |stream: &Value, key: &str| {
        stream
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let found = args
        .get("streams")?
        .as_array()?
        .iter()
        .filter(|stream| {
            stream
                .get("url")
                .and_then(Value::as_str)
                .is_some_and(|url| url.starts_with("http"))
        })
        .find(|stream| {
            matcher.as_ref().is_none_or(|matcher| {
                let haystack = ["name", "title", "description", "addonName", "url"]
                    .iter()
                    .map(|key| text(stream, key))
                    .collect::<Vec<_>>()
                    .join("\n");
                matcher.is_match(&haystack)
            })
        })?;
    let headers: Vec<Value> = headers_of(found)
        .into_iter()
        .map(|(name, value)| json!([name, value]))
        .collect();
    Some(json!({"url": found.get("url")?, "headers": headers}).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bif(frames: &[(u32, &[u8])]) -> Vec<u8> {
        let mut out = BIF_MAGIC.to_vec();
        out.extend(0u32.to_le_bytes());
        out.extend((frames.len() as u32).to_le_bytes());
        out.extend(1000u32.to_le_bytes());
        out.resize(BIF_HEADER, 0);
        let mut offset = BIF_HEADER + (frames.len() + 1) * 8;
        for (time, data) in frames {
            out.extend(time.to_le_bytes());
            out.extend((offset as u32).to_le_bytes());
            offset += data.len();
        }
        out.extend(u32::MAX.to_le_bytes());
        out.extend((offset as u32).to_le_bytes());
        for (_, data) in frames {
            out.extend(*data);
        }
        out
    }

    #[test]
    fn bif_frames_slice_the_right_bytes() {
        let file = bif(&[(0, b"aa"), (10, b"bbb"), (20, b"c")]);
        let frames = parse_bif(&file).unwrap();
        assert_eq!(frames.len(), 3);
        let second = frame_at(&frames, 14_000).unwrap();
        assert_eq!(&file[second.start..second.end], b"bbb");
        assert_eq!(second.time_ms, 10_000);
        let last = frame_at(&frames, 9_999_999).unwrap();
        assert_eq!(&file[last.start..last.end], b"c");
    }

    #[test]
    fn truncated_bif_is_rejected() {
        let mut file = bif(&[(0, b"aa"), (10, b"bbb")]);
        file.truncate(file.len() - 3);
        assert!(parse_bif(&file).is_none());
        assert!(parse_bif(b"not a bif").is_none());
    }

    #[test]
    fn tile_walks_rows_then_sheets() {
        let tile = |ms| tile_at(ms, 10_000, 10, 10, 250).unwrap();
        assert_eq!(
            tile(0),
            Tile {
                sheet: 0,
                column: 0,
                row: 0
            }
        );
        assert_eq!(
            tile(135_000),
            Tile {
                sheet: 0,
                column: 3,
                row: 1
            }
        );
        assert_eq!(
            tile(1_000_000),
            Tile {
                sheet: 1,
                column: 0,
                row: 0
            }
        );
        assert_eq!(
            tile(99_999_999),
            Tile {
                sheet: 2,
                column: 9,
                row: 4
            }
        );
    }

    #[test]
    fn stream_without_trickplay_has_no_source() {
        assert!(source_from_stream(&json!({"url": "x"})).is_none());
        let stream = json!({"trickplay": {"kind": "bif", "url": "u"}});
        assert_eq!(
            source_from_stream(&stream),
            Some(Source::Bif { url: "u".into() })
        );
    }

    #[test]
    fn pick_uses_first_http_stream_without_pattern() {
        let streams = json!([
            {"name": "torrent", "infoHash": "abc"},
            {"name": "Low", "url": "https://a/low.m3u8", "behaviorHints": {"proxyHeaders": {"request": {"Referer": "https://r/"}}}},
            {"name": "High", "url": "https://a/high.mkv"}
        ]);
        let picked: Value = serde_json::from_str(
            &pick_source_json(&json!({"streams": streams, "pattern": ""}).to_string()).unwrap(),
        )
        .unwrap();
        assert_eq!(picked["url"], "https://a/low.m3u8");
        assert_eq!(picked["headers"], json!([["Referer", "https://r/"]]));
    }

    #[test]
    fn pick_filters_by_regex_and_rejects_bad_patterns() {
        let streams = json!([
            {"name": "Debrid", "title": "2160p remux", "url": "https://a/4k.mkv"},
            {"name": "Debrid", "title": "480P web", "url": "https://a/480.mp4"}
        ]);
        let pick = |pattern: &str| {
            pick_source_json(&json!({"streams": streams, "pattern": pattern}).to_string())
        };
        assert!(pick("480p").unwrap().contains("480.mp4"));
        assert!(pick("1080p").is_none());
        assert!(pick("(").is_none());
    }

    #[test]
    fn server_urls_follow_movie_and_episode_ids() {
        let url = |id: &str| {
            let args = json!({"base": "https://api.example.app/static/trickplay/", "videoId": id});
            server_manifest_url_json(&args.to_string())
                .map(|text| serde_json::from_str::<String>(&text).unwrap())
        };
        assert_eq!(
            url("tt0111161").unwrap(),
            "https://api.example.app/static/trickplay/tt0111161/manifest.json"
        );
        assert_eq!(
            url("tt0903747:1:1").unwrap(),
            "https://api.example.app/static/trickplay/tt0903747/1/1/manifest.json"
        );
        assert!(url("").is_none());
    }

    #[test]
    fn server_manifest_becomes_sheet_source() {
        let args = json!({
            "base": "https://api.example.app/static/trickplay",
            "videoId": "tt0903747:1:1",
            "manifest": {"intervalMs": 10000, "count": 350, "width": 320, "height": 180, "columns": 5, "rows": 5},
        });
        let source: Source =
            serde_json::from_str(&server_source_json(&args.to_string()).unwrap()).unwrap();
        assert_eq!(
            source,
            Source::Sheets {
                url: "https://api.example.app/static/trickplay/tt0903747/1/1/{sheet}.jpg"
                    .to_owned(),
                interval: 10000,
                tile_width: 5,
                tile_height: 5,
                count: 350,
            }
        );
    }

    #[test]
    fn plan_reads_settings_and_clamps_interval() {
        let plan = |settings: Value| -> Value {
            serde_json::from_str(&plan_json(&json!({"settings": settings}).to_string()).unwrap())
                .unwrap()
        };
        let defaults = plan(json!({}));
        assert_eq!(defaults["source"]["kind"], "stream");
        assert_eq!(defaults["interval"], 10.0);
        assert_eq!(defaults["pregenerate"], false);
        let custom = plan(json!({
            "trickplaySource": "plugin:trickplay", "trickplayInterval": "0.2",
            "trickplayMode": "pregenerate", "trickplaySheet": true
        }));
        assert_eq!(custom["source"]["id"], "trickplay");
        assert_eq!(custom["interval"], 5.0);
        assert_eq!(custom["pregenerate"], true);
        let no_sheet = plan(json!({"trickplayMode": "pregenerate", "trickplaySheet": false}));
        assert_eq!(no_sheet["pregenerate"], false);
    }
}
