use super::*;

const FIELDS: &str = "ProviderIds,Overview,Genres,People,PremiereDate,ProductionYear,CommunityRating,OfficialRating,RunTimeTicks";

fn headers(server: &Server) -> Vec<(&'static str, String)> {
    let client = format!(
        "Client=\"{PRODUCT}\", Device=\"{PRODUCT}\", DeviceId=\"{}\", Version=\"{}\"",
        server.device_id,
        env!("CARGO_PKG_VERSION")
    );
    let mut headers = vec![("Content-Type", "application/json".to_owned())];
    if server.kind == "emby" {
        headers.push(("X-Emby-Authorization", format!("MediaBrowser {client}")));
        headers.push(("X-Emby-Token", server.token.clone()));
    } else if server.token.is_empty() {
        headers.push(("Authorization", format!("MediaBrowser {client}")));
    } else {
        headers.push((
            "Authorization",
            format!("MediaBrowser Token=\"{}\", {client}", server.token),
        ));
    }
    headers
}

fn get(server: &Server, path: &str, pairs: &[(&str, String)]) -> Value {
    let url = if pairs.is_empty() {
        format!("{}{path}", server.base)
    } else {
        format!("{}{path}?{}", server.base, query(pairs))
    };
    plan("GET", url, headers(server), Value::Null)
}

fn post(server: &Server, path: &str, body: Value) -> Value {
    plan(
        "POST",
        format!("{}{path}", server.base),
        headers(server),
        body,
    )
}

fn playback_body(params: &Value) -> Value {
    json!({
        "ItemId": str_field(params, "itemId"),
        "MediaSourceId": str_field(params, "mediaSourceId"),
        "PositionTicks": param_i64(params, "positionMs", 0) * TICKS_PER_MS,
        "IsPaused": str_field(params, "action") == "pause",
        "CanSeek": true,
        "PlayMethod": "DirectStream",
    })
}

