use super::fluxa_account::{Credentials, Supabase, client};
use super::*;
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio_tungstenite::tungstenite::Message;

pub type ActionSink = Arc<dyn Fn(Value) + Send + Sync>;

const HEARTBEAT: Duration = Duration::from_secs(25);

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

struct Connection {
    room: String,
    outbound: UnboundedSender<Value>,
    task: tokio::task::JoinHandle<()>,
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
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn rest(client: &Client, config: &Supabase, token: &str, path: &str) -> reqwest::RequestBuilder {
    client
        .get(format!("{}/rest/v1/{path}", config.base))
        .header("apikey", config.key)
        .bearer_auth(token)
}

fn rpc(client: &Client, config: &Supabase, token: &str, name: &str) -> reqwest::RequestBuilder {
    client
        .post(format!("{}/rest/v1/rpc/{name}", config.base))
        .header("apikey", config.key)
        .bearer_auth(token)
}

async fn json_response(response: reqwest::Response) -> Result<Value, String> {
    let status = response.status();
    let body = response.json::<Value>().await.unwrap_or(Value::Null);
    if status.is_success() {
        return Ok(body);
    }
    let message = ["message", "error", "msg"]
        .iter()
        .find_map(|key| body.get(key).and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("request failed with status {status}"));
    Err(message)
}

async fn fetch_roster(
    client: &Client,
    config: &Supabase,
    token: &str,
    code: &str,
) -> Result<(String, Vec<Value>), String> {
    let room = json_response(
        rest(
            client,
            config,
            token,
            &format!("watch_rooms?code=eq.{code}&select=host_id"),
        )
        .send()
        .await
        .map_err(|error| error.to_string())?,
    )
    .await?;
    let host_id = room
        .get(0)
        .and_then(|row| row["host_id"].as_str())
        .ok_or_else(|| "room not found".to_owned())?
        .to_owned();
    let rows = json_response(
        rest(
            client,
            config,
            token,
            &format!(
                "watch_room_members?room_code=eq.{code}&select=user_id,display_name,avatar,color&order=joined_at.asc"
            ),
        )
        .send()
        .await
        .map_err(|error| error.to_string())?,
    )
    .await?;
    let members = rows
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    json!({
                        "id": row["user_id"],
                        "name": row["display_name"],
                        "avatar": row["avatar"],
                        "color": row["color"],
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok((host_id, members))
}

impl EffectExecutor {
    pub fn set_watch_sink(&self, sink: ActionSink) {
        self.watch.lock().sink = Some(sink);
    }

    pub fn take_watch_commands(&self) -> Vec<Value> {
        self.watch.lock().commands.drain(..).collect()
    }

    pub(super) async fn watch_room(&self, payload: &Value) -> Result<Value, String> {
        let code = payload["code"].as_str().map(str::to_uppercase);
        match payload["op"].as_str() {
            Some("create") => self.enter_room(None, payload).await,
            Some("join") => {
                let code = code.ok_or_else(|| "room code is required".to_owned())?;
                self.enter_room(Some(code), payload).await
            }
            Some("leave") => {
                let code = code.ok_or_else(|| "room code is required".to_owned())?;
                self.leave_room(&code).await
            }
            _ => Err("unknown watch together operation".to_owned()),
        }
    }

    pub(super) fn watch_broadcast(&self, payload: &Value) -> Result<Value, String> {
        let room = payload["room"].as_str().unwrap_or_default();
        let inner = self.watch.lock();
        let connection = inner
            .connection
            .as_ref()
            .filter(|connection| connection.room == room)
            .ok_or_else(|| "not connected to the room".to_owned())?;
        connection
            .outbound
            .send(payload["message"].clone())
            .map_err(|_| "room connection closed".to_owned())?;
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

    async fn enter_room(&self, code: Option<String>, payload: &Value) -> Result<Value, String> {
        let config = Supabase::configured()?;
        let credentials = self.watch_credentials().await?;
        let client = client()?;
        let profile_id = payload["profileId"]
            .as_str()
            .filter(|_| !credentials.anonymous);
        let name = payload["displayName"].as_str().unwrap_or("Guest");
        let avatar = payload["avatar"].as_str();
        let room = match &code {
            None => {
                rpc(&client, &config, &credentials.token, "create_watch_room")
                    .json(&json!({"display_name": name, "profile_id": profile_id, "avatar": avatar}))
            }
            Some(code) => rpc(&client, &config, &credentials.token, "join_watch_room").json(
                &json!({"code": code, "display_name": name, "profile_id": profile_id, "avatar": avatar}),
            ),
        };
        let room = json_response(room.send().await.map_err(|error| error.to_string())?).await?;
        let room_code = room["code"]
            .as_str()
            .ok_or_else(|| "room response was invalid".to_owned())?
            .to_owned();
        let (host_id, members) =
            fetch_roster(&client, &config, &credentials.token, &room_code).await?;
        self.open_connection(room_code.clone());
        Ok(json!({
            "room": room_code,
            "clientId": credentials.user_id,
            "hostId": host_id,
            "members": members,
        }))
    }

    async fn leave_room(&self, code: &str) -> Result<Value, String> {
        let connection = {
            let mut inner = self.watch.lock();
            match inner.connection.as_ref() {
                Some(connection) if connection.room == code => inner.connection.take(),
                _ => None,
            }
        };
        let config = Supabase::configured()?;
        let credentials = self.watch_credentials().await?;
        let result = rpc(&client()?, &config, &credentials.token, "leave_watch_room")
            .json(&json!({"code": code}))
            .send()
            .await
            .map_err(|error| error.to_string());
        if let Some(connection) = connection {
            connection.task.abort();
        }
        json_response(result?).await.map(|_| Value::Null)
    }

    fn open_connection(&self, room: String) {
        let (outbound, inbox) = unbounded_channel();
        let executor = self.clone();
        let task_room = room.clone();
        let task = runtime().spawn(async move {
            executor.keep_connected(task_room, inbox).await;
        });
        let previous = self.watch.lock().connection.replace(Connection {
            room,
            outbound,
            task,
        });
        if let Some(previous) = previous {
            previous.task.abort();
        }
    }

    async fn keep_connected(&self, room: String, mut inbox: UnboundedReceiver<Value>) {
        let mut delay = Duration::from_secs(1);
        loop {
            let started = std::time::Instant::now();
            match self.realtime_session(&room, &mut inbox).await {
                Ok(()) => return,
                Err(error) => crate::log!("[watch-together] connection lost: {error}"),
            }
            if started.elapsed() > Duration::from_secs(60) {
                delay = Duration::from_secs(1);
            }
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(Duration::from_secs(30));
        }
    }

    async fn realtime_session(
        &self,
        room: &str,
        inbox: &mut UnboundedReceiver<Value>,
    ) -> Result<(), String> {
        let config = Supabase::configured()?;
        let mut credentials = self.watch_credentials().await?;
        let socket_url = format!(
            "{}/realtime/v1/websocket?apikey={}&vsn=1.0.0",
            config.base.replacen("http", "ws", 1),
            config.key
        );
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (mut socket, _) = tokio_tungstenite::connect_async(socket_url)
            .await
            .map_err(|error| error.to_string())?;
        let topic = format!("realtime:watch-together:{room}");
        let mut counter = 0u64;
        let mut next_ref = move || {
            counter += 1;
            counter.to_string()
        };
        let join_ref = next_ref();
        let frame = |event: &str, payload: Value, reference: String| {
            json!({"topic": topic, "event": event, "payload": payload, "ref": reference})
                .to_string()
        };
        socket
            .send(Message::text(
                json!({
                    "topic": topic,
                    "event": "phx_join",
                    "ref": join_ref,
                    "join_ref": join_ref,
                    "payload": {
                        "config": {
                            "broadcast": {"self": false, "ack": false},
                            "presence": {"key": credentials.user_id},
                            "private": true,
                        },
                        "access_token": credentials.token,
                    },
                })
                .to_string(),
            ))
            .await
            .map_err(|error| error.to_string())?;
        let client = client()?;
        let mut heartbeat = tokio::time::interval(HEARTBEAT);
        heartbeat.tick().await;
        loop {
            tokio::select! {
                incoming = socket.next() => {
                    let Some(incoming) = incoming else {
                        return Err("socket closed".to_owned());
                    };
                    let text = match incoming.map_err(|error| error.to_string())? {
                        Message::Text(text) => text,
                        Message::Close(_) => return Err("socket closed".to_owned()),
                        _ => continue,
                    };
                    let Ok(event) = serde_json::from_str::<Value>(text.as_str()) else {
                        continue;
                    };
                    if event["topic"] != topic {
                        continue;
                    }
                    match event["event"].as_str() {
                        Some("broadcast") if event["payload"]["event"] == "wt" => {
                            self.watch.emit(json!({
                                "type": "watchTogetherMessageReceived",
                                "message": event["payload"]["payload"],
                                "nowMs": now_ms(),
                            }));
                        }
                        Some("phx_reply") if event["ref"].as_str() == Some(join_ref.as_str()) => {
                            if event["payload"]["status"] != "ok" {
                                return Err(format!("join rejected: {}", event["payload"]["response"]));
                            }
                            let track = json!({
                                "type": "presence",
                                "event": "TRACK",
                                "payload": {"user_id": credentials.user_id},
                            });
                            socket
                                .send(Message::text(frame("presence", track, next_ref())))
                                .await
                                .map_err(|error| error.to_string())?;
                            self.publish_roster(&client, &config, &credentials, room).await;
                        }
                        Some("presence_diff") => {
                            self.publish_roster(&client, &config, &credentials, room).await;
                        }
                        Some("phx_error" | "phx_close") => return Err("channel closed".to_owned()),
                        _ => {}
                    }
                }
                outgoing = inbox.recv() => {
                    let Some(message) = outgoing else {
                        let _ = socket.close(None).await;
                        return Ok(());
                    };
                    let payload = json!({"type": "broadcast", "event": "wt", "payload": message});
                    socket
                        .send(Message::text(frame("broadcast", payload, next_ref())))
                        .await
                        .map_err(|error| error.to_string())?;
                }
                _ = heartbeat.tick() => {
                    socket
                        .send(Message::text(
                            json!({"topic": "phoenix", "event": "heartbeat", "payload": {}, "ref": next_ref()})
                                .to_string(),
                        ))
                        .await
                        .map_err(|error| error.to_string())?;
                    let fresh = self.watch_credentials().await?;
                    if fresh.token != credentials.token {
                        credentials = fresh;
                        socket
                            .send(Message::text(frame(
                                "access_token",
                                json!({"access_token": credentials.token}),
                                next_ref(),
                            )))
                            .await
                            .map_err(|error| error.to_string())?;
                    }
                }
            }
        }
    }

    async fn publish_roster(
        &self,
        client: &Client,
        config: &Supabase,
        credentials: &Credentials,
        room: &str,
    ) {
        match fetch_roster(client, config, &credentials.token, room).await {
            Ok((host_id, members)) => self.watch.emit(json!({
                "type": "watchTogetherMembersChanged",
                "hostId": host_id,
                "members": members,
                "nowMs": now_ms(),
            })),
            Err(error) => crate::log!("[watch-together] roster refresh failed: {error}"),
        }
    }
}
