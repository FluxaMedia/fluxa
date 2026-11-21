use super::plugins::PluginClient;
use fluxa_plugin_runtime::{
    PluginHttpClient, PluginHttpRequest, PluginHttpResponse, execute_provider,
};
use serde_json::Value;
use std::sync::Arc;

struct DeclaredHosts {
    hosts: Vec<String>,
    inner: PluginClient,
}

impl DeclaredHosts {
    fn allows(&self, url: &str) -> bool {
        let Ok(parsed) = url::Url::parse(url) else {
            return false;
        };
        let Some(host) = parsed.host_str().map(str::to_ascii_lowercase) else {
            return false;
        };
        self.hosts
            .iter()
            .any(|declared| host == *declared || host.ends_with(&format!(".{declared}")))
    }
}

impl PluginHttpClient for DeclaredHosts {
    fn trusted_authorities(&self) -> Vec<String> {
        self.inner.trusted_authorities()
    }

    fn fetch(&self, request: PluginHttpRequest) -> PluginHttpResponse {
        if !self.allows(&request.url) {
            return PluginHttpResponse {
                error: Some("host is not listed in the extension's networkHosts".to_owned()),
                ..Default::default()
            };
        }
        self.inner.fetch(request)
    }
}

const WRAPPER: &str = r#"
class Provider {
  async call(method, args) {
    const payload = globalThis.SCRAPER_SETTINGS || {};
    const store = Object.assign({}, payload.storage || {});
    const limit = payload.limit || 65536;
    const stored = {};
    const removed = [];
    const requests = [];
    globalThis.storage = {
      get(key, fallback) { return key in store ? store[key] : fallback; },
      keys() { return Object.keys(store); },
      set(key, value) {
        const previous = store[key];
        const had = key in store;
        store[key] = value;
        if (JSON.stringify(store).length > limit) {
          if (had) store[key] = previous; else delete store[key];
          throw new Error("storage limit exceeded");
        }
        stored[key] = value;
        const at = removed.indexOf(key);
        if (at >= 0) removed.splice(at, 1);
      },
      remove(key) {
        delete store[key];
        delete stored[key];
        if (!removed.includes(key)) removed.push(key);
      },
    };
    globalThis.settings = payload.settings || {};
    const snapshot = payload.state;
    globalThis.state = (path) => {
      if (snapshot === undefined || snapshot === null) return undefined;
      let node = snapshot;
      for (const part of String(path || "").split(".").filter(Boolean)) {
        if (node === null || typeof node !== "object") return undefined;
        node = node[part];
      }
      return node;
    };
    globalThis.app = { call(name, args) { requests.push({ name: String(name), args: args === undefined ? {} : args }); } };
    let target = __skin;
    let owner = null;
    for (const part of String(method).split(".")) {
      owner = target;
      target = target && target[part];
    }
    if (typeof target !== "function") throw new Error("script has no " + method);
    const value = await target.apply(owner, args);
    return { value: value === undefined ? null : value, stored, removed, requests };
  }
}
"#;

