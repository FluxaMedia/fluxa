use fluxa_aggmeta::search::encode_component;
use fluxa_aggmeta::service::{FetchError, Fetcher, HttpRequest};
use serde_json::Value;

pub const PROVIDERS: [&str; 5] = [
    "torbox",
    "realdebrid",
    "premiumize",
    "alldebrid",
    "debridlink",
];

#[derive(Debug, PartialEq)]
pub enum DebridError {
    NotCached,
    Provider(String),
}

pub struct Target {
    pub magnet: String,
    pub file_idx: Option<i64>,
    pub filename: Option<String>,
}

struct File {
    id: String,
    name: String,
    size: u64,
}

const VIDEO: [&str; 7] = ["mkv", "mp4", "avi", "mov", "m4v", "webm", "ts"];

fn pick(files: &[File], target: &Target) -> Option<usize> {
    if let Some(wanted) = target.filename.as_deref().map(str::to_lowercase) {
        let found = files.iter().position(|file| {
            let name = file.name.to_lowercase();
            name == wanted || name.ends_with(&format!("/{wanted}"))
        });
        if found.is_some() {
            return found;
        }
    }
    if let Some(index) = target
        .file_idx
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < files.len())
    {
        return Some(index);
    }
    let is_video = |file: &File| {
        file.name
            .rsplit('.')
            .next()
            .is_some_and(|ext| VIDEO.contains(&ext.to_lowercase().as_str()))
    };
    let largest = |videos_only: bool| {
        files
            .iter()
            .enumerate()
            .filter(|(_, file)| !videos_only || is_video(file))
            .max_by_key(|(_, file)| file.size)
            .map(|(index, _)| index)
    };
    largest(true).or_else(|| largest(false))
}

fn get(url: String, headers: Vec<(String, String)>) -> HttpRequest {
    HttpRequest {
        url,
        post_body: None,
        headers,
    }
}

fn bearer(key: &str) -> Vec<(String, String)> {
    vec![("Authorization".into(), format!("Bearer {key}"))]
}

fn form(url: String, body: String, key: &str) -> HttpRequest {
    let mut headers = bearer(key);
    headers.push((
        "Content-Type".into(),
        "application/x-www-form-urlencoded".into(),
    ));
    HttpRequest {
        url,
        post_body: Some(body),
        headers,
    }
}

async fn call<F: Fetcher>(fetcher: &F, request: HttpRequest) -> Result<Value, DebridError> {
    let response = fetcher
        .fetch(request)
        .await
        .map_err(|FetchError::Network(error)| DebridError::Provider(error))?;
    if !(200..300).contains(&response.status) {
        return Err(DebridError::Provider(format!("status {}", response.status)));
    }
    serde_json::from_str(&response.body).map_err(|error| DebridError::Provider(error.to_string()))
}

pub async fn resolve<F: Fetcher>(
    fetcher: &F,
    provider: &str,
    key: &str,
    target: &Target,
) -> Result<String, DebridError> {
    match provider {
        "torbox" => torbox(fetcher, key, target).await,
        "realdebrid" => realdebrid(fetcher, key, target).await,
        "premiumize" => premiumize(fetcher, key, target).await,
        "alldebrid" => alldebrid(fetcher, key, target).await,
        "debridlink" => debridlink(fetcher, key, target).await,
        other => Err(DebridError::Provider(format!("unknown provider {other}"))),
    }
}

const TORBOX: &str = "https://api.torbox.app/v1/api";

fn torbox_files(entry: &Value) -> Vec<File> {
    entry
        .get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|file| {
            Some(File {
                id: file.get("id")?.as_u64()?.to_string(),
                name: file.get("name")?.as_str()?.to_owned(),
                size: file.get("size").and_then(Value::as_u64).unwrap_or(0),
            })
        })
        .collect()
}

fn multipart(magnet: &str) -> (String, String) {
    let boundary = "----fluxa-debrid";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"magnet\"\r\n\r\n{magnet}\r\n--{boundary}--\r\n"
    );
    (format!("multipart/form-data; boundary={boundary}"), body)
}

