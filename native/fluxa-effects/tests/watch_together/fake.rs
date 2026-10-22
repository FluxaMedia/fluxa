use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use http_body_util::{BodyExt, Full};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio_tungstenite::tungstenite::Message;

const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

struct Member {
    id: String,
    name: String,
    avatar: Value,
}

struct Room {
    host: String,
    locked: bool,
    banned: HashSet<String>,
    members: Vec<Member>,
}

struct Conn {
    tx: UnboundedSender<String>,
    topics: HashSet<String>,
}

#[derive(Default)]
struct State {
    users: HashMap<String, String>,
    rooms: HashMap<String, Room>,
    conns: HashMap<usize, Conn>,
    counter: usize,
}

type Shared = Arc<Mutex<State>>;

pub fn serve(port: u16) {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("fake runtime");
        runtime.block_on(async move {
            let listener = match TcpListener::bind(("127.0.0.1", port)).await {
                Ok(listener) => listener,
                Err(error) => {
                    ready_tx.send(Err(error.to_string())).ok();
                    return;
                }
            };
            ready_tx.send(Ok(())).ok();
            let state = Shared::default();
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };
                tokio::spawn(connection(stream, state.clone()));
            }
        });
    });
    ready_rx
        .recv()
        .expect("fake server")
        .unwrap_or_else(|error| panic!("cannot bind fake Supabase on port {port}: {error}"));
}

async fn connection(stream: TcpStream, state: Shared) {
    let mut head = [0u8; 2048];
    let Ok(read) = stream.peek(&mut head).await else {
        return;
    };
    let head = String::from_utf8_lossy(&head[..read]).to_lowercase();
    if head.contains("upgrade: websocket") {
        socket(stream, state).await;
        return;
    }
    let service = service_fn(move |request: Request<hyper::body::Incoming>| {
        let state = state.clone();
        async move { Ok::<_, Infallible>(rest(&state, request).await) }
    });
    let _ = hyper::server::conn::http1::Builder::new()
        .serve_connection(TokioIo::new(stream), service)
        .await;
}

fn reply(status: u16, body: Value) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}

fn failure(status: u16, message: &str) -> Response<Full<Bytes>> {
    reply(status, json!({"message": message}))
}

fn code_for(n: usize) -> String {
    let mut n = n + 1_000;
    let mut code = String::new();
    for _ in 0..6 {
        code.push(ALPHABET[n % ALPHABET.len()] as char);
        n /= ALPHABET.len();
    }
    code
}

async fn rest(state: &Shared, request: Request<hyper::body::Incoming>) -> Response<Full<Bytes>> {
    let (parts, body) = request.into_parts();
    let body: Value =
        serde_json::from_slice(&body.collect().await.unwrap().to_bytes()).unwrap_or(Value::Null);
    let path = parts.uri.path().to_owned();
    let query = parts.uri.query().unwrap_or("").to_owned();
    let token = parts
        .headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned);
    let mut guard = state.lock().unwrap();
    let state = &mut *guard;

    if path == "/auth/v1/signup" {
        state.counter += 1;
        let id = format!("user-{}", state.counter);
        let token = format!("token-{}", state.counter);
        state.users.insert(token.clone(), id.clone());
        return reply(
            200,
            json!({
                "access_token": token,
                "refresh_token": format!("refresh-{}", state.counter),
                "expires_in": 3600,
                "user": {"id": id},
            }),
        );
    }

    let Some(user) = token.and_then(|token| state.users.get(&token).cloned()) else {
        return failure(401, "invalid token");
    };

    if let Some(name) = path.strip_prefix("/rest/v1/rpc/") {
        let code = body["code"].as_str().unwrap_or_default().to_uppercase();
        let touched = match name {
            "create_watch_room" => {
                state.counter += 1;
                let code = code_for(state.counter);
                state.rooms.insert(
                    code.clone(),
                    Room {
                        host: user.clone(),
                        locked: false,
                        banned: HashSet::new(),
                        members: vec![Member {
                            id: user,
                            name: body["display_name"].as_str().unwrap_or("Guest").to_owned(),
                            avatar: body["avatar"].clone(),
                        }],
                    },
                );
                drop(guard);
                return reply(200, json!({"code": code}));
            }
            "join_watch_room" => {
                let Some(room) = state.rooms.get_mut(&code) else {
                    return failure(404, "room not found");
                };
                if room.banned.contains(&user) {
                    return failure(403, "you were removed from this room");
                }
                let present = room.members.iter().any(|member| member.id == user);
                if room.locked && !present {
                    return failure(403, "room is locked");
                }
                if !present {
                    room.members.push(Member {
                        id: user,
                        name: body["display_name"].as_str().unwrap_or("Guest").to_owned(),
                        avatar: body["avatar"].clone(),
                    });
                }
                code
            }
            "leave_watch_room" => {
                let Some(room) = state.rooms.get_mut(&code) else {
                    return reply(200, Value::Null);
                };
                room.members.retain(|member| member.id != user);
                if room.host == user || room.members.is_empty() {
                    state.rooms.remove(&code);
                }
                code
            }
            "kick_watch_room_member" => {
                let Some(room) = state.rooms.get_mut(&code) else {
                    return failure(404, "room not found");
                };
                if room.host != user {
                    return failure(403, "only the host can remove members");
                }
                let target = body["member"].as_str().unwrap_or_default().to_owned();
                room.members.retain(|member| member.id != target);
                room.banned.insert(target);
                code
            }
            "set_watch_room_locked" => {
                let Some(room) = state.rooms.get_mut(&code) else {
                    return failure(404, "room not found");
                };
                if room.host != user {
                    return failure(403, "only the host can lock the room");
                }
                room.locked = body["lock"].as_bool().unwrap_or(false);
                code
            }
            _ => return failure(404, "unknown function"),
        };
        announce(state, &touched);
        return reply(200, json!({"code": touched}));
    }

    let code = query
        .split('&')
        .find_map(|pair| {
            pair.strip_prefix("code=eq.")
                .or_else(|| pair.strip_prefix("room_code=eq."))
        })
        .unwrap_or_default()
        .to_owned();
    let Some(room) = state
        .rooms
        .get(&code)
        .filter(|room| room.members.iter().any(|member| member.id == user))
    else {
        return reply(200, json!([]));
    };
    match path.as_str() {
        "/rest/v1/watch_rooms" => reply(
            200,
            json!([{"host_id": room.host, "locked": room.locked, "max_members": 8}]),
        ),
        "/rest/v1/watch_room_members" => reply(
            200,
            Value::Array(
                room.members
                    .iter()
                    .map(|member| {
                        json!({
                            "user_id": member.id,
                            "display_name": member.name,
                            "avatar": member.avatar,
                            "color": null,
                        })
                    })
                    .collect(),
            ),
        ),
        _ => failure(404, "unknown table"),
    }
}

