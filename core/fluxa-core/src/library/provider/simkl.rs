use super::*;

pub(super) fn simkl_bucket(responses: &Value, status: &str) -> (String, String) {
    let shows = concat(&[
        responses
            .get(&format!("{status}_shows"))
            .and_then(|value| value.get("shows"))
            .cloned()
            .unwrap_or(json!([])),
        responses
            .get(&format!("{status}_anime"))
            .and_then(|value| value.get("anime"))
            .and_then(Value::as_array)
            .map(|anime| {
                Value::Array(
                    anime
                        .iter()
                        .map(|entry| {
                            let mut entry = entry.clone();
                            entry["anime"] = json!(true);
                            if let Some(show) = entry.get("show").or_else(|| entry.get("anime")) {
                                entry["show"] = show.clone();
                            }
                            entry
                        })
                        .collect(),
                )
            })
            .unwrap_or(json!([])),
    ]);
    let movies = responses
        .get(&format!("{status}_movies"))
        .and_then(|value| value.get("movies"))
        .cloned()
        .unwrap_or(json!([]));
    (shows.to_string(), movies.to_string())
}


