use serde_json::{Value, json};

fn meta_link_kind(category: &str) -> Option<&'static str> {
    let category = category.to_lowercase();
    if ["cast", "actor", "actors", "starring"]
        .iter()
        .any(|key| category.contains(key))
    {
        Some("cast")
    } else if category.contains("director") {
        Some("director")
    } else {
        None
    }
}

pub(crate) fn classify_meta_links_json(links_json: &str) -> Option<String> {
    let links: Vec<Value> = serde_json::from_str(links_json).ok()?;
    let mut cast: Vec<Value> = Vec::new();
    let mut directors: Vec<Value> = Vec::new();
    for link in links {
        let category = link.get("category").and_then(Value::as_str).unwrap_or("");
        match meta_link_kind(category) {
            Some("cast") => cast.push(link),
            Some("director") => directors.push(link),
            _ => {}
        }
    }
    serde_json::to_string(&json!({ "cast": cast, "directors": directors })).ok()
}
