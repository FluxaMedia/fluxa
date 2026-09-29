use super::*;

const PLEX_TV: &str = "https://plex.tv";

fn headers(server: &Server) -> Vec<(&'static str, String)> {
    vec![
        ("Accept", "application/json".to_owned()),
        ("X-Plex-Token", server.token.clone()),
        ("X-Plex-Client-Identifier", server.device_id.clone()),
        ("X-Plex-Product", PRODUCT.to_owned()),
        ("X-Plex-Version", env!("CARGO_PKG_VERSION").to_owned()),
        ("X-Plex-Platform", PRODUCT.to_owned()),
        ("X-Plex-Device-Name", PRODUCT.to_owned()),
    ]
}

fn call(server: &Server, method: &str, url: String, pairs: &[(&str, String)]) -> Value {
    let url = if pairs.is_empty() {
        url
    } else {
        format!("{url}?{}", query(pairs))
    };
    plan(method, url, headers(server), Value::Null)
}

fn get(server: &Server, path: &str, pairs: &[(&str, String)]) -> Value {
    call(server, "GET", format!("{}{path}", server.base), pairs)
}

fn paging(params: &Value) -> Vec<(&'static str, String)> {
    vec![
        (
            "X-Plex-Container-Start",
            param_i64(params, "skip", 0).to_string(),
        ),
        (
            "X-Plex-Container-Size",
            param_i64(params, "limit", 50).to_string(),
        ),
        ("includeGuids", "1".to_owned()),
    ]
}

fn plex_type(params: &Value) -> Option<&'static str> {
    match str_field(params, "kind") {
        "movie" => Some("1"),
        "series" => Some("2"),
        _ => None,
    }
}

pub(super) fn request(server: &Server, operation: &str, params: &Value) -> Option<Value> {
    let item = str_field(params, "itemId");
    let plan = match operation {
        "pinStart" => call(
            server,
            "POST",
            format!("{PLEX_TV}/api/v2/pins"),
            &[("strong", "true".to_owned())],
        ),
        "pinCheck" => call(
            server,
            "GET",
            format!("{PLEX_TV}/api/v2/pins/{}", str_field(params, "pinId")),
            &[],
        ),
        "servers" => call(
            server,
            "GET",
            format!("{PLEX_TV}/api/v2/resources"),
            &[
                ("includeHttps", "1".to_owned()),
                ("includeRelay", "1".to_owned()),
            ],
        ),
        "info" => get(server, "/", &[]),
        "libraries" => get(server, "/library/sections", &[]),
        "catalog" => {
            let mut pairs = paging(params);
            pairs.push(("sort", "titleSort".to_owned()));
            if let Some(genre) = Some(str_field(params, "genre")).filter(|genre| !genre.is_empty())
            {
                pairs.push(("genre", genre.to_owned()));
            }
            get(
                server,
                &format!("/library/sections/{}/all", str_field(params, "libraryId")),
                &pairs,
            )
        }
        "latest" => {
            let library = str_field(params, "libraryId");
            let path = if library.is_empty() {
                "/library/recentlyAdded".to_owned()
            } else {
                format!("/library/sections/{library}/recentlyAdded")
            };
            get(server, &path, &paging(params))
        }
        "resume" | "nextUp" => get(server, "/library/onDeck", &paging(params)),
        "search" => get(
            server,
            "/hubs/search",
            &[
                ("query", str_field(params, "query").to_owned()),
                ("limit", param_i64(params, "limit", 30).to_string()),
                ("includeGuids", "1".to_owned()),
            ],
        ),
        "lookup" => {
            external_id(str_field(params, "externalId"))?;
            let mut pairs = vec![
                ("includeGuids", "1".to_owned()),
                ("X-Plex-Container-Start", "0".to_owned()),
                ("X-Plex-Container-Size", "5000".to_owned()),
            ];
            if let Some(kind) = plex_type(params) {
                pairs.push(("type", kind.to_owned()));
            }
            get(server, "/library/all", &pairs)
        }
        "meta" | "streams" => get(
            server,
            &format!("/library/metadata/{item}"),
            &[("includeGuids", "1".to_owned())],
        ),
        "episodes" => get(server, &format!("/library/metadata/{item}/allLeaves"), &[]),
        "progress" => {
            let state = match str_field(params, "action") {
                "pause" => "paused",
                "stop" => "stopped",
                _ => "playing",
            };
            get(
                server,
                "/:/timeline",
                &[
                    ("ratingKey", item.to_owned()),
                    ("key", format!("/library/metadata/{item}")),
                    ("identifier", "com.plexapp.plugins.library".to_owned()),
                    ("state", state.to_owned()),
                    ("time", param_i64(params, "positionMs", 0).to_string()),
                    ("duration", param_i64(params, "durationMs", 0).to_string()),
                ],
            )
        }
        "markWatched" => {
            let path = if params.get("watched").and_then(Value::as_bool) == Some(false) {
                "/:/unscrobble"
            } else {
                "/:/scrobble"
            };
            get(
                server,
                path,
                &[
                    ("key", item.to_owned()),
                    ("identifier", "com.plexapp.plugins.library".to_owned()),
                ],
            )
        }
        _ => return None,
    };
    Some(plan)
}

