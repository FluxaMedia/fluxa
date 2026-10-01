use serde_json::{Value, json};

const WATCH_NEXT_LIMIT: usize = 10;
const ROW_LIMIT: usize = 3;
const ROW_ITEM_LIMIT: usize = 15;

fn text<'a>(item: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .filter_map(|key| item.get(key).and_then(Value::as_str))
        .find(|value| !value.trim().is_empty())
}

fn image<'a>(item: &'a Value, keys: &[&str]) -> Option<&'a str> {
    text(item, keys).filter(|url| url.starts_with("http"))
}

fn is_continue(category: &Value) -> bool {
    category.get("id").and_then(Value::as_str) == Some("continue_watching")
        || category.get("type").and_then(Value::as_str) == Some("continue_watching")
}

fn watch_next(item: &Value) -> Option<Value> {
    let id = text(item, &["id"])?;
    let kind = text(item, &["type"])?;
    let title = text(item, &["name", "title"])?;
    let seconds = |key: &str| item.get(key).and_then(Value::as_f64).unwrap_or(0.0);
    Some(json!({
        "id": id,
        "type": kind,
        "title": title,
        "subtitle": text(item, &["episodeLabel", "lastEpisodeName"]),
        "image": image(item, &[
            "continueWatchingBackground",
            "lastEpisodeThumbnail",
            "background",
            "backgroundUrl",
            "poster",
            "posterUrl",
        ]),
        "positionMs": (seconds("timeOffset") * 1000.0) as i64,
        "durationMs": (seconds("duration") * 1000.0) as i64,
    }))
}

fn catalog_item(item: &Value) -> Option<Value> {
    let image = image(item, &["poster", "posterUrl", "resolvedPosterUrl"])?;
    Some(json!({
        "id": text(item, &["id"])?,
        "type": text(item, &["type"])?,
        "title": text(item, &["name", "title"])?,
        "image": image,
        "backdrop": image_or_none(item),
    }))
}

fn image_or_none(item: &Value) -> Option<&str> {
    image(
        item,
        &["background", "backgroundUrl", "backdrop", "backdropUrl"],
    )
}

pub(crate) fn feed(snapshot: &Value) -> Value {
    let home = snapshot.get("home").unwrap_or(&Value::Null);
    let categories = home
        .get("categories")
        .or_else(|| home.get("rows"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let continuing = home
        .get("continueWatching")
        .and_then(Value::as_array)
        .or_else(|| {
            categories
                .iter()
                .find(|category| is_continue(category))
                .and_then(|category| category.get("items"))
                .and_then(Value::as_array)
        });
    let watching: Vec<Value> = continuing
        .into_iter()
        .flatten()
        .filter_map(watch_next)
        .take(WATCH_NEXT_LIMIT)
        .collect();
    let rows: Vec<Value> = categories
        .iter()
        .filter(|category| !is_continue(category))
        .filter_map(|category| {
            let items: Vec<Value> = category
                .get("items")?
                .as_array()?
                .iter()
                .filter_map(catalog_item)
                .take(ROW_ITEM_LIMIT)
                .collect();
            let title = text(category, &["homeTitle", "name", "label", "title"])?;
            (!items.is_empty())
                .then(|| json!({"id": text(category, &["id"]), "title": title, "items": items}))
        })
        .take(ROW_LIMIT)
        .collect();
    json!({"watchNext": watching, "rows": rows})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_row_is_not_a_catalog_row() {
        let snapshot = json!({"home": {"categories": [
            {"id": "continue_watching", "items": [
                {"id": "tt1", "type": "movie", "name": "A", "timeOffset": 60, "duration": 600, "poster": "https://x/a.jpg"}
            ]},
            {"id": "top", "name": "Top", "items": [
                {"id": "tt2", "type": "movie", "name": "B", "poster": "https://x/b.jpg"},
                {"id": "tt3", "type": "movie", "name": "C"}
            ]}
        ]}});
        let feed = feed(&snapshot);
        assert_eq!(feed["watchNext"].as_array().unwrap().len(), 1);
        assert_eq!(feed["watchNext"][0]["positionMs"], 60000);
        assert_eq!(feed["rows"].as_array().unwrap().len(), 1);
        assert_eq!(feed["rows"][0]["items"].as_array().unwrap().len(), 1);
    }
}
