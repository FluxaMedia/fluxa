use super::{request, str_field};
use serde_json::{Value, json};

const TRAKT_API: &str = "https://api.trakt.tv";
const SIMKL_API: &str = "https://api.simkl.com";
const LIMIT: usize = 16;

pub(crate) fn detail_similar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let movie = str_field(&args, "contentType") == "movie";
    let id = str_field(&args, "id").split(':').next().unwrap_or_default();
    let get = |url: String| request(provider, &args, "GET", url, Value::Null);
    let plan = match (provider, str_field(&args, "step")) {
        ("trakt", _) => {
            if !id.starts_with("tt") {
                return None;
            }
            let kind = if movie { "movies" } else { "shows" };
            get(format!(
                "{TRAKT_API}/{kind}/{id}/related?extended=full,images&limit={LIMIT}"
            ))
        }
        ("simkl", "lookup") => {
            if !id.starts_with("tt") {
                return None;
            }
            get(format!("{SIMKL_API}/search/id?imdb={id}"))
        }
        ("simkl", "detail") => {
            let simkl = args.get("simklId")?.as_i64()?;
            let kind = if movie { "movies" } else { "tv" };
            get(format!("{SIMKL_API}/{kind}/{simkl}"))
        }
        ("simkl", "resolve") => {
            let simkl = args.get("simklId")?.as_i64()?;
            get(format!("{SIMKL_API}/search/id?simkl={simkl}"))
        }
        ("simkl", "meta") => {
            let simkl = args.get("simklId")?.as_i64()?;
            let kind = if movie { "movies" } else { "tv" };
            get(format!("{SIMKL_API}/{kind}/{simkl}?extended=full"))
        }
        _ => return None,
    };
    serde_json::to_string(&plan).ok()
}

pub(crate) fn detail_similar_items_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let response = args.get("response")?;
    let movie = str_field(&args, "contentType") == "movie";
    let kind = if movie { "movie" } else { "series" };
    let out = match (str_field(&args, "provider"), str_field(&args, "step")) {
        ("trakt", _) => json!(
            response
                .as_array()?
                .iter()
                .filter_map(|item| trakt_card(item, kind))
                .take(LIMIT)
                .collect::<Vec<_>>()
        ),
        ("simkl", "lookup") => {
            let simkl = response
                .as_array()?
                .iter()
                .find_map(|item| item.pointer("/ids/simkl")?.as_i64())?;
            json!({"simklId": simkl})
        }
        ("simkl", "detail") => json!(
            response
                .get("users_recommendations")?
                .as_array()?
                .iter()
                .filter_map(|item| {
                    Some(json!({
                        "simklId": item.pointer("/ids/simkl")?.as_i64()?,
                        "name": item.get("title")?,
                        "year": item.get("year"),
                        "poster": item.get("poster").and_then(Value::as_str).map(simkl_poster),
                    }))
                })
                .take(LIMIT)
                .collect::<Vec<_>>()
        ),
        ("simkl", "resolve") => {
            let ids = response.as_array()?.first()?.get("ids")?;
            let id = ids
                .get("imdb")
                .and_then(Value::as_str)
                .filter(|id| id.starts_with("tt"))
                .map(str::to_owned)
                .or_else(|| Some(format!("tmdb:{}", ids.get("tmdb")?.as_i64()?)))?;
            json!({"id": id})
        }
        ("simkl", "meta") => json!({
            "description": response.get("overview"),
            "genres": response.get("genres"),
            "runtime": response.get("runtime").and_then(Value::as_i64).map(|minutes| format!("{minutes}min")),
            "background": response.get("fanart").and_then(Value::as_str).map(|path| format!("https://simkl.in/fanart/{path}_w.jpg")),
        }),
        _ => return None,
    };
    serde_json::to_string(&out).ok()
}

fn title_case(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn trakt_card(item: &Value, kind: &str) -> Option<Value> {
    let ids = item.get("ids")?;
    let id = ids
        .get("imdb")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| Some(format!("tmdb:{}", ids.get("tmdb")?.as_i64()?)))?;
    let poster = item
        .get("images")
        .and_then(|images| super::trakt::trakt_image_url(images, "poster"));
    Some(json!({
        "id": id,
        "type": kind,
        "name": item.get("title")?,
        "year": item.get("year"),
        "poster": poster,
        "background": item.get("images").and_then(|images| super::trakt::trakt_image_url(images, "fanart")),
        "logo": item.get("images").and_then(|images| super::trakt::trakt_image_url(images, "logo")),
        "description": item.get("overview").and_then(Value::as_str).filter(|text| !text.is_empty()),
        "genres": item.get("genres").and_then(Value::as_array).map(|genres| {
            genres.iter().filter_map(Value::as_str).map(title_case).collect::<Vec<_>>()
        }),
        "runtime": item.get("runtime").and_then(Value::as_i64).filter(|minutes| *minutes > 0).map(|minutes| format!("{minutes}min")),
    }))
}

fn simkl_poster(path: &str) -> String {
    format!("https://simkl.in/posters/{path}_m.jpg")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: fn(&str) -> Option<String>, args: Value) -> Option<Value> {
        serde_json::from_str(&f(&args.to_string())?).ok()
    }

    #[test]
    fn trakt_related_needs_an_imdb_id_and_maps_posters() {
        let plan = run(
            detail_similar_plan_json,
            json!({"provider": "trakt", "contentType": "movie", "id": "tt1375666", "clientId": "c"}),
        )
        .unwrap();
        assert_eq!(
            plan["url"],
            "https://api.trakt.tv/movies/tt1375666/related?extended=full,images&limit=16"
        );
        assert!(
            run(
                detail_similar_plan_json,
                json!({"provider": "trakt", "contentType": "movie", "id": "tmdb:5"})
            )
            .is_none()
        );
        let cards = run(
            detail_similar_items_json,
            json!({"provider": "trakt", "contentType": "series", "response": [
                {"title": "Dark", "year": 2017, "ids": {"imdb": "tt5753856", "tmdb": 70523},
                 "images": {"poster": ["media.trakt.tv/p.jpg"]}},
                {"title": "No ids", "ids": {}},
            ]}),
        )
        .unwrap();
        assert_eq!(cards.as_array().unwrap().len(), 1);
        assert_eq!(cards[0]["id"], "tt5753856");
        assert_eq!(cards[0]["type"], "series");
        assert_eq!(cards[0]["poster"], "https://media.trakt.tv/p.jpg");
    }

    #[test]
    fn simkl_recommendations_resolve_to_imdb_ids_in_three_steps() {
        let lookup = run(
            detail_similar_items_json,
            json!({"provider": "simkl", "step": "lookup", "contentType": "movie",
                   "response": [{"ids": {"simkl": 53992}}]}),
        )
        .unwrap();
        assert_eq!(lookup["simklId"], 53992);
        let recs = run(
            detail_similar_items_json,
            json!({"provider": "simkl", "step": "detail", "contentType": "movie",
                   "response": {"users_recommendations": [
                       {"title": "Tenet", "year": 2020, "poster": "12/abc", "ids": {"simkl": 7}}]}}),
        )
        .unwrap();
        assert_eq!(recs[0]["simklId"], 7);
        assert_eq!(recs[0]["poster"], "https://simkl.in/posters/12/abc_m.jpg");
        let resolved = run(
            detail_similar_items_json,
            json!({"provider": "simkl", "step": "resolve", "contentType": "movie",
                   "response": [{"ids": {"simkl": 7, "imdb": "tt6723592"}}]}),
        )
        .unwrap();
        assert_eq!(resolved["id"], "tt6723592");
    }
}
