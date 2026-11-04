use super::helpers::usable_artwork;
use serde_json::{Value, json};

pub(crate) fn addon_release_items_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let prefix = request.get("monthPrefix").and_then(Value::as_str)?;
    let library = request.get("items").and_then(Value::as_array)?;
    let metas = request.get("metas").and_then(Value::as_object)?;
    let mut items = Vec::new();
    for entry in library {
        let Some(id) = entry.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some(meta) = metas.get(id) else { continue };
        let title = meta.get("name").or_else(|| entry.get("name"));
        let poster = [meta.get("poster"), entry.get("poster")]
            .into_iter()
            .flatten()
            .find_map(|value| usable_artwork(value.as_str()))
            .map(str::to_string);
        let videos = meta.get("videos").and_then(Value::as_array);
        for video in videos.into_iter().flatten() {
            let Some(date) = release_date(video.get("released"), prefix) else {
                continue;
            };
            let season = video.get("season").and_then(Value::as_i64).unwrap_or(1);
            if season == 0 {
                continue;
            }
            let number = video
                .get("episode")
                .or_else(|| video.get("number"))
                .and_then(Value::as_i64);
            let thumbnail = usable_artwork(video.get("thumbnail").and_then(Value::as_str));
            items.push(json!({
                "id": format!("{id}:{season}:{}", number.unwrap_or_default()),
                "title": title,
                "episodeTitle": video.get("name").or_else(|| video.get("title")),
                "seasonNumber": season,
                "episodeNumber": number,
                "dateIso": date,
                "contentId": id,
                "seriesId": id,
                "metaType": "series",
                "poster": thumbnail.or(poster.as_deref()),
                "episodePoster": thumbnail,
                "seriesPoster": poster,
            }));
        }
    }
    serde_json::to_string(&items).ok()
}

fn release_date<'a>(released: Option<&'a Value>, prefix: &str) -> Option<&'a str> {
    released?.as_str().filter(|date| date.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_episodes_released_in_the_month() {
        let request = json!({
            "monthPrefix": "2026-11",
            "items": [{"id": "tt1", "type": "series", "name": "Example Show"}],
            "metas": {"tt1": {"videos": [
                {"season": 1, "episode": 1, "released": "2026-10-28T00:00:00.000Z"},
                {"season": 1, "episode": 2, "released": "2026-11-04T00:00:00.000Z"},
                {"season": 0, "episode": 1, "released": "2026-11-05T00:00:00.000Z"}
            ]}}
        });
        let out: Vec<Value> =
            serde_json::from_str(&addon_release_items_json(&request.to_string()).unwrap()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["id"], "tt1:1:2");
        assert_eq!(out[0]["title"], "Example Show");
    }
}
