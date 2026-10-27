use super::plugins::PluginTarget;
use super::*;
use fluxa_plugin_runtime::{execute_provider, transpile_typescript};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const SUPPORTED_TYPES: [&str; 2] = ["onlinestream-provider", "anime-torrent-provider"];
const MAX_TORRENTS: usize = 20;

pub(super) fn is_seanime(scraper: &Value) -> bool {
    scraper.get("kind").and_then(Value::as_str) == Some("seanime")
}

pub(super) fn manifest_to_repository(manifest_url: &str, raw: &Value) -> Result<Value, String> {
    let entries: Vec<&Value> = match raw {
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    };
    let scrapers: Vec<Value> = entries
        .iter()
        .filter_map(|entry| scraper_entry(manifest_url, entry))
        .collect();
    if scrapers.is_empty() {
        return Err("extension manifest has no supported provider".to_owned());
    }
    let first = scrapers[0].clone();
    let single = scrapers.len() == 1;
    Ok(json!({
        "name": if single { first["name"].clone() } else { json!("Seanime extensions") },
        "description": first["description"].clone(),
        "version": if single { first["version"].clone() } else { json!("1") },
        "scrapers": scrapers,
    }))
}

fn scraper_entry(manifest_url: &str, entry: &Value) -> Option<Value> {
    let text = |key: &str| {
        entry
            .get(key)
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
    };
    let kind = text("type").filter(|kind| SUPPORTED_TYPES.contains(kind))?;
    let id = text("id")?;
    let payload_uri = text("payloadURI").map(|uri| resolve_url(manifest_url, uri));
    if payload_uri.is_none() && text("payload").is_none() {
        return None;
    }
    let config_fields: Vec<Value> = entry
        .pointer("/userConfig/fields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|field| field.get("name").and_then(Value::as_str).is_some())
        .take(4)
        .cloned()
        .collect();
    Some(json!({
        "id": format!("seanime:{id}"),
        "name": text("name").unwrap_or(id),
        "description": text("description"),
        "version": text("version").unwrap_or("1.0.0"),
        "filename": "",
        "kind": "seanime",
        "seanimeType": kind,
        "language": text("language").unwrap_or("javascript"),
        "payload": text("payload"),
        "payloadUri": payload_uri,
        "logo": text("icon"),
        "contentLanguage": text("lang").map(|lang| vec![lang]),
        "supportedTypes": ["movie", "tv"],
        "enabled": true,
        "hasSettings": !config_fields.is_empty(),
        "configFields": config_fields,
    }))
}

fn resolve_url(base: &str, reference: &str) -> String {
    url::Url::parse(base)
        .and_then(|base| base.join(reference))
        .map_or_else(|_| reference.to_owned(), |url| url.to_string())
}

async fn extension_code(scraper: &Value) -> Result<String, String> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let source_key = scraper
        .get("payloadUri")
        .and_then(Value::as_str)
        .or_else(|| scraper.get("id").and_then(Value::as_str))
        .unwrap_or_default()
        .to_owned();
    if let Some(code) = cache
        .lock()
        .ok()
        .and_then(|cache| cache.get(&source_key).cloned())
    {
        return Ok(code);
    }
    let source = match scraper.get("payload").and_then(Value::as_str) {
        Some(inline) => inline.to_owned(),
        None => {
            let url = scraper
                .get("payloadUri")
                .and_then(Value::as_str)
                .ok_or_else(|| "extension has no payload".to_owned())?;
            let client = Client::builder()
                .bundled_roots()
                .user_agent("Fluxa/1.0")
                .native_timeout(Duration::from_secs(20))
                .build()
                .map_err(|error| error.to_string())?;
            let (status, body) = fetch_text(&client, url).await?;
            body.filter(|_| (200..300).contains(&status))
                .ok_or_else(|| format!("extension payload returned HTTP {status}"))?
        }
    };
    let typescript = scraper.get("language").and_then(Value::as_str) == Some("typescript");
    let code = if typescript {
        tokio::task::spawn_blocking(move || transpile_typescript(&source))
            .await
            .map_err(|error| error.to_string())??
    } else {
        source
    };
    if let Ok(mut cache) = cache.lock() {
        cache.insert(source_key, code.clone());
    }
    Ok(code)
}

struct Provider {
    code: String,
    id: String,
    settings: String,
}

impl Provider {
    async fn call(&self, method: &str, args: Value) -> Result<Value, String> {
        let (code, id, settings) = (self.code.clone(), self.id.clone(), self.settings.clone());
        let method = method.to_owned();
        let raw = tokio::task::spawn_blocking(move || {
            execute_provider(
                Arc::new(super::plugins::PluginClient::default()),
                code,
                id,
                settings,
                method,
                args.to_string(),
            )
        })
        .await
        .map_err(|error| error.to_string())??;
        serde_json::from_str(&raw).map_err(|error| error.to_string())
    }
}