#[derive(Clone, Debug, Default)]
pub struct SkinHost {
    pub settings: Value,
    pub storage: serde_json::Map<String, Value>,
    pub state: Value,
    pub storage_limit: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptRequest {
    pub name: String,
    pub args: Value,
}

#[derive(Clone, Debug, Default)]
pub struct SkinOutcome {
    pub value: Value,
    pub stored: serde_json::Map<String, Value>,
    pub removed: Vec<String>,
    pub requests: Vec<ScriptRequest>,
}

fn wrap(source: &str) -> String {
    format!(
        "const __skin = new ((function() {{\n{source}\n; return typeof Provider !== 'undefined' ? Provider : globalThis.Provider; }})())();\n{WRAPPER}"
    )
}

pub fn run_skin_script(
    source: &str,
    id: &str,
    method: &str,
    hosts: &[String],
    trusted: Vec<String>,
    args: Vec<Value>,
    host: &SkinHost,
) -> Result<SkinOutcome, String> {
    let client = DeclaredHosts {
        hosts: hosts.iter().map(|host| host.to_ascii_lowercase()).collect(),
        inner: PluginClient::trusting(trusted),
    };
    let payload = serde_json::json!({
        "settings": host.settings,
        "storage": host.storage,
        "state": host.state,
        "limit": host.storage_limit,
    });
    let raw = execute_provider(
        Arc::new(client),
        wrap(source),
        id.to_owned(),
        payload.to_string(),
        "call".to_owned(),
        serde_json::json!([method, args]).to_string(),
    )?;
    let envelope: Value = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    let requests = envelope["requests"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|request| {
            Some(ScriptRequest {
                name: request.get("name")?.as_str()?.to_owned(),
                args: request.get("args").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();
    Ok(SkinOutcome {
        value: envelope["value"].clone(),
        stored: envelope["stored"].as_object().cloned().unwrap_or_default(),
        removed: envelope["removed"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|key| key.as_str().map(str::to_owned))
            .collect(),
        requests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{Read, Write};

    fn serve(body: &'static str) -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten().take(4) {
                let mut stream = stream;
                let mut buffer = [0u8; 2048];
                let _ = stream.read(&mut buffer);
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        port
    }

    fn script(port: u16) -> String {
        format!(
            r#"class Provider {{
                async data() {{
                    const reply = await fetch("http://127.0.0.1:{port}/days");
                    if (!reply.ok) return {{ blocked: true }};
                    const days = (await reply.json()).days;
                    return {{ cells: days.map(day => ({{ day, label: "D" + day }})) }};
                }}
            }}"#
        )
    }

    #[test]
    fn declared_host_feeds_cells_to_the_skin() {
        let port = serve(r#"{"days":[1,2,3]}"#);
        let begun = std::time::Instant::now();
        let value = run_skin_script(
            &script(port),
            "calendar",
            "data",
            &["127.0.0.1".to_owned()],
            vec![format!("127.0.0.1:{port}")],
            Vec::new(),
            &SkinHost::default(),
        )
        .unwrap()
        .value;
        eprintln!("skin script round trip: {:?}", begun.elapsed());
        assert_eq!(value["cells"][2]["label"], "D3");
    }

    #[test]
    fn undeclared_host_is_refused() {
        let port = serve(r#"{"days":[1]}"#);
        let value = run_skin_script(
            &script(port),
            "calendar",
            "data",
            &["example.com".to_owned()],
            vec![format!("127.0.0.1:{port}")],
            Vec::new(),
            &SkinHost::default(),
        )
        .unwrap()
        .value;
        assert_eq!(value["blocked"], true);
    }

    fn call(source: &str, method: &str, args: Vec<Value>, host: &SkinHost) -> SkinOutcome {
        run_skin_script(source, "probe", method, &[], Vec::new(), args, host).unwrap()
    }

    #[test]
    fn state_is_read_by_path() {
        let host = SkinHost {
            state: json!({"player": {"position": 12.5, "paused": true}}),
            ..Default::default()
        };
        let source = r#"class Provider { async data() { return { at: state("player.position"), paused: state("player.paused"), missing: state("player.nope") === undefined, all: state().player.paused }; } }"#;
        let value = call(source, "data", Vec::new(), &host).value;
        assert_eq!(value["at"], 12.5);
        assert_eq!(value["paused"], true);
        assert_eq!(value["missing"], true);
        assert_eq!(value["all"], true);
    }

    #[test]
    fn state_is_absent_without_a_snapshot() {
        let source = r#"class Provider { async data() { return { none: state("player") === undefined }; } }"#;
        assert_eq!(
            call(source, "data", Vec::new(), &SkinHost::default()).value["none"],
            true
        );
    }

    #[test]
    fn storage_reads_and_reports_changes() {
        let mut storage = serde_json::Map::new();
        storage.insert("count".into(), json!(2));
        storage.insert("old".into(), json!("x"));
        let host = SkinHost {
            storage,
            storage_limit: 1024,
            ..Default::default()
        };
        let source = r#"class Provider { async bump() { storage.set("count", storage.get("count", 0) + 1); storage.remove("old"); return storage.keys(); } }"#;
        let outcome = call(source, "bump", Vec::new(), &host);
        assert_eq!(outcome.stored["count"], 3);
        assert_eq!(outcome.removed, vec!["old".to_owned()]);
        assert_eq!(outcome.value, json!(["count"]));
    }

    #[test]
    fn storage_refuses_writes_past_the_limit() {
        let host = SkinHost {
            storage_limit: 32,
            ..Default::default()
        };
        let source = r#"class Provider { async fill() { storage.set("a", "short"); try { storage.set("b", "x".repeat(100)); } catch (e) { return e.message; } return "no error"; } }"#;
        let outcome = call(source, "fill", Vec::new(), &host);
        assert_eq!(outcome.value, "storage limit exceeded");
        assert!(outcome.stored.contains_key("a"));
        assert!(!outcome.stored.contains_key("b"));
    }

    #[test]
    fn declared_settings_and_requests_round_trip() {
        let host = SkinHost {
            settings: json!({"volume": 40}),
            ..Default::default()
        };
        let source = r#"class Provider { async go(args) { app.call("player.volume", { value: settings.volume + args.extra }); app.call("player.pause"); return null; } }"#;
        let outcome = call(source, "go", vec![json!({"extra": 5})], &host);
        assert_eq!(
            outcome.requests,
            vec![
                ScriptRequest {
                    name: "player.volume".into(),
                    args: json!({"value": 45})
                },
                ScriptRequest {
                    name: "player.pause".into(),
                    args: json!({})
                },
            ]
        );
    }

    #[test]
    fn dotted_methods_reach_nested_objects() {
        let source = r#"class Provider { constructor() { this.tracker = { id: "t", name() { return this.id + "!"; } }; } }"#;
        let outcome = call(source, "tracker.name", Vec::new(), &SkinHost::default());
        assert_eq!(outcome.value, "t!");
    }
}