pub(super) fn request(server: &Server, operation: &str, params: &Value) -> Option<Value> {
    let user = server.user_id.as_str();
    let item = str_field(params, "itemId");
    let limit = param_i64(params, "limit", 50).to_string();
    let plan = match operation {
        "info" => get(server, "/System/Info/Public", &[]),
        "login" => post(
            server,
            "/Users/AuthenticateByName",
            json!({"Username": str_field(params, "username"), "Pw": str_field(params, "password")}),
        ),
        "quickConnectStart" => post(server, "/QuickConnect/Initiate", Value::Null),
        "quickConnectCheck" => get(
            server,
            "/QuickConnect/Connect",
            &[("secret", str_field(params, "secret").to_owned())],
        ),
        "quickConnectLogin" => post(
            server,
            "/Users/AuthenticateWithQuickConnect",
            json!({"Secret": str_field(params, "secret")}),
        ),
        "libraries" => get(server, &format!("/Users/{user}/Views"), &[]),
        "catalog" => {
            let mut pairs = vec![
                ("ParentId", str_field(params, "libraryId").to_owned()),
                ("Recursive", "true".to_owned()),
                ("SortBy", "SortName".to_owned()),
                ("SortOrder", "Ascending".to_owned()),
                ("StartIndex", param_i64(params, "skip", 0).to_string()),
                ("Limit", limit),
                ("Fields", FIELDS.to_owned()),
                (
                    "IncludeItemTypes",
                    item_kinds(params).unwrap_or("Movie,Series").to_owned(),
                ),
            ];
            if let Some(genre) = Some(str_field(params, "genre")).filter(|genre| !genre.is_empty())
            {
                pairs.push(("Genres", genre.to_owned()));
            }
            get(server, &format!("/Users/{user}/Items"), &pairs)
        }
        "latest" => get(
            server,
            &format!("/Users/{user}/Items/Latest"),
            &[
                ("ParentId", str_field(params, "libraryId").to_owned()),
                ("Limit", limit),
                ("Fields", FIELDS.to_owned()),
                (
                    "IncludeItemTypes",
                    item_kinds(params).unwrap_or("Movie,Series").to_owned(),
                ),
            ],
        ),
        "resume" => get(
            server,
            &format!("/Users/{user}/Items/Resume"),
            &[
                ("Limit", limit),
                ("MediaTypes", "Video".to_owned()),
                ("Fields", FIELDS.to_owned()),
            ],
        ),
        "nextUp" => get(
            server,
            "/Shows/NextUp",
            &[
                ("UserId", user.to_owned()),
                ("Limit", limit),
                ("Fields", FIELDS.to_owned()),
            ],
        ),
        "search" => get(
            server,
            &format!("/Users/{user}/Items"),
            &[
                ("SearchTerm", str_field(params, "query").to_owned()),
                ("Recursive", "true".to_owned()),
                ("Limit", limit),
                ("Fields", FIELDS.to_owned()),
                (
                    "IncludeItemTypes",
                    item_kinds(params).unwrap_or("Movie,Series").to_owned(),
                ),
            ],
        ),
        "lookup" => {
            let (provider, id) = external_id(str_field(params, "externalId"))?;
            get(
                server,
                &format!("/Users/{user}/Items"),
                &[
                    ("AnyProviderIdEquals", format!("{provider}.{id}")),
                    ("Recursive", "true".to_owned()),
                    ("Fields", FIELDS.to_owned()),
                    (
                        "IncludeItemTypes",
                        item_kinds(params).unwrap_or("Movie,Series").to_owned(),
                    ),
                ],
            )
        }
        "meta" | "streams" => get(server, &format!("/Users/{user}/Items/{item}"), &[]),
        "episodes" => get(
            server,
            &format!("/Shows/{item}/Episodes"),
            &[
                ("UserId", user.to_owned()),
                ("Fields", "Overview,PremiereDate,RunTimeTicks".to_owned()),
            ],
        ),
        "progress" => {
            let path = match str_field(params, "action") {
                "start" => "/Sessions/Playing",
                "stop" => "/Sessions/Playing/Stopped",
                _ => "/Sessions/Playing/Progress",
            };
            post(server, path, playback_body(params))
        }
        "markWatched" => {
            let path = format!("/Users/{user}/PlayedItems/{item}");
            let method = if params.get("watched").and_then(Value::as_bool) == Some(false) {
                "DELETE"
            } else {
                "POST"
            };
            plan(
                method,
                format!("{}{path}", server.base),
                headers(server),
                Value::Null,
            )
        }
        _ => return None,
    };
    Some(plan)
}

fn image(server: &Server, id: &str, kind: &str, tag: &str, width: u32) -> Value {
    json!(format!(
        "{}/Items/{id}/Images/{kind}?{}",
        server.base,
        query(&[
            ("tag", tag.to_owned()),
            ("maxWidth", width.to_string()),
            ("api_key", server.token.clone()),
        ])
    ))
}

fn image_tag<'a>(item: &'a Value, kind: &str) -> &'a str {
    item.get("ImageTags")
        .map_or("", |tags| str_field(tags, kind))
}

fn provider_id<'a>(item: &'a Value, name: &str) -> &'a str {
    item.get("ProviderIds")
        .map_or("", |ids| str_field(ids, name))
}

fn names(item: &Value, key: &str) -> Vec<Value> {
    list(item, key).iter().map(|name| json!(name)).collect()
}

fn people(item: &Value, kind: &str) -> Vec<Value> {
    list(item, "People")
        .iter()
        .filter(|person| str_field(person, "Type") == kind)
        .filter_map(|person| person.get("Name").cloned())
        .collect()
}

fn ticks_ms(item: &Value, key: &str) -> Option<i64> {
    int_field(item, key).map(|ticks| ticks / TICKS_PER_MS)
}

