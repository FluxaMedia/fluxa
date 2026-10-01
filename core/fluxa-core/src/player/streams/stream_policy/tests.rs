use super::magnet::*;
use super::torrent_files::*;
use serde_json::json;






#[test]
fn build_magnet_dedupes_addon_tracker_and_appends_fallbacks() {
    let magnet = build_magnet(
        "ABCDEF1234567890ABCDEF1234567890ABCDEF12",
        &["tracker:udp://tracker.example:1337/announce".to_string()],
    );

    assert!(magnet.starts_with("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12"));
    assert_eq!(magnet.matches("tracker.example%3A1337").count(), 1);
    assert!(magnet.contains("opentrackr.org"));
}

#[test]
fn stream_magnet_link_builds_from_info_hash_and_sources() {
    let stream = json!({
        "infoHash": "ABCDEF1234567890ABCDEF1234567890ABCDEF12",
        "sources": ["tracker:udp://tracker.example:1337/announce"],
    });
    let link = stream_magnet_link(&stream).unwrap();
    assert!(link.starts_with("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12"));
}



#[test]
fn stream_magnet_link_none_for_direct_http_stream() {
    let stream = json!({ "url": "https://cdn.example/video.mkv" });
    assert!(stream_magnet_link(&stream).is_none());
}

#[test]
fn resolve_torrent_file_index_prefers_requested_then_filename_then_largest_video() {
    let stats = vec![
        TorrentFileStat {
            id: 1,
            path: "Show.S01E01.mkv".to_string(),
            length: 100,
        },
        TorrentFileStat {
            id: 2,
            path: "Show.S01E02.mkv".to_string(),
            length: 300,
        },
        TorrentFileStat {
            id: 3,
            path: "sample.txt".to_string(),
            length: 999_999,
        },
    ];

    // Addon-provided fileIdx wins outright, even though it doesn't match any stat.
    assert_eq!(
        resolve_torrent_file_index("title", Some(9), None, &stats),
        (Some(9), Some("requested".to_string()))
    );

    // No requested index, but a preferred filename matches by basename.
    assert_eq!(
        resolve_torrent_file_index("title", None, Some("Show.S01E01.mkv"), &stats),
        (Some(1), Some("filename".to_string()))
    );

    // No requested index or filename match — falls back to the largest *video* file,
    // ignoring the much larger non-video sample.txt.
    assert_eq!(
        resolve_torrent_file_index("title", None, None, &stats),
        (Some(2), Some("largest-video".to_string()))
    );

    assert_eq!(
        resolve_torrent_file_index("title", None, None, &[]),
        (None, None)
    );
}