async fn torbox<F: Fetcher>(
    fetcher: &F,
    key: &str,
    target: &Target,
) -> Result<String, DebridError> {
    let (content_type, body) = multipart(&target.magnet);
    let mut headers = bearer(key);
    headers.push(("Content-Type".into(), content_type));
    let created = call(
        fetcher,
        HttpRequest {
            url: format!("{TORBOX}/torrents/createtorrent"),
            post_body: Some(body),
            headers,
        },
    )
    .await?;
    let id = created
        .pointer("/data/torrent_id")
        .and_then(Value::as_u64)
        .ok_or_else(|| DebridError::Provider("no torrent id".into()))?;
    let listed = call(
        fetcher,
        get(
            format!("{TORBOX}/torrents/mylist?id={id}&bypass_cache=true"),
            bearer(key),
        ),
    )
    .await?;
    let entry = listed.get("data").unwrap_or(&Value::Null);
    let ready = ["download_finished", "download_present"]
        .iter()
        .any(|field| entry.get(*field).and_then(Value::as_bool) == Some(true));
    if !ready {
        return Err(DebridError::NotCached);
    }
    let files = torbox_files(entry);
    let file = &files[pick(&files, target).ok_or(DebridError::NotCached)?];
    let link = call(
        fetcher,
        get(
            format!(
                "{TORBOX}/torrents/requestdl?token={}&torrent_id={id}&file_id={}&zip_link=false",
                encode_component(key),
                file.id
            ),
            Vec::new(),
        ),
    )
    .await?;
    link.get("data")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| DebridError::Provider("no download link".into()))
}

const REAL_DEBRID: &str = "https://api.real-debrid.com/rest/1.0";

fn rd_files(info: &Value) -> Vec<File> {
    info.get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|file| {
            Some(File {
                id: file.get("id")?.as_u64()?.to_string(),
                name: file.get("path")?.as_str()?.to_owned(),
                size: file.get("bytes").and_then(Value::as_u64).unwrap_or(0),
            })
        })
        .collect()
}

async fn realdebrid<F: Fetcher>(
    fetcher: &F,
    key: &str,
    target: &Target,
) -> Result<String, DebridError> {
    let added = call(
        fetcher,
        form(
            format!("{REAL_DEBRID}/torrents/addMagnet"),
            format!("magnet={}", encode_component(&target.magnet)),
            key,
        ),
    )
    .await?;
    let id = added
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| DebridError::Provider("no torrent id".into()))?
        .to_owned();
    let info_url = format!("{REAL_DEBRID}/torrents/info/{id}");
    let info = call(fetcher, get(info_url.clone(), bearer(key))).await?;
    let files = rd_files(&info);
    let file = &files[pick(&files, target).ok_or(DebridError::NotCached)?];
    fetcher
        .fetch(form(
            format!("{REAL_DEBRID}/torrents/selectFiles/{id}"),
            format!("files={}", file.id),
            key,
        ))
        .await
        .map_err(|FetchError::Network(error)| DebridError::Provider(error))?;
    let info = call(fetcher, get(info_url, bearer(key))).await?;
    if info.get("status").and_then(Value::as_str) != Some("downloaded") {
        return Err(DebridError::NotCached);
    }
    let source = info
        .pointer("/links/0")
        .and_then(Value::as_str)
        .ok_or(DebridError::NotCached)?;
    let unrestricted = call(
        fetcher,
        form(
            format!("{REAL_DEBRID}/unrestrict/link"),
            format!("link={}", encode_component(source)),
            key,
        ),
    )
    .await?;
    unrestricted
        .get("download")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| DebridError::Provider("no download link".into()))
}

