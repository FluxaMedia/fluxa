use serde_json::json;

pub fn feed_payload(address: usize, size: usize, pts_ns: i64, video: bool) -> String {
    json!({
        "bufferAddr": format!("{address:#x}"),
        "bufferSize": size,
        "pts": pts_ns,
        "esData": if video { 1 } else { 2 },
    })
    .to_string()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FeedReply {
    Ok,
    BufferFull,
    Failed,
}

impl FeedReply {
    pub fn parse(reply: &str) -> FeedReply {
        if reply.contains("BufferFull") {
            FeedReply::BufferFull
        } else if reply.contains("Ok") {
            FeedReply::Ok
        } else {
            FeedReply::Failed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_hex_string_and_audio_is_two() {
        let json: serde_json::Value = serde_json::from_str(&feed_payload(0x1000, 64, 5, false)).unwrap();
        assert_eq!(json["bufferAddr"], "0x1000");
        assert_eq!(json["esData"], 2);
    }

    #[test]
    fn buffer_full_wins_over_ok() {
        assert_eq!(FeedReply::parse("BufferFull"), FeedReply::BufferFull);
        assert_eq!(FeedReply::parse("Ok"), FeedReply::Ok);
        assert_eq!(FeedReply::parse(""), FeedReply::Failed);
    }
}
