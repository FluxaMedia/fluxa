use super::*;
use crate::components;
use crate::draw_text;
use serde_json::Value;

const HOUR_MS: i64 = 3_600_000;

#[derive(Clone, Debug, Default)]
pub struct GuideProgramme {
    pub title: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Clone, Debug, Default)]
pub struct GuideChannel {
    pub name: String,
    pub logo: Option<String>,
    pub raw: Value,
    pub programmes: Vec<GuideProgramme>,
}

#[derive(Clone, Debug, Default)]
pub struct GuideModel {
    pub channels: Vec<GuideChannel>,
    pub from_ms: i64,
    pub to_ms: i64,
    pub now_ms: i64,
    pub utc_offset_ms: i64,
    pub loading: bool,
    pub failed: bool,
    pub language: String,
}

impl GuideModel {
    pub fn apply(&mut self, guide: &Value) {
        self.failed = guide["failed"].as_bool() == Some(true);
        self.channels = guide["channels"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|channel| GuideChannel {
                name: channel["name"].as_str().unwrap_or_default().to_owned(),
                logo: channel["logo"]
                    .as_str()
                    .or_else(|| channel["poster"].as_str())
                    .map(ToOwned::to_owned),
                raw: channel.clone(),
                programmes: channel["programmes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|programme| GuideProgramme {
                        title: programme["title"]
                            .as_str()
                            .or_else(|| programme["name"].as_str())
                            .unwrap_or_default()
                            .to_owned(),
                        start_ms: programme["startMs"].as_i64().unwrap_or_default(),
                        end_ms: programme["endMs"].as_i64().unwrap_or_default(),
                    })
                    .collect(),
            })
            .collect();
    }

    pub fn channel_for_node(&self, node: u64) -> Option<&GuideChannel> {
        let index = if (NODE_GUIDE_CELL_BASE..NODE_GUIDE_CELL_BASE + NODE_GUIDE_CELL_SPAN)
            .contains(&node)
        {
            (node - NODE_GUIDE_CELL_BASE) / NODE_GUIDE_ROW_STRIDE
        } else if (NODE_GUIDE_CHANNEL_BASE..NODE_GUIDE_CELL_BASE).contains(&node) {
            node - NODE_GUIDE_CHANNEL_BASE
        } else {
            return None;
        };
        self.channels.get(index as usize)
    }

    pub fn node_label(&self, node: u64) -> Option<String> {
        let channel = self.channel_for_node(node)?;
        if node >= NODE_GUIDE_CELL_BASE {
            let index = ((node - NODE_GUIDE_CELL_BASE) % NODE_GUIDE_ROW_STRIDE) as usize;
            if let Some(programme) = channel.programmes.get(index) {
                return Some(format!("{}, {}", channel.name, programme.title));
            }
        }
        Some(channel.name.clone())
    }

    pub fn last_end_ms(&self) -> Option<i64> {
        self.channels
            .iter()
            .filter_map(|channel| channel.programmes.last())
            .map(|programme| programme.end_ms)
            .max()
    }
}

pub const NODE_GUIDE_ROW_STRIDE: u64 = 512;
pub const NODE_GUIDE_CELL_SPAN: u64 = 400 * NODE_GUIDE_ROW_STRIDE;