async fn premiumize<F: Fetcher>(
    fetcher: &F,
    key: &str,
    target: &Target,
) -> Result<String, DebridError> {
    let direct = call(
        fetcher,
        form(
            "https://www.premiumize.me/api/transfer/directdl".to_owned(),
            format!("src={}", encode_component(&target.magnet)),
            key,
        ),
    )
    .await?;
    if direct.get("status").and_then(Value::as_str) != Some("success") {
        return Err(DebridError::NotCached);
    }
    let content = direct
        .get("content")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let files: Vec<File> = content
        .iter()
        .enumerate()
        .map(|(index, file)| File {
            id: index.to_string(),
            name: file
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            size: file.get("size").and_then(Value::as_u64).unwrap_or(0),
        })
        .collect();
    let index = pick(&files, target).ok_or(DebridError::NotCached)?;
    content[index]
        .get("link")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| DebridError::Provider("no download link".into()))
}

const ALL_DEBRID: &str = "https://api.alldebrid.com/v4";
const DEBRID_LINK: &str = "https://debrid-link.com/api/v2";

async fn alldebrid<F: Fetcher>(
    fetcher: &F,
    key: &str,
    target: &Target,
) -> Result<String, DebridError> {
    let added = call(
        fetcher,
        form(
            format!("{ALL_DEBRID}/magnet/upload"),
            format!(
                "agent=fluxa&magnets%5B%5D={}",
                encode_component(&target.magnet)
            ),
            key,
        ),
    )
    .await?;
    let entry = added
        .pointer("/data/magnets/0")
        .ok_or_else(|| DebridError::Provider("no magnet entry".into()))?;
    if entry.get("ready").and_then(Value::as_bool) != Some(true) {
        return Err(DebridError::NotCached);
    }
    let id = entry
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| DebridError::Provider("no magnet id".into()))?;
    let status = call(
        fetcher,
        get(
            format!("https://api.alldebrid.com/v4.1/magnet/status?agent=fluxa&id={id}"),
            bearer(key),
        ),
    )
    .await?;
    let links = status
        .pointer("/data/magnets/links")
        .or_else(|| status.pointer("/data/magnets/0/links"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let files: Vec<File> = links
        .iter()
        .enumerate()
        .map(|(index, link)| File {
            id: index.to_string(),
            name: link
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            size: link.get("size").and_then(Value::as_u64).unwrap_or(0),
        })
        .collect();
    let index = pick(&files, target).ok_or(DebridError::NotCached)?;
    let source = links[index]
        .get("link")
        .and_then(Value::as_str)
        .ok_or(DebridError::NotCached)?;
    let unlocked = call(
        fetcher,
        get(
            format!(
                "{ALL_DEBRID}/link/unlock?agent=fluxa&link={}",
                encode_component(source)
            ),
            bearer(key),
        ),
    )
    .await?;
    unlocked
        .pointer("/data/link")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| DebridError::Provider("no download link".into()))
}

async fn debridlink<F: Fetcher>(
    fetcher: &F,
    key: &str,
    target: &Target,
) -> Result<String, DebridError> {
    let added = call(
        fetcher,
        form(
            format!("{DEBRID_LINK}/seedbox/add"),
            format!("url={}&async=true", encode_component(&target.magnet)),
            key,
        ),
    )
    .await?;
    let value = added
        .get("value")
        .ok_or_else(|| DebridError::Provider("no seedbox entry".into()))?;
    if value.get("downloadPercent").and_then(Value::as_f64) != Some(100.0) {
        return Err(DebridError::NotCached);
    }
    let listed = value
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let files: Vec<File> = listed
        .iter()
        .enumerate()
        .map(|(index, file)| File {
            id: index.to_string(),
            name: file
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            size: file.get("size").and_then(Value::as_u64).unwrap_or(0),
        })
        .collect();
    let index = pick(&files, target).ok_or(DebridError::NotCached)?;
    listed[index]
        .get("downloadUrl")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| DebridError::Provider("no download link".into()))
}