fn announce(state: &State, code: &str) {
    let needle = format!(":{code}:");
    for conn in state.conns.values() {
        let Some(topic) = conn.topics.iter().find(|topic| topic.contains(&needle)) else {
            continue;
        };
        let _ = conn.tx.send(
            json!({
                "topic": topic,
                "event": "presence_diff",
                "payload": {"joins": {}, "leaves": {}},
                "ref": null,
            })
            .to_string(),
        );
    }
}

async fn socket(stream: TcpStream, state: Shared) {
    let Ok(socket) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    let (mut sink, mut source) = socket.split();
    let (tx, mut outbox) = unbounded_channel::<String>();
    let id = {
        let mut guard = state.lock().unwrap();
        guard.counter += 1;
        let id = guard.counter;
        guard.conns.insert(
            id,
            Conn {
                tx: tx.clone(),
                topics: HashSet::new(),
            },
        );
        id
    };
    let writer = tokio::spawn(async move {
        while let Some(text) = outbox.recv().await {
            if sink.send(Message::text(text)).await.is_err() {
                break;
            }
        }
    });
    while let Some(Ok(message)) = source.next().await {
        let Message::Text(text) = message else {
            continue;
        };
        let Ok(frame) = serde_json::from_str::<Value>(text.as_str()) else {
            continue;
        };
        let topic = frame["topic"].as_str().unwrap_or_default().to_owned();
        let ack = json!({
            "topic": topic,
            "event": "phx_reply",
            "ref": frame["ref"],
            "payload": {"status": "ok", "response": {}},
        })
        .to_string();
        let mut guard = state.lock().unwrap();
        match frame["event"].as_str() {
            Some("phx_join") => {
                guard.conns.get_mut(&id).unwrap().topics.insert(topic);
                let _ = tx.send(ack);
            }
            Some("phx_leave") => {
                guard.conns.get_mut(&id).unwrap().topics.remove(&topic);
                let _ = tx.send(ack);
            }
            Some("heartbeat") => {
                let _ = tx.send(ack);
            }
            Some("broadcast") => {
                let out = json!({
                    "topic": topic,
                    "event": "broadcast",
                    "payload": frame["payload"],
                    "ref": null,
                })
                .to_string();
                for (other, conn) in &guard.conns {
                    if *other != id && conn.topics.contains(&topic) {
                        let _ = conn.tx.send(out.clone());
                    }
                }
            }
            Some("presence") => {
                if let Some(code) = topic.split(':').nth(2) {
                    announce(&guard, code);
                }
            }
            _ => {}
        }
    }
    state.lock().unwrap().conns.remove(&id);
    writer.abort();
}