fn clock(ms: i64, offset_ms: i64) -> String {
    let minutes = (ms + offset_ms).div_euclid(60_000).rem_euclid(24 * 60);
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn model() -> GuideModel {
        let mut model = GuideModel::default();
        model.apply(&json!({
            "failed": false,
            "channels": [
                {"id": "c1", "name": "One", "programmes": [
                    {"title": "A", "startMs": 0, "endMs": 10},
                    {"title": "B", "startMs": 10, "endMs": 20}
                ]},
                {"id": "c2", "name": "Two", "programmes": [
                    {"title": "C", "startMs": 5, "endMs": 30}
                ]}
            ]
        }));
        model
    }

    #[test]
    fn programme_node_maps_back_to_its_channel() {
        let model = model();
        let node = NODE_GUIDE_CELL_BASE + NODE_GUIDE_ROW_STRIDE + 0;
        assert_eq!(model.channel_for_node(node).unwrap().name, "Two");
        assert_eq!(model.node_label(node).unwrap(), "Two, C");
        assert_eq!(model.node_label(NODE_GUIDE_CHANNEL_BASE).unwrap(), "One");
    }

    #[test]
    fn nodes_outside_the_guide_are_ignored() {
        let model = model();
        assert!(
            model
                .channel_for_node(NODE_GUIDE_CHANNEL_BASE + 5)
                .is_none()
        );
        assert!(model.channel_for_node(NODE_HOME).is_none());
    }

    #[test]
    fn last_end_is_the_latest_programme_end() {
        assert_eq!(model().last_end_ms(), Some(30));
    }

    #[test]
    fn clock_applies_the_utc_offset_and_wraps_the_day() {
        assert_eq!(clock(23 * HOUR_MS, 2 * HOUR_MS), "01:00");
        assert_eq!(clock(0, -HOUR_MS), "23:00");
    }
}

const GUIDE_LABEL_WIDTH: f32 = 140.0;

pub fn guide_data(guide: &GuideModel) -> serde_json::Value {
    let language = guide.language.as_str();
    let hour = 280.0_f32;
    let scale = |ms: i64| ms as f32 / HOUR_MS as f32 * hour;
    let span = scale(guide.to_ms - guide.from_ms).max(0.0);
    let hours: Vec<_> = (0..=((guide.to_ms - guide.from_ms) / HOUR_MS).max(0))
        .map(|step| {
            serde_json::json!({
                "label": clock(guide.from_ms + step * HOUR_MS, guide.utc_offset_ms),
                "width": hour.to_string(),
            })
        })
        .collect();
    let channels: Vec<_> = guide
        .channels
        .iter()
        .enumerate()
        .map(|(index, channel)| {
            let mut cursor = guide.from_ms;
            let mut cells = Vec::new();
            for (slot, programme) in channel.programmes.iter().enumerate() {
                let start = programme.start_ms.max(cursor);
                let end = programme.end_ms.min(guide.to_ms);
                if end <= start {
                    continue;
                }
                if start > cursor {
                    cells.push(serde_json::json!({
                        "blank": true,
                        "width": scale(start - cursor).to_string(),
                        "title": "",
                        "time": "",
                        "node": "0",
                    }));
                }
                let live = programme.start_ms <= guide.now_ms && guide.now_ms < programme.end_ms;
                cells.push(serde_json::json!({
                    "blank": false,
                    "width": (scale(end - start) - 4.0).max(4.0).to_string(),
                    "title": programme.title,
                    "time": clock(programme.start_ms, guide.utc_offset_ms),
                    "class": if live { "guide-cell-live" } else { "guide-cell" },
                    "node": (NODE_GUIDE_CELL_BASE + index as u64 * NODE_GUIDE_ROW_STRIDE + slot as u64).to_string(),
                }));
                cursor = end;
            }
            serde_json::json!({
                "name": channel.name,
                "logo": channel.logo,
                "node": (NODE_GUIDE_CHANNEL_BASE + index as u64).to_string(),
                "cells": cells,
            })
        })
        .collect();
    serde_json::json!({
        "title": localized("nav.live_tv", language),
        "hours": hours,
        "span": span.to_string(),
        "nowX": scale(guide.now_ms - guide.from_ms).clamp(0.0, span).to_string(),
        "labelWidth": GUIDE_LABEL_WIDTH.to_string(),
        "channels": channels,
        "loading": guide.loading,
        "failed": guide.failed,
        "empty": guide.channels.is_empty() && !guide.loading,
        "nowOffset": scale(guide.now_ms - guide.from_ms).to_string(),
    })
}