pub async fn cached<F: Fetcher>(
    fetcher: &F,
    provider: &str,
    key: &str,
    hashes: &[String],
) -> std::collections::HashMap<String, bool> {
    let mut found = std::collections::HashMap::new();
    if hashes.is_empty() {
        return found;
    }
    match provider {
        "torbox" => {
            let request = get(
                format!(
                    "{TORBOX}/torrents/checkcached?hash={}&format=object&list_files=false",
                    hashes.join(",")
                ),
                bearer(key),
            );
            let data = call(fetcher, request).await.ok();
            let listed = data.as_ref().and_then(|body| body.get("data"));
            for hash in hashes {
                let hit = listed
                    .and_then(|data| data.get(hash.as_str()))
                    .is_some_and(|entry| !entry.is_null() && entry != &Value::Bool(false));
                found.insert(hash.clone(), hit);
            }
        }
        "premiumize" => {
            let items = hashes
                .iter()
                .map(|hash| format!("items%5B%5D={hash}"))
                .collect::<Vec<_>>()
                .join("&");
            let request = get(
                format!("https://www.premiumize.me/api/cache/check?{items}"),
                bearer(key),
            );
            if let Ok(body) = call(fetcher, request).await {
                let flags = body.get("response").and_then(Value::as_array);
                for (hash, flag) in hashes.iter().zip(flags.into_iter().flatten()) {
                    found.insert(hash.clone(), flag.as_bool() == Some(true));
                }
            }
        }
        "debridlink" => {
            let request = get(
                format!("{DEBRID_LINK}/seedbox/cached?url={}", hashes.join(",")),
                bearer(key),
            );
            if let Ok(body) = call(fetcher, request).await {
                let listed = body.get("value");
                for hash in hashes {
                    let hit = listed
                        .and_then(|value| value.get(hash.as_str()))
                        .is_some_and(|entry| !entry.is_null());
                    found.insert(hash.clone(), hit);
                }
            }
        }
        _ => {}
    }
    found
}

