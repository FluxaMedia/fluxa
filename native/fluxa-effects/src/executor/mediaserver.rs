use super::providers::{random_hex, send};
use super::*;
use std::sync::OnceLock;

pub(super) const HOST: &str = "mediaserver.fluxa.internal";
const SERVERS_KEY: &str = "media_servers";
const CLIENT_KEY: &str = "media_client_id";
const PAGE_SIZE: i64 = 50;

static STORAGE: OnceLock<Storage> = OnceLock::new();

pub(super) fn init(storage: &Storage) {
    let _ = STORAGE.set(storage.clone());
}

pub(super) fn is_kind(provider: &str) -> bool {
    matches!(provider, "jellyfin" | "emby" | "plex")
}

pub fn media_servers(storage: &Storage) -> Vec<Value> {
    storage
        .read_json(SERVERS_KEY)
        .ok()
        .flatten()
        .and_then(|servers| servers.as_array().cloned())
        .unwrap_or_default()
}

pub fn remove_media_server(storage: &Storage, key: &str) -> Result<(), String> {
    let servers: Vec<Value> = media_servers(storage)
        .into_iter()
        .filter(|server| str_of(server, "key") != key)
        .collect();
    storage.write_json(SERVERS_KEY, &Value::Array(servers))
}

fn upsert_server(storage: &Storage, server: Value) -> Result<(), String> {
    let mut servers = media_servers(storage);
    let key = str_of(&server, "key").to_owned();
    servers.retain(|existing| str_of(existing, "key") != key);
    servers.push(server);
    storage.write_json(SERVERS_KEY, &Value::Array(servers))
}

fn client_id(storage: &Storage) -> Result<String, String> {
    if let Some(id) = storage
        .read_json(CLIENT_KEY)?
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|id| !id.is_empty())
    {
        return Ok(id);
    }
    let id = random_hex().ok_or("secure randomness is unavailable")?[..32].to_owned();
    storage.write_json(CLIENT_KEY, &json!(id))?;
    Ok(id)
}

fn unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

fn str_of<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn transport(key: &str) -> String {
    format!("https://{HOST}/{key}")
}

fn slug(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .take(8)
        .collect::<String>()
        .to_lowercase()
}

fn normalize_base(url: &str) -> String {
    let url = url.trim().trim_end_matches('/');
    if url.contains("://") {
        url.to_owned()
    } else {
        format!("http://{url}")
    }
}

fn client(seconds: u64) -> Result<Client, String> {
    Client::builder()
        .user_agent("Fluxa/1.0")
        .native_timeout(Duration::from_secs(seconds))
        .build()
        .map_err(|error| error.to_string())
}

async fn call(
    client: &Client,
    server: &Value,
    operation: &str,
    params: Value,
) -> Result<Value, String> {
    let args = json!({
        "kind": server["kind"],
        "server": server,
        "operation": operation,
        "params": params,
    });
    let plan = core_value("mediaServerRequest", args.clone())
        .ok_or_else(|| format!("unsupported media server operation {operation}"))?;
    let (status, body) = send(client, &plan).await?;
    let mut parse = args;
    parse["status"] = json!(status);
    parse["body"] = body;
    core_value("mediaServerParse", parse)
        .ok_or_else(|| format!("could not read the {operation} response"))
}

fn descriptor(server: &Value) -> Value {
    let key = str_of(server, "key");
    let name = str_of(server, "name");
    let prefix = format!("ms:{key}:");
    let mut catalogs = Vec::new();
    let mut add = |id: &str, kind: &str, label: String, extra: Value| {
        catalogs.push(json!({"id": id, "type": kind, "name": label, "extra": extra}));
    };
    let paged = json!([{"name": "skip"}]);
    for kind in ["movie", "series"] {
        add(
            "__resume",
            kind,
            format!("{name} · Continue Watching"),
            json!([]),
        );
        add(
            "__latest",
            kind,
            format!("{name} · Recently Added"),
            json!([]),
        );
        add(
            "__search",
            kind,
            format!("{name} · Search"),
            json!([{"name": "search", "isRequired": true}]),
        );
    }
    for library in server
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        add(
            str_of(library, "id"),
            str_of(library, "type"),
            format!("{name} · {}", str_of(library, "name")),
            paged.clone(),
        );
    }
    let url = transport(key);
    json!({
        "transportUrl": url,
        "manifest": {
            "id": format!("mediaserver.{key}"),
            "version": "1.0.0",
            "name": name,
            "description": format!("{} library", str_of(server, "kind")),
            "transportUrl": url,
            "types": ["movie", "series"],
            "resources": [
                "catalog",
                {"name": "meta", "types": ["movie", "series"], "idPrefixes": [prefix]},
                {"name": "stream", "types": ["movie", "series"], "idPrefixes": [prefix, "tt"]},
            ],
            "idPrefixes": [prefix, "tt"],
            "catalogs": catalogs,
        }
    })
}

