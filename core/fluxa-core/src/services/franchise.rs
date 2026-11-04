use super::str_field;
use serde_json::{Value, json};

const ENDPOINT: &str = "https://graphql.anilist.co";
const QUERY: &str = "query($ids:[Int]){Page(perPage:50){media(id_in:$ids,type:ANIME){id format startDate{year month day} title{romaji english} description(asHtml:false) averageScore coverImage{extraLarge} bannerImage relations{edges{relationType node{id type}}}}}}";
const FOLLOWED: [&str; 6] = [
    "PREQUEL",
    "SEQUEL",
    "PARENT",
    "SIDE_STORY",
    "SPIN_OFF",
    "ALTERNATIVE",
];

pub(crate) fn detail_franchise_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let out = match str_field(&args, "step") {
        "tmdbCollectionId" => json!({"id": args.pointer("/detail/belongs_to_collection/id")}),
        "tmdbParts" => json!({"parts": tmdb_parts(&args)?}),
        "anilistRequest" => anilist_request(args.get("ids")?),
        "anilistStep" => anilist_step(&args)?,
        "anilistCards" => json!(anilist_cards(&args)?),
        _ => return None,
    };
    serde_json::to_string(&out).ok()
}

fn tmdb_parts(args: &Value) -> Option<Vec<Value>> {
    let current = args.get("current").and_then(Value::as_u64);
    let mut parts: Vec<Value> = args
        .pointer("/collection/parts")?
        .as_array()?
        .iter()
        .filter(|part| part.get("id").and_then(Value::as_u64) != current)
        .cloned()
        .collect();
    parts.sort_by_key(|part| {
        let date = str_field(part, "release_date");
        (date.is_empty(), date.to_owned())
    });
    Some(parts)
}

fn anilist_request(ids: &Value) -> Value {
    json!({
        "method": "POST",
        "url": ENDPOINT,
        "headers": {"Content-Type": "application/json", "Accept": "application/json"},
        "body": {"query": QUERY, "variables": {"ids": ids}},
    })
}

fn anilist_step(args: &Value) -> Option<Value> {
    let media = args.pointer("/response/data/Page/media")?.as_array()?;
    let seen: Vec<u64> = args
        .get("seen")
        .and_then(Value::as_array)
        .map(|ids| ids.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default();
    let mut next = Vec::new();
    for edge in media
        .iter()
        .filter_map(|item| item.pointer("/relations/edges")?.as_array())
        .flatten()
    {
        let followed = FOLLOWED.contains(&str_field(edge, "relationType"));
        let anime = edge.pointer("/node/type").and_then(Value::as_str) == Some("ANIME");
        let Some(id) = edge.pointer("/node/id").and_then(Value::as_u64) else {
            continue;
        };
        if followed && anime && !seen.contains(&id) && !next.contains(&id) {
            next.push(id);
        }
    }
    Some(json!({"media": media, "next": next}))
}

fn start(media: &Value) -> (u64, u64, u64) {
    let part = |key: &str| {
        media
            .pointer(&format!("/startDate/{key}"))
            .and_then(Value::as_u64)
    };
    match part("year") {
        Some(year) => (year, part("month").unwrap_or(1), part("day").unwrap_or(1)),
        None => (u64::MAX, 0, 0),
    }
}

fn format_label(format: &str) -> Option<&'static str> {
    Some(match format {
        "TV" => "TV",
        "TV_SHORT" => "TV Short",
        "MOVIE" => "Movie",
        "SPECIAL" => "Special",
        "OVA" => "OVA",
        "ONA" => "ONA",
        _ => return None,
    })
}

fn anilist_cards(args: &Value) -> Option<Vec<Value>> {
    let current = args.get("current").and_then(Value::as_u64);
    let mut media: Vec<&Value> = args
        .get("media")?
        .as_array()?
        .iter()
        .filter(|item| item.get("id").and_then(Value::as_u64) != current)
        .collect();
    media.sort_by_key(|item| (start(item), item.get("id").and_then(Value::as_u64)));
    media.dedup_by_key(|item| item.get("id").and_then(Value::as_u64));
    Some(
        media
            .into_iter()
            .filter_map(|item| {
                let id = item.get("id").and_then(Value::as_u64)?;
                let format = format_label(str_field(item, "format"))?;
                let title = item.get("title")?;
                let name = [str_field(title, "english"), str_field(title, "romaji")]
                    .into_iter()
                    .find(|name| !name.is_empty())?;
                let year = item.pointer("/startDate/year").and_then(Value::as_u64);
                Some(json!({
                    "id": format!("anilist:{id}"),
                    "type": if format == "Movie" { "movie" } else { "series" },
                    "name": name,
                    "poster": item.pointer("/coverImage/extraLarge"),
                    "background": item.get("bannerImage"),
                    "description": item.get("description"),
                    "year": year,
                    "releaseLabel": match year {
                        Some(year) => format!("{year} · {format}"),
                        None => format.to_owned(),
                    },
                }))
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: Value) -> Value {
        serde_json::from_str(&detail_franchise_json(&args.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn collection_parts_are_dated_oldest_first_without_the_current_title() {
        let parts = run(json!({
            "step": "tmdbParts",
            "current": 2,
            "collection": {"parts": [
                {"id": 3, "release_date": "2012-05-01"},
                {"id": 4, "release_date": ""},
                {"id": 2, "release_date": "2010-01-01"},
                {"id": 1, "release_date": "2008-04-30"},
            ]}
        }));
        let ids: Vec<u64> = parts["parts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|part| part["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, [1, 3, 4]);
    }

    #[test]
    fn anilist_step_follows_only_story_relations_to_unseen_anime() {
        let response = json!({"data": {"Page": {"media": [{"id": 1, "relations": {"edges": [
            {"relationType": "SEQUEL", "node": {"id": 2, "type": "ANIME"}},
            {"relationType": "ADAPTATION", "node": {"id": 3, "type": "MANGA"}},
            {"relationType": "PREQUEL", "node": {"id": 4, "type": "ANIME"}},
            {"relationType": "CHARACTER", "node": {"id": 5, "type": "ANIME"}},
            {"relationType": "SIDE_STORY", "node": {"id": 6, "type": "MANGA"}},
        ]}}]}}});
        let step = run(json!({"step": "anilistStep", "response": response, "seen": [4]}));
        assert_eq!(step["next"], json!([2]));
    }

    #[test]
    fn anilist_cards_run_chronologically_and_skip_music_and_current() {
        let media = json!([
            {"id": 2, "format": "MOVIE", "startDate": {"year": 2020}, "title": {"romaji": "Second"}},
            {"id": 1, "format": "TV", "startDate": {"year": 2018, "month": 4}, "title": {"english": "First", "romaji": "Dai"}},
            {"id": 3, "format": "MUSIC", "startDate": {"year": 2019}, "title": {"romaji": "Song"}},
            {"id": 9, "format": "TV", "startDate": {"year": 2021}, "title": {"romaji": "Current"}},
        ]);
        let cards = run(json!({"step": "anilistCards", "media": media, "current": 9}));
        let names: Vec<&str> = cards
            .as_array()
            .unwrap()
            .iter()
            .map(|card| card["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["First", "Second"]);
        assert_eq!(cards[1]["type"], "movie");
        assert_eq!(cards[0]["releaseLabel"], "2018 · TV");
    }
}