pub fn hash_of(stream: &Value) -> Option<String> {
    let direct = stream.get("infoHash").and_then(Value::as_str);
    let url = stream.get("url").and_then(Value::as_str).unwrap_or("");
    direct
        .map(str::to_lowercase)
        .or_else(|| hash_from(url))
        .filter(|hash| hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
}

pub fn mark_cached(
    streams: &mut [Value],
    provider: &str,
    status: &std::collections::HashMap<String, bool>,
) {
    for stream in streams {
        let Some(hit) = hash_of(stream).and_then(|hash| status.get(&hash).copied()) else {
            continue;
        };
        if !stream["behaviorHints"].is_object() {
            stream["behaviorHints"] = serde_json::json!({});
        }
        stream["behaviorHints"]["service"] = serde_json::json!(provider);
        stream["behaviorHints"]["cached"] = serde_json::json!(hit);
    }
}

pub fn target_from(url: &str, stream: &Value) -> Option<Target> {
    let hash = stream
        .get("infoHash")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| hash_from(url))?;
    let sources = stream
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|source| source.strip_prefix("tracker:"))
        .map(|tracker| format!("&tr={}", encode_component(tracker)))
        .collect::<String>();
    Some(Target {
        magnet: format!("magnet:?xt=urn:btih:{}{sources}", hash.to_lowercase()),
        file_idx: stream.get("fileIdx").and_then(Value::as_i64),
        filename: stream
            .pointer("/behaviorHints/filename")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn hash_from(url: &str) -> Option<String> {
    let lower = url.trim().to_lowercase();
    if let Some(rest) = lower.strip_prefix("stremio://torrent/") {
        return rest.split('/').next().map(str::to_owned);
    }
    if let Some(rest) = lower.strip_prefix("infohash:") {
        return Some(rest.to_owned());
    }
    let start = lower.find("btih:")? + 5;
    lower[start..].split('&').next().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::RefCell;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    use fluxa_aggmeta::service::HttpResponse;

    struct Fake {
        routes: Vec<(&'static str, String)>,
        hits: RefCell<Vec<String>>,
    }

    impl Fetcher for Fake {
        async fn fetch(&self, request: HttpRequest) -> Result<HttpResponse, FetchError> {
            self.hits.borrow_mut().push(request.url.clone());
            let body = self
                .routes
                .iter()
                .find(|(needle, _)| request.url.contains(needle))
                .map(|(_, body)| body.clone())
                .unwrap_or_else(|| "{}".into());
            Ok(HttpResponse { status: 200, body })
        }
    }

    fn block_on<T>(fut: impl Future<Output = T>) -> T {
        let mut fut = Box::pin(fut);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(value) = fut.as_mut().poll(&mut cx) {
                return value;
            }
        }
    }

    fn target() -> Target {
        Target {
            magnet: "magnet:?xt=urn:btih:abc".into(),
            file_idx: None,
            filename: None,
        }
    }

    #[test]
    fn alldebrid_unlocks_the_picked_link_when_ready() {
        let fake = Fake {
            routes: vec![
                ("magnet/upload", r#"{"data":{"magnets":[{"id":7,"ready":true}]}}"#.into()),
                ("magnet/status", r#"{"data":{"magnets":{"links":[{"filename":"a.nfo","size":1,"link":"l1"},{"filename":"movie.mkv","size":900,"link":"l2"}]}}}"#.into()),
                ("link/unlock", r#"{"data":{"link":"https://cdn.example/movie.mkv"}}"#.into()),
            ],
            hits: RefCell::new(Vec::new()),
        };
        let target = Target {
            magnet: "magnet:?xt=urn:btih:aa".into(),
            file_idx: None,
            filename: None,
        };
        assert_eq!(
            block_on(resolve(&fake, "alldebrid", "k", &target)).unwrap(),
            "https://cdn.example/movie.mkv"
        );
    }

    #[test]
    fn debridlink_reports_partial_downloads_as_uncached() {
        let fake = Fake {
            routes: vec![(
                "seedbox/add",
                r#"{"value":{"downloadPercent":40,"files":[]}}"#.into(),
            )],
            hits: RefCell::new(Vec::new()),
        };
        let target = Target {
            magnet: "magnet:?xt=urn:btih:aa".into(),
            file_idx: None,
            filename: None,
        };
        assert_eq!(
            block_on(resolve(&fake, "debridlink", "k", &target)),
            Err(DebridError::NotCached)
        );
    }

    #[test]
    fn cached_hashes_are_marked_on_streams() {
        let hash = "a".repeat(40);
        let fake = Fake {
            routes: vec![(
                "checkcached",
                format!(r#"{{"data":{{"{hash}":{{"name":"x"}}}}}}"#),
            )],
            hits: RefCell::new(Vec::new()),
        };
        let status = block_on(cached(&fake, "torbox", "k", std::slice::from_ref(&hash)));
        let mut streams = vec![
            json!({"infoHash": hash.to_uppercase()}),
            json!({"url": "https://cdn.example/a.mkv"}),
        ];
        mark_cached(&mut streams, "torbox", &status);
        assert_eq!(streams[0]["behaviorHints"]["cached"], true);
        assert!(streams[1].get("behaviorHints").is_none());
    }

    #[test]
    fn premiumize_flags_follow_hash_order() {
        let (a, b) = ("a".repeat(40), "b".repeat(40));
        let fake = Fake {
            routes: vec![(
                "cache/check",
                r#"{"status":"success","response":[false,true]}"#.into(),
            )],
            hits: RefCell::new(Vec::new()),
        };
        let status = block_on(cached(&fake, "premiumize", "k", &[a.clone(), b.clone()]));
        assert_eq!((status[&a], status[&b]), (false, true));
    }

    #[test]
    fn largest_video_file_is_picked_without_hints() {
        let files = vec![
            File {
                id: "1".into(),
                name: "sample.mkv".into(),
                size: 10,
            },
            File {
                id: "2".into(),
                name: "movie.mkv".into(),
                size: 900,
            },
            File {
                id: "3".into(),
                name: "extras.iso".into(),
                size: 5000,
            },
        ];
        assert_eq!(pick(&files, &target()), Some(1));
    }

    #[test]
    fn filename_hint_wins_over_size() {
        let files = vec![
            File {
                id: "1".into(),
                name: "ep1.mkv".into(),
                size: 10,
            },
            File {
                id: "2".into(),
                name: "ep2.mkv".into(),
                size: 900,
            },
        ];
        let wanted = Target {
            filename: Some("EP1.mkv".into()),
            ..target()
        };
        assert_eq!(pick(&files, &wanted), Some(0));
    }

    #[test]
    fn torbox_returns_the_requested_link_for_a_cached_torrent() {
        let fetcher = Fake {
            routes: vec![
                ("createtorrent", json!({"data": {"torrent_id": 7}}).to_string()),
                (
                    "mylist",
                    json!({"data": {"download_finished": true, "files": [{"id": 3, "name": "movie.mkv", "size": 9}]}})
                        .to_string(),
                ),
                ("requestdl", json!({"data": "https://cdn.example/movie.mkv"}).to_string()),
            ],
            hits: RefCell::new(Vec::new()),
        };
        let url = block_on(resolve(&fetcher, "torbox", "k", &target())).unwrap();
        assert_eq!(url, "https://cdn.example/movie.mkv");
        assert!(
            fetcher
                .hits
                .borrow()
                .iter()
                .any(|hit| hit.contains("file_id=3"))
        );
    }

    #[test]
    fn torbox_reports_an_uncached_torrent() {
        let fetcher = Fake {
            routes: vec![
                (
                    "createtorrent",
                    json!({"data": {"torrent_id": 7}}).to_string(),
                ),
                (
                    "mylist",
                    json!({"data": {"download_finished": false}}).to_string(),
                ),
            ],
            hits: RefCell::new(Vec::new()),
        };
        assert_eq!(
            block_on(resolve(&fetcher, "torbox", "k", &target())),
            Err(DebridError::NotCached)
        );
    }

    #[test]
    fn realdebrid_unrestricts_the_first_link_once_downloaded() {
        let fetcher = Fake {
            routes: vec![
                ("addMagnet", json!({"id": "X"}).to_string()),
                (
                    "torrents/info",
                    json!({"status": "downloaded", "files": [{"id": 1, "path": "/movie.mkv", "bytes": 9}], "links": ["https://rd.example/l"]})
                        .to_string(),
                ),
                ("unrestrict", json!({"download": "https://cdn.example/movie.mkv"}).to_string()),
            ],
            hits: RefCell::new(Vec::new()),
        };
        let url = block_on(resolve(&fetcher, "realdebrid", "k", &target())).unwrap();
        assert_eq!(url, "https://cdn.example/movie.mkv");
    }

    #[test]
    fn premiumize_needs_a_successful_direct_download() {
        let fetcher = Fake {
            routes: vec![(
                "directdl",
                json!({"status": "success", "content": [{"path": "movie.mkv", "size": 9, "link": "https://cdn.example/m.mkv"}]})
                    .to_string(),
            )],
            hits: RefCell::new(Vec::new()),
        };
        assert_eq!(
            block_on(resolve(&fetcher, "premiumize", "k", &target())).unwrap(),
            "https://cdn.example/m.mkv"
        );
    }

    #[test]
    fn target_reads_hashes_from_links_and_streams() {
        let from_link = target_from("stremio://torrent/ABCDEF", &json!({})).unwrap();
        assert_eq!(from_link.magnet, "magnet:?xt=urn:btih:abcdef");
        let stream =
            json!({"infoHash": "AB12", "fileIdx": 2, "sources": ["tracker:udp://t.example:1/a"]});
        let from_stream = target_from("", &stream).unwrap();
        assert_eq!(from_stream.file_idx, Some(2));
        assert!(from_stream.magnet.contains("&tr=udp%3A%2F%2Ft.example"));
    }
}
