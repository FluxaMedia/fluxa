use super::fluxa_account::{Supabase, client};
use super::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use web_time::Instant;

mod socket;
use socket::{Socket, Task, Ticker};
pub(crate) use socket::{sleep, spawn};

pub type ActionSink = Arc<dyn Fn(Value) + Send + Sync>;

const HEARTBEAT: Duration = Duration::from_secs(25);
const MAX_MESSAGE_BYTES: usize = 32 * 1024;
const CLOCK_SAMPLES: u32 = 4;
const CLOCK_GAP: Duration = Duration::from_millis(400);
const DEVICE_KEY: &str = "watch_device_id";
const MISSING_FUNCTION: &str = "PGRST202";

#[derive(Default)]
pub(super) struct WatchTogether {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    sink: Option<ActionSink>,
    commands: VecDeque<Value>,
    connection: Option<Connection>,
}

enum Outbound {
    Broadcast { event: String, payload: Value },
    Track(Value),
}

struct Connection {
    party: String,
    member: String,
    nuvio: bool,
    outbound: UnboundedSender<Outbound>,
    task: Task,
}

#[derive(Clone)]
struct Link {
    base: String,
    key: &'static str,
    token: String,
    profile_index: Option<i64>,
    nuvio: bool,
}

impl WatchTogether {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn emit(&self, action: Value) {
        let sink = self.lock().sink.clone();
        if let Some(sink) = sink {
            sink(action);
        }
    }