pub(super) fn addons(storage: &Storage) -> Vec<Value> {
    media_servers(storage).iter().map(descriptor).collect()
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let decoded = (bytes[index] == b'%')
            .then(|| text.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match decoded {
            Some(byte) => {
                out.push(byte);
                index += 3;
            }
            None => {
                out.push(bytes[index]);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(super) async fn respond(url: &reqwest::Url) -> (i32, Option<String>) {
    match handle(url).await {
        Ok(body) => (200, Some(body.to_string())),
        Err(error) => {
            crate::log!("[fluxa-native] media server request failed: {error}");
            (502, Some("{}".to_owned()))
        }
    }
}

async fn handle(url: &reqwest::Url) -> Result<Value, String> {
    let storage = STORAGE.get().ok_or("storage is not ready")?;
    handle_with(storage, url).await
}

async fn handle_with(storage: &Storage, url: &reqwest::Url) -> Result<Value, String> {
    let mut segments: Vec<String> = url
        .path_segments()
        .ok_or("media server url has no path")?
        .map(percent_decode)
        .collect();
    if let Some(last) = segments.last_mut() {
        if let Some(stripped) = last.strip_suffix(".json") {
            *last = stripped.to_owned();
        }
    }
    let key = segments.first().cloned().unwrap_or_default();
    let server = media_servers(storage)
        .into_iter()
        .find(|server| str_of(server, "key") == key)
        .ok_or("media server is not configured")?;
    let client = client(20)?;
    match segments.get(1).map(String::as_str) {
        Some("manifest") => Ok(descriptor(&server)["manifest"].clone()),
        Some("catalog") => {
            let kind = segments.get(2).cloned().unwrap_or_default();
            let id = segments.get(3).cloned().unwrap_or_default();
            let extra = segments.get(4).cloned().unwrap_or_default();
            catalog(&client, &server, &kind, &id, &extra).await
        }
        Some("meta") => {
            let kind = segments.get(2).cloned().unwrap_or_default();
            let id = segments.get(3).cloned().unwrap_or_default();
            meta(&client, &server, &kind, &id).await
        }
        Some("stream") => {
            let kind = segments.get(2).cloned().unwrap_or_default();
            let id = segments.get(3).cloned().unwrap_or_default();
            streams(&client, &server, &kind, &id).await
        }
        _ => Err("unknown media server resource".to_owned()),
    }
}

fn of_kind(result: &Value, kind: &str) -> Value {
    let metas: Vec<Value> = result["metas"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|meta| str_of(meta, "type") == kind)
        .cloned()
        .collect();
    json!({"metas": metas})
}

async fn catalog(
    client: &Client,
    server: &Value,
    kind: &str,
    id: &str,
    extra: &str,
) -> Result<Value, String> {
    let extra: std::collections::HashMap<String, String> =
        reqwest::Url::parse(&format!("http://local/?{extra}"))
            .map_err(|error| error.to_string())?
            .query_pairs()
            .into_owned()
            .collect();
    let skip = extra
        .get("skip")
        .and_then(|skip| skip.parse::<i64>().ok())
        .unwrap_or(0);
    let (operation, params) = match id {
        "__search" => (
            "search",
            json!({"query": extra.get("search"), "kind": kind, "limit": PAGE_SIZE}),
        ),
        "__resume" => ("resume", json!({"limit": PAGE_SIZE})),
        "__latest" => ("latest", json!({"kind": kind, "limit": 40})),
        library => (
            "catalog",
            json!({
                "libraryId": library,
                "kind": kind,
                "skip": skip,
                "limit": PAGE_SIZE,
                "genre": extra.get("genre"),
            }),
        ),
    };
    let result = call(client, server, operation, params).await?;
    if result["error"] == true {
        return Err(format!("{operation} failed with {}", result["status"]));
    }
    Ok(of_kind(&result, kind))
}

async fn meta(client: &Client, server: &Value, kind: &str, id: &str) -> Result<Value, String> {
    let item = id
        .strip_prefix(&format!("ms:{}:", str_of(server, "key")))
        .ok_or("not a media server item")?;
    let result = call(client, server, "meta", json!({"itemId": item})).await?;
    let mut meta = result
        .get("meta")
        .cloned()
        .ok_or("media server item not found")?;
    if kind == "series" {
        let episodes = call(client, server, "episodes", json!({"itemId": item})).await?;
        meta["videos"] = episodes["videos"].clone();
    }
    Ok(json!({"meta": meta}))
}

async fn resolve_item(
    client: &Client,
    server: &Value,
    kind: &str,
    id: &str,
) -> Result<Option<String>, String> {
    let prefix = format!("ms:{}:", str_of(server, "key"));
    if let Some(item) = id.strip_prefix(&prefix) {
        return Ok(Some(item.to_owned()));
    }
    let mut parts = id.split(':');
    let external = parts.next().unwrap_or_default();
    let season = parts.next().and_then(|part| part.parse::<i64>().ok());
    let episode = parts.next().and_then(|part| part.parse::<i64>().ok());
    let found = call(
        client,
        server,
        "lookup",
        json!({"externalId": external, "kind": kind}),
    )
    .await?;
    let Some(item) = found["metas"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|meta| str_of(meta, "type") == kind)
        .and_then(|meta| meta["mediaServer"]["itemId"].as_str())
    else {
        return Ok(None);
    };
    if kind != "series" {
        return Ok(Some(item.to_owned()));
    }
    let (Some(season), Some(episode)) = (season, episode) else {
        return Ok(None);
    };
    let episodes = call(client, server, "episodes", json!({"itemId": item})).await?;
    Ok(episodes["videos"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|video| video["season"] == season && video["episode"] == episode)
        .and_then(|video| video["mediaServer"]["itemId"].as_str())
        .map(ToOwned::to_owned))
}

async fn streams(client: &Client, server: &Value, kind: &str, id: &str) -> Result<Value, String> {
    let Some(item) = resolve_item(client, server, kind, id).await? else {
        return Ok(json!({"streams": []}));
    };
    let mut result = call(client, server, "streams", json!({"itemId": item})).await?;
    let name = str_of(server, "name").to_owned();
    if let Some(streams) = result["streams"].as_array_mut() {
        for stream in streams {
            stream["name"] = json!(name);
        }
    }
    Ok(json!({"streams": result["streams"]}))
}

impl EffectExecutor {
    pub(super) async fn media_auth_start(&self, provider: &str) -> Result<Value, String> {
        if provider != "plex" {
            return Err(format!("{provider} signs in with a server address"));
        }
        let device_id = client_id(&self.storage)?;
        let server = json!({"kind": "plex", "deviceId": device_id});
        let outcome = call(&client(20)?, &server, "pinStart", json!({})).await?;
        let pin = outcome.get("pin").ok_or("plex did not return a PIN")?;
        Ok(json!({
            "provider": provider,
            "state": "pending",
            "device": {
                "userCode": pin["code"],
                "deviceCode": pin["id"].to_string(),
                "verificationUrl": pin["authUrl"],
                "interval": 2,
                "expiresAt": unix_seconds() + 900,
            }
        }))
    }

    pub(super) async fn media_auth_exchange(&self, payload: &Value) -> Result<Value, String> {
        let provider = str_of(payload, "provider");
        let code = str_of(payload, "code");
        let profile = payload.get("profile").cloned().unwrap_or(Value::Null);
        let client = client(20)?;
        let device_id = client_id(&self.storage)?;
        if provider == "plex" {
            let base = json!({"kind": "plex", "deviceId": device_id, "token": ""});
            let checked = call(&client, &base, "pinCheck", json!({"pinId": code})).await?;
            match str_of(&checked, "state") {
                "success" => {}
                "pending" => return Ok(json!({"provider": provider, "state": "pending"})),
                state => return Err(format!("plex sign-in failed: {state}")),
            }
            let token = checked["auth"]["accessToken"].as_str().unwrap_or_default();
            let account = json!({"kind": "plex", "deviceId": device_id, "token": token});
            let discovered = call(&client, &account, "servers", json!({})).await?;
            let mut added = 0;
            for resource in discovered["servers"].as_array().into_iter().flatten() {
                if self
                    .add_plex_server(resource, token, &device_id)
                    .await
                    .is_ok()
                {
                    added += 1;
                }
            }
            if added == 0 {
                return Err("no reachable Plex server was found".to_owned());
            }
            return Ok(json!({"provider": provider, "state": "success", "profile": profile}));
        }
        let credentials: Value =
            serde_json::from_str(code).map_err(|_| "invalid server credentials".to_owned())?;
        let base_url = normalize_base(str_of(&credentials, "baseUrl"));
        let mut server = json!({
            "kind": provider,
            "baseUrl": base_url,
            "token": "",
            "userId": "",
            "deviceId": device_id,
        });
        let info = call(&client, &server, "info", json!({})).await?;
        let login = call(
            &client,
            &server,
            "login",
            json!({
                "username": str_of(&credentials, "username"),
                "password": str_of(&credentials, "password"),
            }),
        )
        .await?;
        if str_of(&login, "state") != "success" {
            return Err(format!("{provider} sign-in failed"));
        }
        let server_id = str_of(&info["server"], "id");
        let server_id = if server_id.is_empty() {
            str_of(&login["auth"], "serverId")
        } else {
            server_id
        };
        server["token"] = login["auth"]["accessToken"].clone();
        server["userId"] = login["auth"]["userId"].clone();
        server["serverId"] = json!(server_id);
        server["key"] = json!(format!("{provider}-{}", slug(server_id)));
        let name = str_of(&info["server"], "name");
        server["name"] = json!(if name.is_empty() { provider } else { name });
        self.save_server(&client, server).await?;
        Ok(json!({"provider": provider, "state": "success", "profile": profile}))
    }

    async fn add_plex_server(
        &self,
        resource: &Value,
        account_token: &str,
        device_id: &str,
    ) -> Result<(), String> {
        let token = Some(str_of(resource, "accessToken"))
            .filter(|token| !token.is_empty())
            .unwrap_or(account_token);
        let mut connections: Vec<&Value> = resource["connections"]
            .as_array()
            .into_iter()
            .flatten()
            .collect();
        connections
            .sort_by_key(|connection| (connection["relay"] == true, connection["local"] != true));
        let probe = client(5)?;
        for connection in connections {
            let mut server = json!({
                "kind": "plex",
                "baseUrl": normalize_base(str_of(connection, "uri")),
                "token": token,
                "userId": "",
                "deviceId": device_id,
            });
            let Ok(info) = call(&probe, &server, "info", json!({})).await else {
                continue;
            };
            if info["status"] != 200 {
                continue;
            }
            let id = str_of(resource, "id");
            server["serverId"] = json!(id);
            server["key"] = json!(format!("plex-{}", slug(id)));
            server["name"] = json!(str_of(resource, "name"));
            return self.save_server(&client(20)?, server).await;
        }
        Err("no reachable connection".to_owned())
    }

    async fn save_server(&self, client: &Client, mut server: Value) -> Result<(), String> {
        let libraries = call(client, &server, "libraries", json!({})).await?;
        server["catalogs"] = libraries["catalogs"].clone();
        upsert_server(&self.storage, server)
    }
}

impl EffectExecutor {
    pub(super) async fn report_media_server(&self, payload: &Value) -> bool {
        let Some(rest) = str_of(payload, "itemId").strip_prefix("ms:") else {
            return false;
        };
        let Some((key, item)) = rest.split_once(':') else {
            return true;
        };
        let Some(server) = media_servers(&self.storage)
            .into_iter()
            .find(|server| str_of(server, "key") == key)
        else {
            return true;
        };
        if let Err(error) = self.push_progress(&server, item, payload).await {
            crate::log!("[fluxa-native] media server progress failed: {error}");
        }
        true
    }

    async fn push_progress(
        &self,
        server: &Value,
        item: &str,
        payload: &Value,
    ) -> Result<(), String> {
        let client = client(10)?;
        let action = str_of(payload, "actionName");
        let percent = payload["progress"].as_f64().unwrap_or(0.0);
        let meta = call(&client, server, "meta", json!({"itemId": item})).await?;
        let duration = meta["meta"]["duration"].as_i64().unwrap_or_else(|| {
            str_of(&meta["meta"], "runtime")
                .split_whitespace()
                .next()
                .and_then(|minutes| minutes.parse::<i64>().ok())
                .map_or(0, |minutes| minutes * 60_000)
        });
        let position = (duration as f64 * percent / 100.0) as i64;
        call(
            &client,
            server,
            "progress",
            json!({
                "itemId": item,
                "mediaSourceId": item,
                "action": action,
                "positionMs": position,
                "durationMs": duration,
            }),
        )
        .await?;
        if action == "stop" && percent >= 90.0 {
            call(
                &client,
                server,
                "markWatched",
                json!({"itemId": item, "watched": true}),
            )
            .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn serve(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten().take(4) {
                let mut stream = stream;
                let mut buffer = [0; 4096];
                let _ = stream.read(&mut buffer);
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        base
    }

    fn storage_with(base: &str) -> (Storage, std::path::PathBuf) {
        let directory = std::env::temp_dir().join(format!("fluxa-media-{}", random_hex().unwrap()));
        let storage = Storage::open(directory.clone()).unwrap();
        upsert_server(
            &storage,
            json!({
                "key": "jellyfin-abc",
                "kind": "jellyfin",
                "name": "Home",
                "baseUrl": base,
                "token": "secret",
                "userId": "u1",
                "deviceId": "d1",
                "catalogs": [{"id": "lib1", "type": "movie", "name": "Movies"}],
            }),
        )
        .unwrap();
        (storage, directory)
    }

    #[tokio::test]
    async fn library_catalog_is_served_as_addon_metas() {
        let base = serve(
            r#"{"Items":[{"Id":"m1","Name":"Heat","Type":"Movie","ProductionYear":1995,"ProviderIds":{"Imdb":"tt0113277"},"ImageTags":{"Primary":"x"}}],"TotalRecordCount":1}"#,
        );
        let (storage, directory) = storage_with(&base);
        let url = reqwest::Url::parse(
            "https://mediaserver.fluxa.internal/jellyfin-abc/catalog/movie/lib1.json",
        )
        .unwrap();
        let body = handle_with(&storage, &url).await.unwrap();
        assert_eq!(body["metas"][0]["name"], "Heat");
        assert_eq!(body["metas"][0]["id"], "ms:jellyfin-abc:m1");
        let _ = std::fs::remove_dir_all(directory);
    }

    #[tokio::test]
    async fn unknown_server_key_is_an_error() {
        let (storage, directory) = storage_with("http://127.0.0.1:1");
        let url =
            reqwest::Url::parse("https://mediaserver.fluxa.internal/gone/manifest.json").unwrap();
        assert!(handle_with(&storage, &url).await.is_err());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn synthetic_addon_matches_external_ids_for_streams() {
        let (storage, directory) = storage_with("http://127.0.0.1:1");
        let addon = &addons(&storage)[0];
        let prefixes = &addon["manifest"]["resources"][2]["idPrefixes"];
        assert_eq!(prefixes[1], "tt");
        assert_eq!(
            addon["transportUrl"],
            "https://mediaserver.fluxa.internal/jellyfin-abc"
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    #[tokio::test]
    #[ignore]
    async fn demo_server_end_to_end() {
        let base = std::env::var("DEMO_JELLYFIN").expect("DEMO_JELLYFIN");
        let directory = std::env::temp_dir().join(format!("fluxa-demo-{}", random_hex().unwrap()));
        let storage = Storage::open(directory.clone()).unwrap();
        let executor = EffectExecutor::new(storage.clone());
        let kind = std::env::var("DEMO_KIND").unwrap_or("jellyfin".into());
        if kind == "plex" {
            let resource = json!({"id": "demoplex", "name": "Plex Demo", "accessToken": "", "connections": [{"uri": base, "local": true}]});
            executor.add_plex_server(&resource, "", "demo-device").await.unwrap();
        } else {
            let signed_in = executor
                .media_auth_exchange(&json!({
                    "provider": kind,
                    "code": json!({"baseUrl": base, "username": "demo", "password": "demo"}).to_string(),
                    "profile": {},
                }))
                .await
                .unwrap();
            println!("sign-in: {signed_in}");
        }
        let server = media_servers(&storage).remove(0);
        println!("stored server: {}", serde_json::to_string(&server).unwrap());
        let key = str_of(&server, "key").to_owned();
        let get = |path: &str| {
            let url = reqwest::Url::parse(&format!("https://{HOST}/{key}/{path}")).unwrap();
            let storage = storage.clone();
            async move { handle_with(&storage, &url).await }
        };
        let item = std::env::var("DEMO_ITEM").unwrap_or("heat".into());
        let imdb = std::env::var("DEMO_IMDB").unwrap_or("tt0113277".into());
        let library = std::env::var("DEMO_LIBRARY").unwrap_or("a".repeat(32));
        for path in [
            "manifest.json".to_owned(),
            "catalog/movie/__latest.json".to_owned(),
            "catalog/movie/__resume.json".to_owned(),
            "catalog/movie/__search/search=alien.json".to_owned(),
            format!("catalog/movie/{library}.json"),
            format!("meta/movie/ms%3A{key}%3A{item}.json"),
            format!("stream/movie/{imdb}.json"),
            format!("stream/movie/ms%3A{key}%3A{item}.json"),
        ] {
            println!("GET {path}\n{:?}\n", get(&path).await);
        }
        for action in ["start", "pause", "stop"] {
            let handled = executor
                .report_media_server(&json!({"itemId": format!("ms:{key}:{item}"), "actionName": action, "progress": 95.0}))
                .await;
            println!("progress {action}: {handled}");
        }
        let _ = std::fs::remove_dir_all(directory);
    }
}
