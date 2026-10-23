use super::fluxa_account::{Credentials, Supabase, client};
use super::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use web_time::Instant;

mod socket;
use socket::{Socket, Task, Ticker};
pub(crate) use socket::{sleep, spawn};

pub type ActionSink = Arc<dyn Fn(Value) + Send + Sync>;

const HEARTBEAT: Duration = Duration::from_secs(25);
const MAX_MESSAGE_BYTES: usize = 8 * 1024;
const FLOOD_WINDOW: Duration = Duration::from_secs(2);
const FLOOD_LIMIT: u32 = 40;

struct Flood {
    since: Instant,
    count: u32,
}

impl Default for Flood {
    fn default() -> Self {
        Self {
            since: Instant::now(),
            count: 0,
        }
    }
}

impl Flood {
    fn exceeded(&mut self) -> bool {
        if self.since.elapsed() > FLOOD_WINDOW {
            self.since = Instant::now();
            self.count = 0;
        }
        self.count += 1;
        self.count > FLOOD_LIMIT
    }
}

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
    task: Task,
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

struct Lanes {
    host_prefix: String,
    member_prefix: String,
    joined: HashMap<String, String>,
    host_id: String,
}

impl Lanes {
    fn sender(&self, topic: &str) -> Option<String> {
        let id = topic
            .strip_prefix(self.member_prefix.as_str())
            .or_else(|| topic.strip_prefix(self.host_prefix.as_str()))?;
        (!id.is_empty()).then(|| id.to_owned())
    }
}

fn frame(topic: &str, event: &str, payload: Value, reference: String) -> String {
    json!({"topic": topic, "event": event, "payload": payload, "ref": reference}).to_string()
}

