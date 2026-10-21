use serde_json::Value;

pub const TIMEOUT_MINUTES: [u32; 6] = [1, 2, 5, 10, 15, 30];
pub const DEFAULT_TIMEOUT_MINUTES: u32 = 5;
pub const MAX_CATALOGS: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slide {
    pub id: String,
    pub title: String,
    pub backdrop: String,
    pub logo: Option<String>,
    pub genres: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogOption {
    pub key: String,
    pub name: String,
    pub content_type: String,
    pub transport_url: String,
    pub catalog_id: String,
}

pub fn timeout_minutes(raw: Option<&str>) -> u32 {
    raw.and_then(|value| value.trim().parse().ok())
        .filter(|minutes| TIMEOUT_MINUTES.contains(minutes))
        .unwrap_or(DEFAULT_TIMEOUT_MINUTES)
}

pub fn catalog_key(transport_url: &str, content_type: &str, catalog_id: &str) -> String {
    format!("{transport_url}|{content_type}|{catalog_id}")
}

pub fn catalog_options(addons: &[Value]) -> Vec<CatalogOption> {
    let mut options = Vec::new();
    for addon in addons {
        if addon.get("enabled").and_then(Value::as_bool) == Some(false) {
            continue;
        }
        let manifest = addon
            .get("manifest")
            .filter(|manifest| manifest.is_object())
            .unwrap_or(addon);
        let Some(transport_url) = addon
            .get("transportUrl")
            .or_else(|| manifest.get("transportUrl"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let addon_name = manifest
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let catalogs = manifest
            .get("catalogs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for catalog in catalogs {
            if requires_extra(catalog) {
                continue;
            }
            let (Some(id), Some(content_type)) = (
                catalog.get("id").and_then(Value::as_str),
                catalog.get("type").and_then(Value::as_str),
            ) else {
                continue;
            };
            let name = catalog
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or(id);
            options.push(CatalogOption {
                key: catalog_key(transport_url, content_type, id),
                name: if addon_name.is_empty() {
                    format!("{name} ({content_type})")
                } else {
                    format!("{name} ({content_type}) · {addon_name}")
                },
                content_type: content_type.to_owned(),
                transport_url: transport_url.to_owned(),
                catalog_id: id.to_owned(),
            });
        }
    }
    options
}

fn requires_extra(catalog: &Value) -> bool {
    let required = |item: &Value| item.get("isRequired").and_then(Value::as_bool) == Some(true);
    catalog
        .get("extra")
        .and_then(Value::as_array)
        .is_some_and(|extra| extra.iter().any(required))
        || catalog
            .get("extraRequired")
            .and_then(Value::as_array)
            .is_some_and(|names| !names.is_empty())
}

pub fn toggle_catalog(selected: &[String], key: &str) -> Vec<String> {
    if selected.iter().any(|item| item == key) {
        return selected
            .iter()
            .filter(|item| *item != key)
            .cloned()
            .collect();
    }
    if selected.len() >= MAX_CATALOGS {
        return selected.to_vec();
    }
    let mut next = selected.to_vec();
    next.push(key.to_owned());
    next
}

pub fn slides(items: &[Value], seed: u64) -> Vec<Slide> {
    let mut slides: Vec<Slide> = items.iter().filter_map(slide).collect();
    shuffle(&mut slides, seed);
    slides
}

fn text(item: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        item.get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn slide(item: &Value) -> Option<Slide> {
    let backdrop = text(
        item,
        &["background", "backgroundUrl", "backdrop", "backdropUrl"],
    )?;
    let title = text(item, &["name", "title"]).unwrap_or_default();
    let id = text(item, &["id"]).unwrap_or_else(|| backdrop.clone());
    let genres = item
        .get("genres")
        .or_else(|| item.get("genre"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|genre| !genre.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    Some(Slide {
        id,
        title,
        backdrop,
        logo: text(item, &["logo", "logoUrl", "clearLogo"]),
        genres,
    })
}

pub fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed | 1;
    for index in (1..items.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        items.swap(index, (state % (index as u64 + 1)) as usize);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn unknown_timeout_falls_back_to_default() {
        assert_eq!(timeout_minutes(Some("7")), 5);
        assert_eq!(timeout_minutes(Some("15")), 15);
        assert_eq!(timeout_minutes(None), 5);
    }

    #[test]
    fn catalogs_needing_extra_input_are_excluded() {
        let addons = [json!({
            "transportUrl": "https://a/manifest.json",
            "manifest": {"name": "A", "catalogs": [
                {"id": "top", "type": "movie", "name": "Top"},
                {"id": "search", "type": "movie", "extra": [{"name": "search", "isRequired": true}]},
                {"id": "genre", "type": "series", "extra": [{"name": "genre"}]}
            ]}
        })];
        let keys: Vec<_> = catalog_options(&addons)
            .into_iter()
            .map(|option| option.catalog_id)
            .collect();
        assert_eq!(keys, ["top", "genre"]);
    }

    #[test]
    fn selection_stops_at_three() {
        let full: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        assert_eq!(toggle_catalog(&full, "d"), full);
        assert_eq!(toggle_catalog(&full, "b"), ["a", "c"]);
    }

    #[test]
    fn slides_without_backdrop_are_dropped() {
        let items = [
            json!({"id": "1", "name": "One", "background": "https://x/1.jpg", "genres": ["Action"]}),
            json!({"id": "2", "name": "Two", "poster": "https://x/2.jpg"}),
        ];
        let slides = slides(&items, 7);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides[0].genres, ["Action"]);
    }
}
