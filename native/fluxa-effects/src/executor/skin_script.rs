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

pub fn run_skin_script(
    source: &str,
    id: &str,
    method: &str,
    hosts: &[String],
    trusted: Vec<String>,
    args: Vec<Value>,
    state: &Value,
) -> Result<Value, String> {
    let client = DeclaredHosts {
        hosts: hosts.iter().map(|host| host.to_ascii_lowercase()).collect(),
        inner: PluginClient::trusting(trusted),
    };
    let raw = execute_provider(
        Arc::new(client),
        source.to_owned(),
        id.to_owned(),
        state.to_string(),
        method.to_owned(),
        Value::Array(args).to_string(),
    )?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
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
            &Value::Null,
        )
        .unwrap();
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
            &Value::Null,
        )
        .unwrap();
        assert_eq!(value["blocked"], true);
    }
}
