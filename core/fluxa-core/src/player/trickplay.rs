use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", rename_all_fields = "camelCase")]
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
        assert_eq!(tile(0), Tile { sheet: 0, column: 0, row: 0 });
        assert_eq!(tile(135_000), Tile { sheet: 0, column: 3, row: 1 });
        assert_eq!(tile(1_000_000), Tile { sheet: 1, column: 0, row: 0 });
        assert_eq!(tile(99_999_999), Tile { sheet: 2, column: 9, row: 4 });
    }

    #[test]
    fn stream_without_trickplay_has_no_source() {
        assert!(source_from_stream(&json!({"url": "x"})).is_none());
        let stream = json!({"trickplay": {"kind": "bif", "url": "u"}});
        assert_eq!(source_from_stream(&stream), Some(Source::Bif { url: "u".into() }));
    }
}