fn container(body: &Value) -> &Value {
    body.get("MediaContainer").unwrap_or(&Value::Null)
}

fn asset(server: &Server, path: &str) -> Value {
    json!(format!(
        "{}{path}?{}",
        server.base,
        query(&[("X-Plex-Token", server.token.clone())])
    ))
}

fn tags(item: &Value, key: &str) -> Vec<Value> {
    list(item, key)
        .iter()
        .filter_map(|tag| tag.get("tag").cloned())
        .collect()
}

fn guid<'a>(item: &'a Value, scheme: &str) -> Option<&'a str> {
    let prefix = format!("{scheme}://");
    list(item, "Guid")
        .iter()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .find_map(|id| id.strip_prefix(prefix.as_str()))
}

fn meta(server: &Server, item: &Value) -> Value {
    let id = str_field(item, "ratingKey");
    let kind = str_field(item, "type");
    let episode = kind == "episode";
    let mut meta = Map::new();
    put(&mut meta, "id", Some(json!(server.item_id(id))));
    put(
        &mut meta,
        "type",
        Some(json!(if kind == "movie" { "movie" } else { "series" })),
    );
    let name = if episode {
        str_field(item, "grandparentTitle")
    } else {
        str_field(item, "title")
    };
    put(&mut meta, "name", Some(json!(name)));
    if episode {
        put(
            &mut meta,
            "episodeTitle",
            non_empty(str_field(item, "title")),
        );
        put(
            &mut meta,
            "season",
            int_field(item, "parentIndex").map(Value::from),
        );
        put(
            &mut meta,
            "episode",
            int_field(item, "index").map(Value::from),
        );
        put(
            &mut meta,
            "seriesId",
            non_empty(str_field(item, "grandparentRatingKey"))
                .map(|_| json!(server.item_id(str_field(item, "grandparentRatingKey")))),
        );
    }
    let poster = if episode {
        str_field(item, "grandparentThumb")
    } else {
        str_field(item, "thumb")
    };
    if !poster.is_empty() {
        put(&mut meta, "poster", Some(asset(server, poster)));
    }
    let art = str_field(item, "art");
    if !art.is_empty() {
        put(&mut meta, "background", Some(asset(server, art)));
    }
    let logo = list(item, "Image")
        .iter()
        .find(|image| str_field(image, "type") == "clearLogo")
        .map(|image| str_field(image, "url"))
        .filter(|url| !url.is_empty());
    if let Some(logo) = logo {
        put(&mut meta, "logo", Some(asset(server, logo)));
    }
    put(
        &mut meta,
        "description",
        non_empty(str_field(item, "summary")),
    );
    put(
        &mut meta,
        "releaseInfo",
        int_field(item, "year").map(|year| json!(year.to_string())),
    );
    put(
        &mut meta,
        "released",
        non_empty(str_field(item, "originallyAvailableAt")),
    );
    put(
        &mut meta,
        "imdbRating",
        float_field(item, "rating").map(|rating| json!(format!("{rating:.1}"))),
    );
    put(&mut meta, "imdb_id", guid(item, "imdb").map(|id| json!(id)));
    put(&mut meta, "tmdbId", guid(item, "tmdb").map(|id| json!(id)));
    put(
        &mut meta,
        "runtime",
        int_field(item, "duration").map(|ms| json!(format!("{} min", ms / 60_000))),
    );
    let genres = tags(item, "Genre");
    put(
        &mut meta,
        "genres",
        (!genres.is_empty()).then(|| json!(genres)),
    );
    let cast = tags(item, "Role");
    put(&mut meta, "cast", (!cast.is_empty()).then(|| json!(cast)));
    let directors = tags(item, "Director");
    put(
        &mut meta,
        "director",
        (!directors.is_empty()).then(|| json!(directors)),
    );
    put(
        &mut meta,
        "watched",
        int_field(item, "viewCount").map(|count| json!(count > 0)),
    );
    put(
        &mut meta,
        "timeOffset",
        int_field(item, "viewOffset")
            .filter(|offset| *offset > 0)
            .map(Value::from),
    );
    put(
        &mut meta,
        "duration",
        int_field(item, "duration").map(Value::from),
    );
    put(&mut meta, "mediaServer", Some(server.media_source(id)));
    Value::Object(meta)
}

