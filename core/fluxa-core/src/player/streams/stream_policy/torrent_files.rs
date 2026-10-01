use super::meta::VIDEO_FILE_EXTENSIONS;

#[derive(Clone, serde::Deserialize)]
pub(crate) struct TorrentFileStat {
    pub(crate) id: i32,
    pub(crate) path: String,
    pub(crate) length: i64,
}

pub(crate) fn normalize_torrent_file_name(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace('\\', "/")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
pub(crate) fn is_likely_video_file(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    VIDEO_FILE_EXTENSIONS
        .iter()
        .any(|extension| path.ends_with(extension))
}
fn torrent_episode_tokens(title: &str) -> Vec<String> {
    let parts: Vec<&str> = title.split(':').collect();
    if parts.len() < 3 {
        return Vec::new();
    }
    let mut parts = title.rsplitn(3, ':');
    let episode = parts.next().and_then(|value| value.parse::<u32>().ok());
    let season = parts.next().and_then(|value| value.parse::<u32>().ok());
    match (season, episode) {
        (Some(season), Some(episode)) => vec![
            format!("s{season:02}e{episode:02}"),
            format!("{season}x{episode:02}"),
        ],
        _ => Vec::new(),
    }
}
fn matches_torrent_episode(path: &str, tokens: &[String]) -> bool {
    let normalized = path.to_ascii_lowercase();
    tokens.iter().any(|token| normalized.contains(token))
}
pub(crate) fn resolve_torrent_file_index(
    title: &str,
    requested_file_idx: Option<i32>,
    preferred_filename: Option<&str>,
    file_stats: &[TorrentFileStat],
) -> (Option<i32>, Option<String>) {
    // addon-provided fileIdx is authoritative — use it directly
    if let Some(idx) = requested_file_idx {
        return (Some(idx), Some("requested".to_string()));
    }

    if file_stats.is_empty() {
        return (None, None);
    }

    if let Some(preferred) = preferred_filename
        .map(normalize_torrent_file_name)
        .filter(|value| !value.is_empty())
        && let Some(stat) = file_stats.iter().find(|stat| {
            let path = normalize_torrent_file_name(&stat.path);
            path == preferred
                || path.ends_with(&format!("/{preferred}"))
                || path.rsplit('/').next() == Some(preferred.as_str())
        })
    {
        return (Some(stat.id), Some("filename".to_string()));
    }

    let episode_tokens = torrent_episode_tokens(title);
    if !episode_tokens.is_empty()
        && let Some(stat) = file_stats
            .iter()
            .filter(|stat| is_likely_video_file(&stat.path))
            .filter(|stat| matches_torrent_episode(&stat.path, &episode_tokens))
            .max_by_key(|stat| stat.length)
    {
        return (Some(stat.id), Some("episode".to_string()));
    }

    file_stats
        .iter()
        .filter(|stat| is_likely_video_file(&stat.path))
        .max_by_key(|stat| stat.length)
        .map(|stat| (Some(stat.id), Some("largest-video".to_string())))
        .unwrap_or((None, None))
}
// Pulls a "s01e02"-style tag out of a lowercased filename, if present.
// Season-pack subtitle folders/files usually keep this tag even when the
// rest of the naming (release group, resolution) differs from the video.

// Decides which torrent files are sibling subtitles for the selected video:
// matched by filename, by shared "s01e02" episode tag, or (for a torrent
// with only one video file) any subtitle file at all since there's nothing
// else to disambiguate against.
pub(crate) fn torrent_fallback_file_indexes(
    title: &str,
    rejected_index: Option<i32>,
    file_stats: &[TorrentFileStat],
) -> Vec<i32> {
    let mut videos: Vec<&TorrentFileStat> = file_stats
        .iter()
        .filter(|stat| is_likely_video_file(&stat.path))
        .collect();
    let episode_tokens = torrent_episode_tokens(title);
    videos.sort_by_key(|stat| {
        (
            !matches_torrent_episode(&stat.path, &episode_tokens),
            std::cmp::Reverse(stat.length),
        )
    });
    videos
        .into_iter()
        .filter(|stat| Some(stat.id) != rejected_index)
        .map(|stat| stat.id)
        .collect()
}
