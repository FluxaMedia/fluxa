use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DownloadStatus {
    #[default]
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
}

impl DownloadStatus {
    fn parse(value: &str) -> Self {
        match value {
            "downloading" => Self::Downloading,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Queued,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DownloadRow {
    pub id: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub title: String,
    pub thumbnail: Option<String>,
    pub status: DownloadStatus,
    pub bytes_done: u64,
    pub bytes_total: Option<u64>,
    pub percent: f32,
    pub speed_bps: f64,
    pub eta_ms: Option<u64>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadGroup {
    pub id: String,
    pub title: String,
    pub is_series: bool,
    pub logo: Option<String>,
    pub background: Option<String>,
    pub poster: Option<String>,
    pub rows: Vec<DownloadRow>,
}

impl DownloadGroup {
    pub fn bytes_done(&self) -> u64 {
        self.rows.iter().map(|row| row.bytes_done).sum()
    }

    pub fn active(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.status == DownloadStatus::Downloading)
            .count()
    }

    pub fn active_fraction(&self) -> Option<f32> {
        let active: Vec<_> = self
            .rows
            .iter()
            .filter(|row| row.status == DownloadStatus::Downloading)
            .collect();
        let total: u64 = active.iter().filter_map(|row| row.bytes_total).sum();
        if active.is_empty() || total == 0 {
            return None;
        }
        let done: u64 = active.iter().map(|row| row.bytes_done).sum();
        Some((done as f32 / total as f32).clamp(0.0, 1.0))
    }

    pub fn speed_bps(&self) -> f64 {
        self.rows
            .iter()
            .filter(|row| row.status == DownloadStatus::Downloading)
            .map(|row| row.speed_bps)
            .sum()
    }

    pub fn eta_ms(&self) -> Option<u64> {
        self.rows
            .iter()
            .filter(|row| row.status == DownloadStatus::Downloading)
            .filter_map(|row| row.eta_ms)
            .max()
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.rows
            .iter()
            .filter(|row| row.status == DownloadStatus::Downloading)
            .map(|row| row.elapsed_ms)
            .max()
            .unwrap_or(0)
    }

    pub fn any(&self, status: DownloadStatus) -> bool {
        self.rows.iter().any(|row| row.status == status)
    }

    pub fn completed(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.status == DownloadStatus::Completed)
            .count()
    }
}

pub fn downloads_from_snapshot(snapshot: &Value) -> Vec<DownloadGroup> {
    let Some(items) = snapshot.pointer("/offline/items").and_then(Value::as_array) else {
        return Vec::new();
    };
    let text = |item: &Value, key: &str| {
        item.get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    };
    let number = |item: &Value, key: &str| {
        item.get(key)
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
    };
    let mut groups: Vec<(String, DownloadGroup)> = Vec::new();
    for item in items {
        let meta_id = text(item, "metaId").unwrap_or_default();
        let name = text(item, "name").unwrap_or_default();
        let season = number(item, "season");
        let episode = number(item, "episode");
        let row = DownloadRow {
            id: text(item, "id").unwrap_or_default(),
            season,
            episode,
            title: text(item, "episodeTitle").unwrap_or_else(|| name.clone()),
            thumbnail: text(item, "thumbnail"),
            status: DownloadStatus::parse(item["status"].as_str().unwrap_or_default()),
            bytes_done: item["bytesDone"].as_u64().unwrap_or(0),
            bytes_total: item["bytesTotal"].as_u64(),
            percent: item["percent"].as_f64().unwrap_or(0.0) as f32,
            speed_bps: item["speedBps"].as_f64().unwrap_or(0.0),
            eta_ms: item["etaMs"].as_u64(),
            elapsed_ms: item["elapsedMs"].as_u64().unwrap_or(0),
            error: text(item, "error"),
        };
        let index = match groups.iter().position(|(id, _)| *id == meta_id) {
            Some(index) => index,
            None => {
                let meta_id_for_group = meta_id.clone();
                groups.push((
                    meta_id,
                    DownloadGroup {
                        id: meta_id_for_group.clone(),
                        title: name,
                        is_series: season.is_some() || episode.is_some(),
                        logo: text(item, "logo"),
                        background: text(item, "background"),
                        poster: text(item, "poster"),
                        rows: Vec::new(),
                    },
                ));
                groups.len() - 1
            }
        };
        groups[index].1.rows.push(row);
    }
    let mut groups = groups
        .into_iter()
        .map(|(_, group)| group)
        .collect::<Vec<_>>();
    for group in &mut groups {
        group
            .rows
            .sort_by_key(|row| (row.season.unwrap_or(0), row.episode.unwrap_or(0)));
    }
    groups
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StorageInfo {
    pub path: String,
    pub label: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadsSummary {
    pub speed_bps: f64,
    pub history: Vec<f32>,
    pub location: Option<StorageInfo>,
    pub volumes: Vec<StorageInfo>,
}

fn storage_info(value: &Value) -> Option<StorageInfo> {
    Some(StorageInfo {
        path: value.get("path")?.as_str()?.to_owned(),
        label: value["label"].as_str().unwrap_or_default().to_owned(),
        free_bytes: value["freeBytes"].as_u64().unwrap_or(0),
        total_bytes: value["totalBytes"].as_u64().unwrap_or(0),
    })
}

pub fn downloads_summary_from_snapshot(snapshot: &Value) -> DownloadsSummary {
    let offline = &snapshot["offline"];
    DownloadsSummary {
        speed_bps: offline["totalSpeedBps"].as_f64().unwrap_or(0.0),
        history: offline["history"]
            .as_array()
            .map(|samples| samples.iter().filter_map(Value::as_f64).map(|v| v as f32).collect())
            .unwrap_or_default(),
        location: storage_info(&offline["location"]),
        volumes: offline["volumes"]
            .as_array()
            .map(|volumes| volumes.iter().filter_map(storage_info).collect())
            .unwrap_or_default(),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn format_duration(ms: u64) -> String {
    let seconds = ms / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn episodes_group_under_one_series_in_episode_order() {
        let item = |id: &str, episode: u32| json!({"id": id, "metaId": "tt1", "name": "Show", "season": 1, "episode": episode, "status": "downloading", "logo": "l"});
        let snapshot = json!({"offline": {"items": [item("c", 3), item("a", 1), {"id": "m", "metaId": "tt2", "name": "Film", "status": "completed"}]}});
        let groups = downloads_from_snapshot(&snapshot);
        assert_eq!(groups.len(), 2);
        assert!(groups[0].is_series);
        assert_eq!(groups[0].rows[0].id, "a");
        assert_eq!(groups[0].rows[1].id, "c");
        assert!(!groups[1].is_series);
    }

    #[test]
    fn summary_reads_speed_history_and_location() {
        let snapshot = json!({"offline": {
            "totalSpeedBps": 2048.0,
            "history": [0.0, 1024.0, 2048.0],
            "location": {"path": "/d", "label": "default", "freeBytes": 5, "totalBytes": 10},
            "volumes": [{"path": "/d", "label": "default", "freeBytes": 5, "totalBytes": 10}, {"label": "broken"}]
        }});
        let summary = downloads_summary_from_snapshot(&snapshot);
        assert_eq!(summary.history, vec![0.0, 1024.0, 2048.0]);
        assert_eq!(summary.location.unwrap().free_bytes, 5);
        assert_eq!(summary.volumes.len(), 1);
    }

    #[test]
    fn byte_sizes_pick_a_readable_unit() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.0 GB");
        assert_eq!(format_bytes(250 * 1024 * 1024), "250 MB");
    }

    #[test]
    fn durations_show_hours_only_when_needed() {
        assert_eq!(format_duration(65_000), "1:05");
        assert_eq!(format_duration(3_725_000), "1:02:05");
    }
}