fn meta(server: &Server, item: &Value) -> Value {
    let id = str_field(item, "Id");
    let episode = str_field(item, "Type") == "Episode";
    let mut meta = Map::new();
    put(&mut meta, "id", Some(json!(server.item_id(id))));
    let kind = if str_field(item, "Type") == "Movie" {
        "movie"
    } else {
        "series"
    };
    put(&mut meta, "type", Some(json!(kind)));
    let name = if episode {
        str_field(item, "SeriesName")
    } else {
        str_field(item, "Name")
    };
    put(&mut meta, "name", Some(json!(name)));
    if episode {
        put(
            &mut meta,
            "episodeTitle",
            non_empty(str_field(item, "Name")),
        );
        put(
            &mut meta,
            "season",
            int_field(item, "ParentIndexNumber").map(Value::from),
        );
        put(
            &mut meta,
            "episode",
            int_field(item, "IndexNumber").map(Value::from),
        );
        put(
            &mut meta,
            "seriesId",
            non_empty(str_field(item, "SeriesId"))
                .map(|_| json!(server.item_id(str_field(item, "SeriesId")))),
        );
    }
    let primary = image_tag(item, "Primary");
    if !primary.is_empty() {
        put(
            &mut meta,
            "poster",
            Some(image(server, id, "Primary", primary, 500)),
        );
    }
    let backdrop = list(item, "BackdropImageTags")
        .first()
        .and_then(Value::as_str)
        .unwrap_or("");
    if !backdrop.is_empty() {
        put(
            &mut meta,
            "background",
            Some(image(server, id, "Backdrop", backdrop, 1280)),
        );
    }
    let logo = image_tag(item, "Logo");
    if !logo.is_empty() {
        put(
            &mut meta,
            "logo",
            Some(image(server, id, "Logo", logo, 500)),
        );
    }
    put(
        &mut meta,
        "description",
        non_empty(str_field(item, "Overview")),
    );
    put(
        &mut meta,
        "releaseInfo",
        int_field(item, "ProductionYear").map(|year| json!(year.to_string())),
    );
    put(
        &mut meta,
        "released",
        non_empty(str_field(item, "PremiereDate")),
    );
    put(
        &mut meta,
        "imdbRating",
        float_field(item, "CommunityRating").map(|rating| json!(format!("{rating:.1}"))),
    );
    put(&mut meta, "imdb_id", non_empty(provider_id(item, "Imdb")));
    put(&mut meta, "tmdbId", non_empty(provider_id(item, "Tmdb")));
    put(
        &mut meta,
        "runtime",
        ticks_ms(item, "RunTimeTicks").map(|ms| json!(format!("{} min", ms / 60_000))),
    );
    let genres = names(item, "Genres");
    put(
        &mut meta,
        "genres",
        (!genres.is_empty()).then(|| json!(genres)),
    );
    let cast = people(item, "Actor");
    put(&mut meta, "cast", (!cast.is_empty()).then(|| json!(cast)));
    let directors = people(item, "Director");
    put(
        &mut meta,
        "director",
        (!directors.is_empty()).then(|| json!(directors)),
    );
    if let Some(data) = item.get("UserData") {
        put(&mut meta, "watched", data.get("Played").cloned());
        put(
            &mut meta,
            "timeOffset",
            ticks_ms(data, "PlaybackPositionTicks")
                .filter(|offset| *offset > 0)
                .map(Value::from),
        );
        put(
            &mut meta,
            "duration",
            ticks_ms(item, "RunTimeTicks").map(Value::from),
        );
    }
    put(&mut meta, "mediaServer", Some(server.media_source(id)));
    Value::Object(meta)
}