fn video(server: &Server, item: &Value) -> Value {
    let id = str_field(item, "ratingKey");
    let mut video = Map::new();
    put(&mut video, "id", Some(json!(server.item_id(id))));
    put(&mut video, "title", non_empty(str_field(item, "title")));
    put(
        &mut video,
        "season",
        int_field(item, "parentIndex").map(Value::from),
    );
    put(
        &mut video,
        "episode",
        int_field(item, "index").map(Value::from),
    );
    put(
        &mut video,
        "overview",
        non_empty(str_field(item, "summary")),
    );
    put(
        &mut video,
        "released",
        non_empty(str_field(item, "originallyAvailableAt")),
    );
    let thumb = str_field(item, "thumb");
    if !thumb.is_empty() {
        put(&mut video, "thumbnail", Some(asset(server, thumb)));
    }
    put(
        &mut video,
        "watched",
        int_field(item, "viewCount").map(|count| json!(count > 0)),
    );
    put(&mut video, "mediaServer", Some(server.media_source(id)));
    Value::Object(video)
}

fn playable(item: &Value) -> bool {
    matches!(str_field(item, "type"), "movie" | "show" | "episode")
}

fn metas(server: &Server, body: &Value) -> Vec<Value> {
    let root = container(body);
    let hubs = list(root, "Hub")
        .iter()
        .flat_map(|hub| list(hub, "Metadata"));
    list(root, "Metadata")
        .iter()
        .chain(hubs)
        .filter(|item| playable(item))
        .map(|item| row_meta(server, item))
        .collect()
}

fn row_meta(server: &Server, item: &Value) -> Value {
    let mut meta = meta(server, item);
    let series = str_field(item, "grandparentRatingKey");
    if str_field(item, "type") != "episode" || series.is_empty() {
        return meta;
    }
    meta["lastVideoId"] = meta["id"].clone();
    meta["id"] = json!(server.item_id(series));
    meta["mediaServer"] = server.media_source(series);
    meta
}

fn library_catalogs(body: &Value) -> Vec<Value> {
    list(container(body), "Directory")
        .iter()
        .filter_map(|section| {
            let kind = match str_field(section, "type") {
                "movie" => "movie",
                "show" => "series",
                _ => return None,
            };
            Some(json!({"id": str_field(section, "key"), "type": kind, "name": str_field(section, "title")}))
        })
        .collect()
}

fn lookup(server: &Server, params: &Value, body: &Value) -> Vec<Value> {
    let wanted = external_id(str_field(params, "externalId"));
    list(container(body), "Metadata")
        .iter()
        .filter(|item| playable(item))
        .filter(|item| match wanted {
            Some((scheme, id)) if !list(item, "Guid").is_empty() => guid(item, scheme) == Some(id),
            _ => true,
        })
        .map(|item| meta(server, item))
        .collect()
}