fn user_settings(scraper: &Value) -> Value {
    let mut merged = serde_json::Map::new();
    for field in scraper
        .get("configFields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let (Some(name), Some(default)) = (
            field.get("name").and_then(Value::as_str),
            field.get("default"),
        ) {
            merged.insert(name.to_owned(), default.clone());
        }
    }
    if let Some(saved) = scraper.get("settings").and_then(Value::as_object) {
        for (name, value) in saved {
            merged.insert(name.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

fn media_of(target: &PluginTarget) -> Value {
    let title = target.title.clone().unwrap_or_default();
    json!({
        "id": 0,
        "idMal": null,
        "status": "FINISHED",
        "format": if target.media_type == "tv" { "TV" } else { "MOVIE" },
        "englishTitle": title,
        "romajiTitle": target.original_title.clone().unwrap_or_else(|| title.clone()),
        "episodeCount": null,
        "absoluteSeasonOffset": 0,
        "synonyms": [],
        "isAdult": false,
        "startDate": target.year.map(|year| json!({"year": year})),
    })
}

pub(super) async fn run(scraper: &Value, target: &PluginTarget) -> Result<Value, String> {
    let title = target
        .title
        .clone()
        .filter(|title| !title.trim().is_empty())
        .ok_or_else(|| "title is unknown for this item".to_owned())?;
    let provider = Provider {
        code: extension_code(scraper).await?,
        id: scraper
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("seanime")
            .to_owned(),
        settings: user_settings(scraper).to_string(),
    };
    let episode = target.episode.unwrap_or(1);
    let media = media_of(target);
    match scraper.get("seanimeType").and_then(Value::as_str) {
        Some("onlinestream-provider") => stream_sources(&provider, &media, &title, episode).await,
        Some("anime-torrent-provider") => torrents(&provider, &media, &title, episode).await,
        _ => Err("unsupported extension type".to_owned()),
    }
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn search_queries(title: &str) -> Vec<String> {
    let mut queries = vec![title.to_owned()];
    for cut in [
        title.split([':', '-', '\u{2013}']).next(),
        title.split_whitespace().next(),
    ] {
        let cut = cut.unwrap_or_default().trim();
        if !cut.is_empty() && !queries.iter().any(|query| query == cut) {
            queries.push(cut.to_owned());
        }
    }
    queries
}

fn best_match<'a>(results: &'a [Value], title: &str) -> Option<&'a Value> {
    let wanted = normalized(title);
    results
        .iter()
        .find(|item| {
            item.get("title")
                .and_then(Value::as_str)
                .is_some_and(|candidate| normalized(candidate) == wanted)
        })
        .or_else(|| results.first())
}

async fn stream_sources(
    provider: &Provider,
    media: &Value,
    title: &str,
    episode: i32,
) -> Result<Value, String> {
    let settings = provider.call("getSettings", json!([])).await?;
    let servers: Vec<String> = settings
        .get("episodeServers")
        .and_then(Value::as_array)
        .map(|servers| {
            servers
                .iter()
                .filter_map(|server| server.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .filter(|servers: &Vec<String>| !servers.is_empty())
        .unwrap_or_else(|| vec!["default".to_owned()]);
    let year = media["startDate"]["year"].as_i64().unwrap_or(0);
    let mut results = Vec::new();
    for query in search_queries(title) {
        let found = provider
            .call(
                "search",
                json!([{"media": media, "query": query, "dub": false, "year": year}]),
            )
            .await?;
        results = found.as_array().cloned().unwrap_or_default();
        if !results.is_empty() {
            break;
        }
    }
    let series = best_match(&results, title).ok_or_else(|| "no search result".to_owned())?;
    let series_id = series["id"].clone();
    let episodes = provider.call("findEpisodes", json!([series_id])).await?;
    let episodes = episodes.as_array().cloned().unwrap_or_default();
    let chosen = episodes
        .iter()
        .find(|item| item["number"].as_i64() == Some(i64::from(episode)))
        .or_else(|| episodes.first().filter(|_| episodes.len() == 1))
        .ok_or_else(|| format!("episode {episode} not found"))?;
    let mut streams = Vec::new();
    for server in servers {
        let served = match provider
            .call("findEpisodeServer", json!([chosen, server]))
            .await
        {
            Ok(served) => served,
            Err(error) => {
                crate::log!("[fluxa-native] seanime server {server} failed: {error}");
                continue;
            }
        };
        let headers = served.get("headers").cloned().unwrap_or(Value::Null);
        for source in served
            .get("videoSources")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let subtitles: Vec<Value> = source
                .get("subtitles")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|subtitle| {
                    Some(json!({
                        "url": subtitle.get("url")?,
                        "language": subtitle.get("language").cloned().unwrap_or(json!("und")),
                        "name": subtitle.get("language"),
                    }))
                })
                .collect();
            streams.push(json!({
                "title": format!("{title} E{episode}"),
                "name": server,
                "url": source.get("url"),
                "quality": source.get("quality"),
                "type": source.get("type"),
                "headers": headers,
                "subtitles": subtitles,
            }));
        }
    }
    Ok(Value::Array(streams))
}

async fn torrents(
    provider: &Provider,
    media: &Value,
    title: &str,
    episode: i32,
) -> Result<Value, String> {
    let settings = provider
        .call("getSettings", json!([]))
        .await
        .unwrap_or(Value::Null);
    let smart = settings["canSmartSearch"].as_bool() == Some(true);
    let listed = if smart {
        provider
            .call(
                "smartSearch",
                json!([{
                    "media": media,
                    "query": "",
                    "batch": false,
                    "episodeNumber": episode,
                    "resolution": "",
                    "anidbAID": 0,
                    "anidbEID": 0,
                    "bestReleases": false,
                }]),
            )
            .await?
    } else {
        provider
            .call(
                "search",
                json!([{"media": media, "query": format!("{title} {episode:02}")}]),
            )
            .await?
    };
    let mut streams = Vec::new();
    for torrent in listed.as_array().into_iter().flatten().take(MAX_TORRENTS) {
        let hash = match info_hash(torrent) {
            Some(hash) => Some(hash),
            None => provider
                .call("getTorrentInfoHash", json!([torrent]))
                .await
                .ok()
                .and_then(|value| value.as_str().map(str::to_lowercase))
                .filter(|hash| !hash.is_empty()),
        };
        let Some(hash) = hash else { continue };
        streams.push(json!({
            "title": torrent.get("name"),
            "infoHash": hash,
            "size": torrent.get("formattedSize"),
            "seeders": torrent.get("seeders"),
            "quality": torrent.get("resolution"),
        }));
    }
    Ok(Value::Array(streams))
}

fn info_hash(torrent: &Value) -> Option<String> {
    if let Some(hash) = torrent
        .get("infoHash")
        .and_then(Value::as_str)
        .filter(|hash| !hash.is_empty())
    {
        return Some(hash.to_lowercase());
    }
    let magnet = torrent.get("magnetLink").and_then(Value::as_str)?;
    let start = magnet.find("btih:")? + 5;
    let hash: String = magnet[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    (hash.len() == 40).then(|| hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_config_fields_become_defaults_overridden_by_saved_values() {
        let raw = json!({
            "id": "nexus", "type": "onlinestream-provider", "name": "Nexus",
            "payloadURI": "p.ts", "language": "typescript",
            "userConfig": {"fields": [{"name": "resolverUrl", "label": "URL", "type": "text", "default": "http://127.0.0.1:3000"}]}
        });
        let mut scraper = scraper_entry("https://example.com/m.json", &raw).unwrap();
        assert_eq!(scraper["hasSettings"], true);
        assert_eq!(user_settings(&scraper)["resolverUrl"], "http://127.0.0.1:3000");
        scraper["settings"] = json!({"resolverUrl": "http://10.0.0.2:3000"});
        assert_eq!(user_settings(&scraper)["resolverUrl"], "http://10.0.0.2:3000");
    }

    #[test]
    fn manifest_array_keeps_only_supported_provider_types() {
        let raw = json!([
            {"id": "a", "type": "onlinestream-provider", "name": "A", "payloadURI": "a.ts", "language": "typescript"},
            {"id": "b", "type": "manga-provider", "name": "B", "payloadURI": "b.ts"},
            {"id": "c", "type": "plugin", "name": "C", "payload": "x"},
            {"id": "d", "type": "anime-torrent-provider", "name": "D", "payload": "class Provider {}"}
        ]);
        let repo = manifest_to_repository("https://example.com/m/index.json", &raw).unwrap();
        let scrapers = repo["scrapers"].as_array().unwrap();
        assert_eq!(scrapers.len(), 2);
        assert_eq!(scrapers[0]["id"], "seanime:a");
        assert_eq!(scrapers[0]["payloadUri"], "https://example.com/m/a.ts");
    }

    #[test]
    fn magnet_links_yield_lowercase_info_hash() {
        let torrent = json!({"magnetLink": "magnet:?xt=urn:btih:ABCDEF0123456789ABCDEF0123456789ABCDEF01&dn=x"});
        assert_eq!(
            info_hash(&torrent).as_deref(),
            Some("abcdef0123456789abcdef0123456789abcdef01")
        );
    }
}