    fn connection_flag(&self, party: &str) -> Option<bool> {
        self.lock()
            .connection
            .as_ref()
            .filter(|connection| connection.party == party)
            .map(|connection| connection.nuvio)
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn frame(topic: &str, event: &str, payload: Value, reference: String) -> String {
    json!({"topic": topic, "event": event, "payload": payload, "ref": reference}).to_string()
}

fn error_code(body: &Value, status: reqwest::StatusCode) -> String {
    ["code", "message", "error", "msg"]
        .iter()
        .find_map(|key| body.get(key).and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("request failed with status {status}"))
}

async fn rpc(link: &Link, name: &str, body: Value) -> Result<Value, String> {
    let response = client()?
        .post(format!(
            "{}/rest/v1/rpc/{name}",
            link.base.trim_end_matches('/')
        ))
        .header("apikey", link.key)
        .bearer_auth(&link.token)
        .json(&body)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let value = response.json::<Value>().await.unwrap_or(Value::Null);
    if status.is_success() {
        Ok(value)
    } else {
        Err(error_code(&value, status))
    }
}

fn presence_entries(map: &Value) -> Vec<(String, Value)> {
    map.as_object()
        .map(|entries| {
            entries
                .iter()
                .map(|(key, value)| (key.clone(), value["metas"][0].clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn presence_keys(map: &Value) -> Vec<String> {
    map.as_object()
        .map(|entries| entries.keys().cloned().collect())
        .unwrap_or_default()
}

fn avatar_url(payload: &Value) -> Value {
    payload["avatar"]
        .as_str()
        .filter(|url| url.starts_with("https://") && url.len() <= 512)
        .map_or(Value::Null, |url| json!(url))
}

impl EffectExecutor {
    pub fn set_watch_sink(&self, sink: ActionSink) {
        self.watch.lock().sink = Some(sink);
    }

    pub fn take_watch_commands(&self) -> Vec<Value> {
        self.watch.lock().commands.drain(..).collect()
    }

    fn device_id(&self) -> Result<String, String> {
        if let Some(id) = self
            .storage
            .read_json(DEVICE_KEY)?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
        {
            return Ok(id);
        }
        let id = format!(
            "fluxa-{:x}-{:x}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
            std::process::id()
        );
        self.storage.write_json(DEVICE_KEY, &json!(id))?;
        Ok(id)
    }

    async fn watch_link(&self, force_nuvio: Option<bool>) -> Result<Link, String> {
        if force_nuvio != Some(false) {
            let (_, profile, _) = self.active_profile()?;
            if let Some(session) = self.nuvio_session(&profile).await? {
                return Ok(Link {
                    base: session.base_url.to_owned(),
                    key: session.api_key,
                    token: session.token,
                    profile_index: Some(session.profile_index),
                    nuvio: true,
                });
            }
            if force_nuvio == Some(true) {
                return Err("Nuvio session unavailable".to_owned());
            }
        }
        let config = Supabase::configured()?;
        let credentials = self.watch_credentials().await?;
        Ok(Link {
            base: config.base,
            key: config.key,
            token: credentials.token,
            profile_index: None,
            nuvio: false,
        })
    }

    async fn party_link(&self, party: &str) -> Result<Link, String> {
        self.watch_link(self.watch.connection_flag(party)).await
    }

    pub(super) async fn watch_room(&self, payload: &Value) -> Result<Value, String> {
        let party = payload["partyId"].as_str().unwrap_or_default();
        match payload["op"].as_str() {
            Some("create") => self.enter_party("watch_party_create", payload).await,
            Some("join") => self.enter_party("watch_party_join", payload).await,
            Some("leave") => self.leave_party(party, "watch_party_leave").await,
            Some("end") => self.leave_party(party, "watch_party_end").await,
            Some("kick") => {
                let member = payload["memberId"].as_str().unwrap_or_default();
                self.party_snapshot(
                    party,
                    "watch_party_remove_member",
                    json!({"p_member_id": member}),
                    None,
                )
                .await
            }
            Some("hostOnly") => {
                let host_only = payload["hostOnly"].as_bool().unwrap_or(false);
                self.party_snapshot(
                    party,
                    "watch_party_set_host_only",
                    json!({"p_host_only": host_only}),
                    None,
                )
                .await
            }
            Some("update") => {
                let mut body = json!({});
                if !payload["media"].is_null() {
                    body["p_media"] = payload["media"].clone();
                }
                if !payload["playback"].is_null() {
                    body["p_playback"] = payload["playback"].clone();
                }
                self.party_snapshot(party, "watch_party_update", body, None)
                    .await
            }
            Some("heartbeat") => {
                let announce = payload["announce"].as_str().map(ToOwned::to_owned);
                self.party_snapshot(party, "watch_party_heartbeat", json!({}), announce)
                    .await
            }
            Some("rejoin") => self.rejoin_party(party, payload).await,
            Some("chat") => {
                let text = payload["text"].as_str().unwrap_or_default();
                let link = self.party_link(party).await?;
                let body = json!({
                    "p_party_id": party,
                    "p_device_id": self.device_id()?,
                    "p_body": text,
                    "p_client_id": format!("{}-{}", now_ms(), text.len()),
                });
                rpc(&link, "watch_party_send_message", body).await?;
                Ok(Value::Null)
            }
            _ => Err("unknown watch party operation".to_owned()),
        }
    }

    pub(super) fn watch_broadcast(&self, payload: &Value) -> Result<Value, String> {
        let room = payload["room"].as_str().unwrap_or_default();
        let inner = self.watch.lock();
        let connection = inner
            .connection
            .as_ref()
            .filter(|connection| connection.party == room)
            .ok_or_else(|| "not connected to the party".to_owned())?;
        let message = if payload.get("track").is_some() {
            Outbound::Track(payload["track"].clone())
        } else {
            Outbound::Broadcast {
                event: payload["event"].as_str().unwrap_or_default().to_owned(),
                payload: payload["payload"].clone(),
            }
        };
        connection
            .outbound
            .send(message)
            .map_err(|_| "party connection closed".to_owned())?;
        Ok(Value::Null)
    }

    pub(super) fn watch_command(&self, payload: &Value) -> Result<Value, String> {
        let mut inner = self.watch.lock();
        if inner.commands.len() >= 64 {
            inner.commands.pop_front();
        }
        inner.commands.push_back(payload.clone());
        Ok(Value::Null)
    }

    fn identity(&self, payload: &Value) -> Result<Value, String> {
        let name: String = payload["displayName"]
            .as_str()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or("Fluxa")
            .chars()
            .take(40)
            .collect();
        Ok(json!({
            "p_device_id": self.device_id()?,
            "p_display_name": name,
            "p_avatar_url": avatar_url(payload),
            "p_avatar_color": Value::Null,
        }))
    }

    async fn call_with_profile(
        &self,
        link: &Link,
        function: &str,
        body: Value,
    ) -> Result<Value, String> {
        let Some(index) = link.profile_index else {
            return rpc(link, function, body).await;
        };
        let mut with_profile = body.clone();
        with_profile["p_profile_index"] = json!(index);
        match rpc(link, function, with_profile).await {
            Err(code) if code == MISSING_FUNCTION => rpc(link, function, body).await,
            other => other,
        }
    }

    async fn enter_party(&self, function: &str, payload: &Value) -> Result<Value, String> {
        let link = self.watch_link(None).await?;
        let mut body = self.identity(payload)?;
        if function == "watch_party_create" {
            body["p_media"] = payload["media"].clone();
            body["p_playback"] = Value::Null;
        } else {
            body["p_code"] = payload["code"].clone();
        }
        let sent_at = now_ms();
        let snapshot = self.call_with_profile(&link, function, body).await?;
        let received_at = now_ms();
        if let Some(error) = snapshot["error"].as_str() {
            return Err(error.to_owned());
        }
        let (Some(party), Some(member)) = (
            snapshot["party"]["id"].as_str(),
            snapshot["member_id"].as_str(),
        ) else {
            return Err("party response was invalid".to_owned());
        };
        self.open_connection(party.to_owned(), member.to_owned(), link.nuvio);
        self.sample_clock(link);
        Ok(json!({"snapshot": snapshot, "sentAt": sent_at, "receivedAt": received_at}))
    }

    async fn party_snapshot(
        &self,
        party: &str,
        function: &str,
        mut body: Value,
        announce: Option<String>,
    ) -> Result<Value, String> {
        let link = self.party_link(party).await?;
        body["p_party_id"] = json!(party);
        body["p_device_id"] = json!(self.device_id()?);
        let snapshot = rpc(&link, function, body).await?;
        self.watch.emit(json!({
            "type": "watchTogetherSnapshotReceived",
            "snapshot": snapshot,
            "announce": announce,
            "nowMs": now_ms(),
        }));
        Ok(Value::Null)
    }

    async fn leave_party(&self, party: &str, function: &str) -> Result<Value, String> {
        let link = self.party_link(party).await;
        let connection = {
            let mut inner = self.watch.lock();
            match inner.connection.as_ref() {
                Some(connection) if connection.party == party => inner.connection.take(),
                _ => None,
            }
        };
        if let Some(connection) = &connection {
            connection.task.abort();
        }
        let link = link?;
        let body = json!({"p_party_id": party, "p_device_id": self.device_id()?});
        rpc(&link, function, body).await.map(|_| Value::Null)
    }

    async fn rejoin_party(&self, party: &str, payload: &Value) -> Result<Value, String> {
        let link = self.party_link(party).await?;
        let mut body = self.identity(payload)?;
        body["p_code"] = payload["code"].clone();
        let result = self
            .call_with_profile(&link, "watch_party_join", body)
            .await;
        let (snapshot, error) = match result {
            Ok(snapshot) => match snapshot["error"].as_str() {
                Some(error) => (None, Some(error.to_owned())),
                None => (Some(snapshot), None),
            },
            Err(error) => (None, Some(error)),
        };
        if let Some(snapshot) = &snapshot
            && let (Some(new_party), Some(member)) = (
                snapshot["party"]["id"].as_str(),
                snapshot["member_id"].as_str(),
            )
        {
            let same = self
                .watch
                .lock()
                .connection
                .as_ref()
                .is_some_and(|c| c.party == new_party && c.member == member);
            if !same {
                self.open_connection(new_party.to_owned(), member.to_owned(), link.nuvio);
            }
        }
        self.watch.emit(json!({
            "type": "watchTogetherRejoinFinished",
            "snapshot": snapshot,
            "error": error,
            "nowMs": now_ms(),
        }));
        Ok(Value::Null)
    }

    fn sample_clock(&self, link: Link) {
        let executor = self.clone();
        let _ = socket::spawn(async move {
            for _ in 0..CLOCK_SAMPLES {
                socket::sleep(CLOCK_GAP).await;
                let sent_at = now_ms();
                let Ok(server) = rpc(&link, "watch_party_clock", json!({})).await else {
                    continue;
                };
                let Some(server_ms) = server
                    .as_i64()
                    .or_else(|| server.as_str().and_then(|text| text.trim().parse().ok()))
                else {
                    continue;
                };
                executor.watch.emit(json!({
                    "type": "watchTogetherClockSampled",
                    "serverMs": server_ms,
                    "sentAt": sent_at,
                    "receivedAt": now_ms(),
                }));
            }
        });
    }

    fn open_connection(&self, party: String, member: String, nuvio: bool) {
        let (outbound, inbox) = unbounded_channel();
        let executor = self.clone();
        let (task_party, task_member) = (party.clone(), member.clone());
        let task = socket::spawn(async move {
            executor
                .keep_connected(task_party, task_member, nuvio, inbox)
                .await;
        });
        let previous = self.watch.lock().connection.replace(Connection {
            party,
            member,
            nuvio,
            outbound,
            task,
        });
        if let Some(previous) = previous {
            previous.task.abort();
        }
    }

    async fn keep_connected(
        &self,
        party: String,
        member: String,
        nuvio: bool,
        mut inbox: UnboundedReceiver<Outbound>,
    ) {
        let mut delay = Duration::from_secs(1);
        loop {
            let started = Instant::now();
            match self
                .realtime_session(&party, &member, nuvio, &mut inbox)
                .await
            {
                Ok(()) => return,
                Err(error) => crate::log!("[watch-party] connection lost: {error}"),
            }
            self.watch.emit(json!({
                "type": "watchTogetherConnectionChanged",
                "connected": false,
                "nowMs": now_ms(),
            }));
            if started.elapsed() > Duration::from_secs(60) {
                delay = Duration::from_secs(1);
            }
            socket::sleep(delay).await;
            delay = (delay * 2).min(Duration::from_secs(30));
        }
    }

    async fn realtime_session(
        &self,
        party: &str,
        member: &str,
        nuvio: bool,
        inbox: &mut UnboundedReceiver<Outbound>,
    ) -> Result<(), String> {
        let mut link = self.watch_link(Some(nuvio)).await?;
        let socket_url = format!(
            "{}/realtime/v1/websocket?apikey={}&vsn=1.0.0",
            link.base.trim_end_matches('/').replacen("http", "ws", 1),
            link.key
        );
        let mut socket = Socket::connect(&socket_url).await?;
        let topic = format!("realtime:watch-party:{party}");
        let mut counter = 1u64;
        let join_ref = counter.to_string();
        let join = json!({
            "config": {
                "broadcast": {"self": false, "ack": false},
                "presence": {"key": member},
                "private": true,
            },
            "access_token": link.token,
        });
        socket
            .send(
                json!({
                    "topic": topic, "event": "phx_join", "payload": join,
                    "ref": join_ref, "join_ref": join_ref,
                })
                .to_string(),
            )
            .await?;
        let mut subscribed = false;
        let mut heartbeat = Ticker::new(HEARTBEAT).await;
        loop {
            tokio::select! {
                incoming = socket.recv() => {
                    let text = incoming?;
                    let Ok(event) = serde_json::from_str::<Value>(&text) else {
                        continue;
                    };
                    if event["topic"] != topic.as_str() {
                        continue;
                    }
                    let payload = &event["payload"];
                    match event["event"].as_str() {
                        Some("phx_reply") if event["ref"] == join_ref.as_str() => {
                            if payload["status"] != "ok" {
                                return Err(format!("join rejected: {}", payload["response"]));
                            }
                            subscribed = true;
                            self.watch.emit(json!({
                                "type": "watchTogetherConnectionChanged",
                                "connected": true,
                                "nowMs": now_ms(),
                            }));
                        }
                        Some("broadcast") => {
                            if payload.to_string().len() > MAX_MESSAGE_BYTES {
                                continue;
                            }
                            let Some(name) = payload["event"].as_str() else {
                                continue;
                            };
                            self.watch.emit(json!({
                                "type": "watchTogetherMessageReceived",
                                "event": name,
                                "payload": payload["payload"],
                                "nowMs": now_ms(),
                            }));
                        }
                        Some("presence_state") => {
                            self.watch.emit(json!({
                                "type": "watchTogetherPresenceChanged",
                                "joins": presence_entries(payload),
                                "leaves": Vec::<String>::new(),
                                "nowMs": now_ms(),
                            }));
                        }
                        Some("presence_diff") => {
                            self.watch.emit(json!({
                                "type": "watchTogetherPresenceChanged",
                                "joins": presence_entries(&payload["joins"]),
                                "leaves": presence_keys(&payload["leaves"]),
                                "nowMs": now_ms(),
                            }));
                        }
                        Some("phx_error" | "phx_close") => return Err("channel closed".to_owned()),
                        _ => {}
                    }
                }
                outgoing = inbox.recv() => {
                    let Some(outgoing) = outgoing else {
                        socket.close().await;
                        return Ok(());
                    };
                    if !subscribed {
                        continue;
                    }
                    counter += 1;
                    let (event, payload) = match outgoing {
                        Outbound::Broadcast { event, payload } => (
                            "broadcast",
                            json!({"type": "broadcast", "event": event, "payload": payload}),
                        ),
                        Outbound::Track(state) => (
                            "presence",
                            json!({"type": "presence", "event": "track", "payload": state}),
                        ),
                    };
                    socket.send(frame(&topic, event, payload, counter.to_string())).await?;
                }
                _ = heartbeat.tick() => {
                    counter += 1;
                    socket
                        .send(json!({
                            "topic": "phoenix", "event": "heartbeat",
                            "payload": {}, "ref": counter.to_string(),
                        }).to_string())
                        .await?;
                    let fresh = self.watch_link(Some(nuvio)).await?;
                    if fresh.token != link.token {
                        link = fresh;
                        counter += 1;
                        socket
                            .send(frame(
                                &topic,
                                "access_token",
                                json!({"access_token": link.token}),
                                counter.to_string(),
                            ))
                            .await?;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_state_exposes_the_first_meta_per_member() {
        let map = json!({"a": {"metas": [{"member_id": "a", "status": "ready", "phx_ref": "x"}]}});
        let entries = presence_entries(&map);
        assert_eq!(entries[0].0, "a");
        assert_eq!(entries[0].1["status"], "ready");
    }

    #[test]
    fn only_https_avatars_are_shared() {
        assert_eq!(
            avatar_url(&json!({"avatar": "http://x/a.png"})),
            Value::Null
        );
        assert_eq!(
            avatar_url(&json!({"avatar": "https://x/a.png"})),
            json!("https://x/a.png")
        );
    }

    #[test]
    fn rpc_errors_prefer_the_postgrest_code() {
        let body = json!({"code": "PGRST202", "message": "no function"});
        assert_eq!(
            error_code(&body, reqwest::StatusCode::NOT_FOUND),
            "PGRST202"
        );
    }
}