fn streams_for(server: &Server, body: &Value) -> Vec<Value> {
    let mut streams = Vec::new();
    for item in list(container(body), "Metadata") {
        let id = str_field(item, "ratingKey");
        for (media_index, media) in list(item, "Media").iter().enumerate() {
            let resolution = str_field(media, "videoResolution");
            let resolution =
                if resolution.chars().all(|c| c.is_ascii_digit()) && !resolution.is_empty() {
                    format!("{resolution}p")
                } else {
                    resolution.to_uppercase()
                };
            let audio = format!(
                "{} {}",
                str_field(media, "audioCodec"),
                int_field(media, "audioChannels")
                    .map_or(String::new(), |channels| format!("{channels}ch"))
            )
            .trim()
            .to_owned();
            for (part_index, part) in list(media, "Part").iter().enumerate() {
                let key = str_field(part, "key");
                if key.is_empty() {
                    continue;
                }
                let size = int_field(part, "size");
                let subtitles: Vec<Value> = list(part, "Stream")
                    .iter()
                    .filter(|stream| {
                        int_field(stream, "streamType") == Some(3) && !str_field(stream, "key").is_empty()
                    })
                    .map(|stream| {
                        json!({
                            "id": str_field(stream, "id"),
                            "lang": non_empty(str_field(stream, "languageCode")).unwrap_or(json!("und")),
                            "url": asset(server, str_field(stream, "key")),
                        })
                    })
                    .collect();
                let filename = str_field(part, "file")
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or("")
                    .to_owned();
                let title = stream_title(&[
                    resolution.clone(),
                    str_field(media, "videoCodec").to_uppercase(),
                    audio.clone(),
                    size.map(size_text).unwrap_or_default(),
                ]);
                streams.push(json!({
                    "name": "plex",
                    "title": title,
                    "url": asset(server, key),
                    "subtitles": subtitles,
                    "behaviorHints": {
                        "videoSize": size,
                        "filename": filename,
                        "bingeGroup": format!("{}-{media_index}", server.key),
                    },
                    "mediaServer": server.media_source(id),
                }));
                let transcode = format!(
                    "{}/video/:/transcode/universal/start.m3u8?{}",
                    server.base,
                    query(&[
                        ("path", format!("/library/metadata/{id}")),
                        ("mediaIndex", media_index.to_string()),
                        ("partIndex", part_index.to_string()),
                        ("protocol", "hls".to_owned()),
                        ("directPlay", "0".to_owned()),
                        ("directStream", "1".to_owned()),
                        ("fastSeek", "1".to_owned()),
                        ("offset", "0".to_owned()),
                        ("session", server.device_id.clone()),
                        ("X-Plex-Client-Identifier", server.device_id.clone()),
                        ("X-Plex-Platform", PRODUCT.to_owned()),
                        ("X-Plex-Token", server.token.clone()),
                    ])
                );
                streams.push(json!({
                    "name": "plex",
                    "title": stream_title(&["Transcode".to_owned(), resolution.clone()]),
                    "url": transcode,
                    "subtitles": subtitles,
                    "mediaServer": server.media_source(id),
                }));
            }
        }
    }
    streams
}

fn servers(body: &Value) -> Vec<Value> {
    body.as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|resource| {
            str_field(resource, "provides")
                .split(',')
                .any(|kind| kind == "server")
        })
        .map(|resource| {
            let connections: Vec<Value> = list(resource, "connections")
                .iter()
                .map(|connection| {
                    json!({
                        "uri": str_field(connection, "uri"),
                        "local": connection.get("local").and_then(Value::as_bool).unwrap_or(false),
                        "relay": connection.get("relay").and_then(Value::as_bool).unwrap_or(false),
                    })
                })
                .collect();
            json!({
                "name": str_field(resource, "name"),
                "id": str_field(resource, "clientIdentifier"),
                "owned": resource.get("owned").and_then(Value::as_bool).unwrap_or(false),
                "accessToken": str_field(resource, "accessToken"),
                "connections": connections,
            })
        })
        .collect()
}

pub(super) fn parse(
    server: &Server,
    operation: &str,
    params: &Value,
    ok: bool,
    body: &Value,
) -> Value {
    if !ok {
        return match operation {
            "pinCheck" => json!({"state": "error"}),
            _ => json!({"error": true}),
        };
    }
    match operation {
        "pinStart" => {
            let code = str_field(body, "code");
            let auth_url = format!(
                "https://app.plex.tv/auth#?{}",
                query(&[
                    ("clientID", server.device_id.clone()),
                    ("code", code.to_owned()),
                    ("context[device][product]", PRODUCT.to_owned()),
                ])
            );
            json!({"state": "pending", "pin": {
                "id": body.get("id").cloned().unwrap_or(Value::Null),
                "code": code,
                "authUrl": auth_url,
            }})
        }
        "pinCheck" => match non_empty(str_field(body, "authToken")) {
            Some(token) => json!({"state": "success", "auth": {"accessToken": token}}),
            None => json!({"state": "pending"}),
        },
        "servers" => json!({"servers": servers(body)}),
        "info" => {
            let root = container(body);
            json!({"server": {
                "name": str_field(root, "friendlyName"),
                "id": str_field(root, "machineIdentifier"),
                "version": str_field(root, "version"),
            }})
        }
        "libraries" => json!({"catalogs": library_catalogs(body)}),
        "catalog" | "latest" | "resume" | "nextUp" | "search" => {
            json!({"metas": metas(server, body)})
        }
        "lookup" => json!({"metas": lookup(server, params, body)}),
        "meta" => match list(container(body), "Metadata").first() {
            Some(item) => json!({"meta": meta(server, item)}),
            None => json!({"error": true}),
        },
        "episodes" => json!({"videos": list(container(body), "Metadata")
            .iter()
            .map(|item| video(server, item))
            .collect::<Vec<_>>()}),
        "streams" => json!({"streams": streams_for(server, body)}),
        _ => json!({"ok": true}),
    }
}
