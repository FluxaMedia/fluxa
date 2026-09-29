use crate::services::*;

pub(crate) const ANILIST_GRAPHQL: &str = "https://graphql.anilist.co";
pub(crate) const ANILIST_REDIRECT_URI: &str = "fluxa://oauth/anilist";
pub(crate) const ANILIST_SAVE_MUTATION: &str = "mutation($mediaId:Int,$status:MediaListStatus,$progress:Int){SaveMediaListEntry(mediaId:$mediaId,status:$status,progress:$progress){id}}";
pub(crate) const ANILIST_LIST_QUERY: &str = "query($userId:Int,$chunk:Int){MediaListCollection(userId:$userId,type:ANIME,forceSingleCompletedList:true,chunk:$chunk,perChunk:500){hasNextChunk lists{isCustomList entries{status progress updatedAt media{id idMal format episodes seasonYear genres title{english romaji native} coverImage{extraLarge large} bannerImage}}}}}";

pub(crate) fn anilist_user_id(token: &str) -> Option<i64> {
    use base64::Engine;
    let payload = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .ok()?;
    let claims: Value = serde_json::from_slice(&bytes).ok()?;
    claims.get("sub")?.as_str()?.parse().ok()
}

pub(crate) fn anilist_snapshot(responses: &Value, now_ms: i64) -> Value {
    let entries: Vec<Value> = ["list_1", "list_2", "list_3"]
        .into_iter()
        .filter_map(|key| responses.pointer(&format!("/{key}/data/MediaListCollection/lists")))
        .filter_map(Value::as_array)
        .flatten()
        .filter(|list| list.get("isCustomList").and_then(Value::as_bool) != Some(true))
        .filter_map(|list| list.get("entries").and_then(Value::as_array))
        .flatten()
        .cloned()
        .collect();
    let synced = crate::services::anilist::anilist_entries_to_sync(&entries, now_ms, None, false);
    let tagged = |key: &str| {
        let items: Vec<Value> = synced
            .get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|mut item| {
                item["source"] = json!("anilist");
                item
            })
            .collect();
        Value::Array(items)
    };
    json!({
        "watchlist": tagged("watchlist"),
        "liked": [],
        "completed": tagged("completed"),
        "watched": synced.get("watched").cloned().unwrap_or(json!({})),
        "dropped": tagged("dropped"),
        "onHold": tagged("onHold"),
        "continueWatching": tagged("watching"),
    })
}

pub(crate) fn anilist_calendar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let year = args.get("year")?.as_i64()?;
    let month = args.get("month")?.as_i64()?;
    let plan = |key: &str, filter: String, schedule: &str| {
        let query = format!(
            "query{{Page(perPage:50){{media(type:ANIME,onList:true,{filter}){{id title{{romaji english}} coverImage{{large}} airingSchedule({schedule}perPage:50){{nodes{{airingAt episode}}}}}}}}}}"
        );
        let mut plan = request(
            "anilist",
            &args,
            "POST",
            ANILIST_GRAPHQL.to_owned(),
            json!({"query": query}),
        );
        plan["key"] = json!(key);
        plan
    };
    let month_start = year * 10_000 + month * 100;
    serde_json::to_string(&json!([
        plan(
            "releasing",
            "status:RELEASING".to_owned(),
            "notYetAired:true,"
        ),
        plan(
            "finished",
            format!("status:FINISHED,endDate_greater:{month_start}"),
            ""
        ),
    ]))
    .ok()
}

pub(crate) fn anilist_save(args: &Value, content_id: &str, fields: Value) -> Option<Value> {
    let media_id: i64 = content_id
        .strip_prefix("anilist:")?
        .split(':')
        .next()?
        .parse()
        .ok()?;
    let mut variables = fields;
    variables["mediaId"] = json!(media_id);
    Some(request(
        "anilist",
        args,
        "POST",
        ANILIST_GRAPHQL.to_owned(),
        json!({"query": ANILIST_SAVE_MUTATION, "variables": variables}),
    ))
}

pub(crate) fn anilist_library_requests(args: &Value) -> Option<Vec<Value>> {
    let user_id = anilist_user_id(str_field(args, "token"))?;
    Some(
        (1..=3)
            .map(|chunk| {
                let mut plan = request(
                    "anilist",
                    args,
                    "POST",
                    ANILIST_GRAPHQL.to_owned(),
                    json!({"query": ANILIST_LIST_QUERY, "variables": {"userId": user_id, "chunk": chunk}}),
                );
                plan["key"] = json!(format!("list_{chunk}"));
                plan
            })
            .collect(),
    )
}

pub(crate) fn anilist_library_snapshot(args: &Value, responses: &Value) -> Value {
    anilist_snapshot(
        responses,
        args.get("nowSeconds").and_then(Value::as_i64).unwrap_or(0) * 1000,
    )
}

pub(crate) fn anilist_toggle_watchlist(
    args: &Value,
    _item: &Value,
    id: &str,
    remove: bool,
) -> Option<Value> {
    if remove {
        return None;
    }
    anilist_save(args, id, json!({"status": "PLANNING"}))
}

pub(crate) fn anilist_mark_watched(args: &Value, change: &WatchedChange) -> Option<Value> {
    if !change.watched {
        return None;
    }
    let episode = change
        .video_ids
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|id| id.matches(':').count() >= 2)
        .filter_map(|id| id.rsplit(':').next()?.parse::<i64>().ok())
        .max();
    let fields = match episode {
        Some(episode) => json!({"status": "CURRENT", "progress": episode}),
        None => json!({"status": "COMPLETED"}),
    };
    anilist_save(args, change.series_id, fields)
}

pub(crate) fn anilist_auth_request(args: &Value, operation: &str) -> Option<Value> {
    if operation != "exchange" {
        return None;
    }
    Some(request(
        "anilist",
        args,
        "POST",
        "https://anilist.co/api/v2/oauth/token".to_owned(),
        json!({
            "grant_type": "authorization_code",
            "client_id": str_field(args, "clientId"),
            "client_secret": str_field(args, "clientSecret"),
            "redirect_uri": ANILIST_REDIRECT_URI,
            "code": str_field(args, "code"),
        }),
    ))
}

pub(crate) fn anilist_token_state(status: u16, body: &Value) -> &'static str {
    if (200..300).contains(&status) && body.get("access_token").is_some() {
        "success"
    } else {
        "error"
    }
}

pub(crate) fn anilist_authorize_url(args: &Value) -> Option<String> {
    let url = url::Url::parse_with_params(
        "https://anilist.co/api/v2/oauth/authorize",
        [
            ("client_id", str_field(args, "clientId")),
            ("response_type", "code"),
            ("redirect_uri", ANILIST_REDIRECT_URI),
            ("state", str_field(args, "state")),
        ],
    )
    .ok()?;
    Some(url.to_string())
}

pub(crate) fn anilist_headers(_client_id: &str) -> Vec<(&'static str, String)> {
    vec![("Accept", "application/json".to_string())]
}