fn video(server: &Server, item: &Value) -> Value {
    let id = str_field(item, "Id");
    let mut video = Map::new();
    put(&mut video, "id", Some(json!(server.item_id(id))));
    put(&mut video, "title", non_empty(str_field(item, "Name")));
    put(
        &mut video,
        "season",
        int_field(item, "ParentIndexNumber").map(Value::from),
    );
    put(
        &mut video,
        "episode",
        int_field(item, "IndexNumber").map(Value::from),
    );
    put(
        &mut video,
        "overview",
        non_empty(str_field(item, "Overview")),
    );
    put(
        &mut video,
        "released",
        non_empty(str_field(item, "PremiereDate")),
    );
    let primary = image_tag(item, "Primary");
    if !primary.is_empty() {
        put(
            &mut video,
            "thumbnail",
            Some(image(server, id, "Primary", primary, 400)),
        );
    }
    put(
        &mut video,
        "watched",
        item.get("UserData")
            .and_then(|data| data.get("Played"))
            .cloned(),
    );
    put(&mut video, "mediaServer", Some(server.media_source(id)));
    Value::Object(video)
}

fn metas(server: &Server, body: &Value) -> Vec<Value> {
    let items = if body.is_array() {
        body.as_array().map_or(&[][..], Vec::as_slice)
    } else {
        list(body, "Items")
    };
    items
        .iter()
        .filter(|item| matches!(str_field(item, "Type"), "Movie" | "Series" | "Episode"))
        .map(|item| row_meta(server, item))
        .collect()
}

fn row_meta(server: &Server, item: &Value) -> Value {
    let mut meta = meta(server, item);
    let series = str_field(item, "SeriesId");
    if str_field(item, "Type") != "Episode" || series.is_empty() {
        return meta;
    }
    meta["lastVideoId"] = meta["id"].clone();
    meta["id"] = json!(server.item_id(series));
    meta["mediaServer"] = server.media_source(series);
    let tag = str_field(item, "SeriesPrimaryImageTag");
    if !tag.is_empty() {
        meta["poster"] = image(server, series, "Primary", tag, 500);
    }
    meta
}

fn library_catalogs(body: &Value) -> Vec<Value> {
    list(body, "Items")
        .iter()
        .flat_map(|view| {
            let kinds: &[&str] = match str_field(view, "CollectionType") {
                "movies" => &["movie"],
                "tvshows" => &["series"],
                "" | "mixed" => &["movie", "series"],
                _ => &[],
            };
            kinds.iter().map(move |kind| {
                json!({"id": str_field(view, "Id"), "type": kind, "name": str_field(view, "Name")})
            })
        })
        .collect()
}

fn streams_for(server: &Server, item: &Value) -> Vec<Value> {
    let id = str_field(item, "Id");
    let mut streams = Vec::new();
    for source in list(item, "MediaSources") {
        let source_id = str_field(source, "Id");
        let media = list(source, "MediaStreams");
        let of_type = |kind: &str| {
            media
                .iter()
                .find(|stream| str_field(stream, "Type") == kind)
        };
        let height = of_type("Video").and_then(|stream| int_field(stream, "Height"));
        let resolution = height
            .map(|height| format!("{height}p"))
            .unwrap_or_default();
        let video_codec = of_type("Video").map_or("", |stream| str_field(stream, "Codec"));
        let audio = of_type("Audio").map_or(String::new(), |stream| {
            format!(
                "{} {}",
                str_field(stream, "Codec"),
                str_field(stream, "ChannelLayout")
            )
            .trim()
            .to_owned()
        });
        let size = int_field(source, "Size").map(size_text).unwrap_or_default();
        let subtitles: Vec<Value> = media
            .iter()
            .filter(|stream| {
                str_field(stream, "Type") == "Subtitle"
                    && stream.get("IsTextSubtitleStream").and_then(Value::as_bool) == Some(true)
            })
            .filter_map(|stream| {
                let index = int_field(stream, "Index")?;
                let lang = non_empty(str_field(stream, "Language")).unwrap_or(json!("und"));
                Some(json!({
                    "id": format!("{source_id}:{index}"),
                    "lang": lang,
                    "url": format!(
                        "{}/Videos/{id}/{source_id}/Subtitles/{index}/0/Stream.vtt?{}",
                        server.base,
                        query(&[("api_key", server.token.clone())])
                    ),
                }))
            })
            .collect();
        let filename = str_field(source, "Path")
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("")
            .to_owned();
        let direct = format!(
            "{}/Videos/{id}/stream?{}",
            server.base,
            query(&[
                ("static", "true".to_owned()),
                ("MediaSourceId", source_id.to_owned()),
                ("DeviceId", server.device_id.clone()),
                ("api_key", server.token.clone()),
            ])
        );
        let transcode = format!(
            "{}/Videos/{id}/master.m3u8?{}",
            server.base,
            query(&[
                ("MediaSourceId", source_id.to_owned()),
                ("DeviceId", server.device_id.clone()),
                ("api_key", server.token.clone()),
                ("VideoCodec", "h264".to_owned()),
                ("AudioCodec", "aac".to_owned()),
                ("SegmentContainer", "ts".to_owned()),
                ("TranscodingMaxAudioChannels", "2".to_owned()),
                ("MaxStreamingBitrate", "20000000".to_owned()),
            ])
        );
        let label = server.kind.clone();
        let title = stream_title(&[
            resolution.clone(),
            video_codec.to_uppercase(),
            audio.clone(),
            size.clone(),
        ]);
        streams.push(json!({
            "name": label,
            "title": title,
            "description": str_field(source, "Name"),
            "url": direct,
            "subtitles": subtitles,
            "behaviorHints": {
                "videoSize": int_field(source, "Size"),
                "filename": filename,
                "bingeGroup": format!("{}-{source_id}", server.key),
            },
            "mediaServer": server.media_source(id),
            "mediaSourceId": source_id,
        }));
        streams.push(json!({
            "name": label,
            "title": stream_title(&["Transcode".to_owned(), resolution]),
            "url": transcode,
            "subtitles": subtitles,
            "mediaServer": server.media_source(id),
            "mediaSourceId": source_id,
        }));
    }
    streams
}