async fn send_join(
    socket: &mut Socket,
    topic: &str,
    credentials: &Credentials,
    reference: &str,
) -> Result<(), String> {
    let join = json!({
        "topic": topic,
        "event": "phx_join",
        "ref": reference,
        "join_ref": reference,
        "payload": {
            "config": {
                "broadcast": {"self": false, "ack": false},
                "presence": {"key": credentials.user_id},
                "private": true,
            },
            "access_token": credentials.token,
        },
    });
    socket.send(join.to_string()).await
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
) -> Result<(String, bool, u64, Vec<Value>), String> {
    let room = json_response(
        rest(
            client,
            config,
            token,
            &format!("watch_rooms?code=eq.{code}&select=host_id,locked,max_members"),
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
    let locked = room[0]["locked"].as_bool().unwrap_or(false);
    let max_members = room[0]["max_members"].as_u64().unwrap_or(0);
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
    Ok((host_id, locked, max_members, members))
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
            Some("kick") => {
                let code = code.ok_or_else(|| "room code is required".to_owned())?;
                let member = payload["memberId"].as_str().unwrap_or_default();
                self.room_admin(
                    "kick_watch_room_member",
                    json!({"code": code, "member": member}),
                )
                .await
            }
            Some("lock") => {
                let code = code.ok_or_else(|| "room code is required".to_owned())?;
                let lock = payload["locked"].as_bool().unwrap_or(false);
                self.room_admin("set_watch_room_locked", json!({"code": code, "lock": lock}))
                    .await
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
            .ok_or_else(|| "room not found".to_owned())?
            .to_owned();
        let (host_id, _, max_members, members) =
            fetch_roster(&client, &config, &credentials.token, &room_code).await?;
        self.open_connection(room_code.clone());
        Ok(json!({
            "room": room_code,
            "clientId": credentials.user_id,
            "hostId": host_id,
            "members": members,
            "maxMembers": max_members,
        }))
    }

    async fn room_admin(&self, function: &str, body: Value) -> Result<Value, String> {
        let config = Supabase::configured()?;
        let credentials = self.watch_credentials().await?;
        let response = rpc(&client()?, &config, &credentials.token, function)
            .json(&body)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        json_response(response).await.map(|_| Value::Null)
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
        let task = socket::spawn(async move {
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
            let started = Instant::now();
            match self.realtime_session(&room, &mut inbox).await {
                Ok(()) => return,
                Err(error) => crate::log!("[watch-together] connection lost: {error}"),
            }
            if started.elapsed() > Duration::from_secs(60) {
                delay = Duration::from_secs(1);
            }
            socket::sleep(delay).await;
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
        let mut socket = Socket::connect(&socket_url).await?;
        let mut lanes = Lanes {
            host_prefix: format!("realtime:watch-together:{room}:h:"),
            member_prefix: format!("realtime:watch-together:{room}:u:"),
            joined: HashMap::new(),
            host_id: String::new(),
        };
        let own_topic = format!("{}{}", lanes.member_prefix, credentials.user_id);
        let mut counter = 0u64;
        let mut next_ref = move || {
            counter += 1;
            counter.to_string()
        };
        let reference = next_ref();
        send_join(&mut socket, &own_topic, &credentials, &reference).await?;
        lanes.joined.insert(own_topic.clone(), reference);
        let mut flooding: HashMap<String, Flood> = HashMap::new();
        let mut kicked: HashSet<String> = HashSet::new();
        let client = client()?;
        let mut heartbeat = Ticker::new(HEARTBEAT).await;
        loop {
            tokio::select! {
                incoming = socket.recv() => {
                    let text = incoming?;
                    let Ok(event) = serde_json::from_str::<Value>(&text) else {
                        continue;
                    };
                    let Some(topic) = event["topic"].as_str() else {
                        continue;
                    };
                    if !lanes.joined.contains_key(topic) {
                        continue;
                    }
                    let topic = topic.to_owned();
                    match event["event"].as_str() {
                        Some("broadcast") if event["payload"]["event"] == "wt" => {
                            let mut message = event["payload"]["payload"].clone();
                            if message.to_string().len() > MAX_MESSAGE_BYTES {
                                continue;
                            }
                            let Some(sender) = lanes.sender(&topic) else {
                                continue;
                            };
                            if !message.is_object() {
                                continue;
                            }
                            if flooding.entry(topic.clone()).or_default().exceeded() {
                                if lanes.host_id == credentials.user_id && kicked.insert(sender.clone()) {
                                    self.watch.emit(json!({
                                        "type": "watchTogetherKickRequested",
                                        "memberId": sender,
                                    }));
                                }
                                continue;
                            }
                            message["from"] = json!(sender);
                            self.watch.emit(json!({
                                "type": "watchTogetherMessageReceived",
                                "message": message,
                                "nowMs": now_ms(),
                            }));
                        }
                        Some("phx_reply") => {
                            let reference = event["ref"].as_str().unwrap_or_default();
                            if lanes.joined.get(&topic).map(String::as_str) != Some(reference) {
                                continue;
                            }
                            if event["payload"]["status"] != "ok" {
                                if topic == own_topic {
                                    return Err(format!("join rejected: {}", event["payload"]["response"]));
                                }
                                lanes.joined.remove(&topic);
                                continue;
                            }
                            if topic.starts_with(lanes.host_prefix.as_str())
                                && lanes.host_id != credentials.user_id
                            {
                                let hello = json!({"type": "broadcast", "event": "wt", "payload": {"type": "hello"}});
                                socket
                                    .send(frame(&own_topic, "broadcast", hello, next_ref()))
                                    .await?;
                            }
                            if topic == own_topic {
                                let track = json!({
                                    "type": "presence",
                                    "event": "TRACK",
                                    "payload": {"user_id": credentials.user_id},
                                });
                                socket
                                    .send(frame(&own_topic, "presence", track, next_ref()))
                                    .await?;
                                self.refresh_lanes(
                                    &mut socket, &client, &config, &credentials, room,
                                    &mut lanes, &mut next_ref,
                                ).await?;
                            }
                        }
                        Some("presence_diff") => {
                            self.refresh_lanes(
                                &mut socket, &client, &config, &credentials, room,
                                &mut lanes, &mut next_ref,
                            ).await?;
                        }
                        Some("phx_error" | "phx_close") => return Err("channel closed".to_owned()),
                        _ => {}
                    }
                }
                outgoing = inbox.recv() => {
                    let Some(message) = outgoing else {
                        socket.close().await;
                        return Ok(());
                    };
                    let topic = match message["type"].as_str() {
                        Some("content" | "start" | "state") => {
                            format!("{}{}", lanes.host_prefix, credentials.user_id)
                        }
                        _ => own_topic.clone(),
                    };
                    if !lanes.joined.contains_key(&topic) {
                        continue;
                    }
                    let payload = json!({"type": "broadcast", "event": "wt", "payload": message});
                    socket
                        .send(frame(&topic, "broadcast", payload, next_ref()))
                        .await?;
                }
                _ = heartbeat.tick() => {
                    socket
                        .send(
                            json!({"topic": "phoenix", "event": "heartbeat", "payload": {}, "ref": next_ref()})
                                .to_string(),
                        )
                        .await?;
                    let fresh = self.watch_credentials().await?;
                    if fresh.token != credentials.token {
                        credentials = fresh;
                        for topic in lanes.joined.keys() {
                            socket
                                .send(frame(
                                    topic,
                                    "access_token",
                                    json!({"access_token": credentials.token}),
                                    next_ref(),
                                ))
                                .await?;
                        }
                    }
                    self.refresh_lanes(
                        &mut socket, &client, &config, &credentials, room,
                        &mut lanes, &mut next_ref,
                    ).await?;
                }
            }
        }
    }

    async fn refresh_lanes(
        &self,
        socket: &mut Socket,
        client: &Client,
        config: &Supabase,
        credentials: &Credentials,
        room: &str,
        lanes: &mut Lanes,
        next_ref: &mut impl FnMut() -> String,
    ) -> Result<(), String> {
        let Some((host_id, ids)) = self.publish_roster(client, config, credentials, room).await
        else {
            return Ok(());
        };
        lanes.host_id = host_id.clone();
        let mut wanted: Vec<String> = ids
            .iter()
            .map(|id| format!("{}{id}", lanes.member_prefix))
            .collect();
        wanted.push(format!("{}{host_id}", lanes.host_prefix));
        let own_member = format!("{}{}", lanes.member_prefix, credentials.user_id);
        let own_host = format!("{}{}", lanes.host_prefix, credentials.user_id);
        let stale: Vec<String> = lanes
            .joined
            .keys()
            .filter(|topic| !wanted.contains(topic) && **topic != own_member)
            .cloned()
            .collect();
        for topic in stale {
            lanes.joined.remove(&topic);
            socket
                .send(frame(&topic, "phx_leave", json!({}), next_ref()))
                .await?;
        }
        for topic in wanted {
            if lanes.joined.contains_key(&topic)
                || (topic == own_host && host_id != credentials.user_id)
            {
                continue;
            }
            let reference = next_ref();
            send_join(socket, &topic, credentials, &reference).await?;
            lanes.joined.insert(topic, reference);
        }
        Ok(())
    }

    async fn publish_roster(
        &self,
        client: &Client,
        config: &Supabase,
        credentials: &Credentials,
        room: &str,
    ) -> Option<(String, Vec<String>)> {
        match fetch_roster(client, config, &credentials.token, room).await {
            Ok((host_id, locked, max_members, members)) => {
                let ids = members
                    .iter()
                    .filter_map(|member| member["id"].as_str().map(ToOwned::to_owned))
                    .collect();
                self.watch.emit(json!({
                    "type": "watchTogetherMembersChanged",
                    "hostId": host_id,
                    "members": members,
                    "locked": locked,
                    "maxMembers": max_members,
                    "nowMs": now_ms(),
                }));
                Some((host_id, ids))
            }
            Err(error) => {
                if error == "room not found" {
                    self.watch
                        .emit(json!({"type": "watchTogetherLeaveRequested"}));
                }
                crate::log!("[watch-together] roster refresh failed: {error}");
                None
            }
        }
    }
}