fn auth(body: &Value) -> Value {
    let token = str_field(body, "AccessToken");
    if token.is_empty() {
        return json!({"state": "error"});
    }
    let user = body.get("User").unwrap_or(&Value::Null);
    json!({
        "state": "success",
        "auth": {
            "accessToken": token,
            "userId": str_field(user, "Id"),
            "userName": str_field(user, "Name"),
            "serverId": str_field(body, "ServerId"),
        }
    })
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
            "login" | "quickConnectLogin" => json!({"state": "error"}),
            "quickConnectCheck" => json!({"state": "error"}),
            _ => json!({"error": true}),
        };
    }
    match operation {
        "info" => json!({"server": {
            "name": str_field(body, "ServerName"),
            "id": str_field(body, "Id"),
            "version": str_field(body, "Version"),
        }}),
        "login" | "quickConnectLogin" => auth(body),
        "quickConnectStart" => json!({"state": "pending", "quickConnect": {
            "code": str_field(body, "Code"),
            "secret": str_field(body, "Secret"),
        }}),
        "quickConnectCheck" => {
            if body.get("Authenticated").and_then(Value::as_bool) == Some(true) {
                json!({"state": "authenticated"})
            } else {
                json!({"state": "pending"})
            }
        }
        "libraries" => json!({"catalogs": library_catalogs(body)}),
        "lookup" => {
            let wanted = external_id(str_field(params, "externalId"));
            let metas = metas(server, body)
                .into_iter()
                .filter(|meta| match wanted {
                    Some(("imdb", id)) => str_field(meta, "imdb_id") == id,
                    Some(("tmdb", id)) => str_field(meta, "tmdbId") == id,
                    _ => true,
                })
                .collect::<Vec<_>>();
            json!({"metas": metas})
        }
        "catalog" | "latest" | "resume" | "nextUp" | "search" => {
            json!({"metas": metas(server, body)})
        }
        "meta" => json!({"meta": meta(server, body)}),
        "episodes" => json!({"videos": list(body, "Items")
            .iter()
            .map(|item| video(server, item))
            .collect::<Vec<_>>()}),
        "streams" => json!({"streams": streams_for(server, body)}),
        _ => json!({"ok": true}),
    }
}
