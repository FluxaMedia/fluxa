use super::*;
use crate::player::VideoCommand;

const TICK: Duration = Duration::from_millis(250);
const SAMPLE: Duration = Duration::from_millis(500);
const POSITION_JUMP: f64 = 2.0;
const SETTLE: Duration = Duration::from_millis(1500);

#[derive(Default)]
pub(crate) struct State {
    room: Option<String>,
    status_sent: Option<(&'static str, Option<String>)>,
    published: bool,
    shared: bool,
    last_sample: Option<Instant>,
    last_tick: Option<Instant>,
    correcting: bool,
    rate: Option<f64>,
    selected: Option<String>,
    settle_until: Option<Instant>,
    event_seen: u64,
    chat_seen: u64,
    keep_stream: bool,
}

fn now_ms() -> i64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}

fn send(state: &mut RendererState, command: VideoCommand) {
    if let Some(video) = state.video.as_mut() {
        video.command(command);
    }
}

fn set_paused(state: &mut RendererState, paused: bool) {
    let value = if paused { "yes" } else { "no" };
    send(
        state,
        VideoCommand::Mpv(vec!["set".to_owned(), "pause".to_owned(), value.to_owned()]),
    );
}

fn set_speed(state: &mut RendererState, rate: f64) {
    send(state, VideoCommand::SetSpeed(rate));
}

fn seek_to(state: &mut RendererState, position: f64) {
    state.watch.settle_until = Some(Instant::now() + SETTLE);
    send(state, VideoCommand::SeekTo(position));
}

fn base_rate(state: &RendererState) -> f64 {
    state.watch.rate.unwrap_or(1.0)
}

fn same_content(a: &Value, b: &Value) -> bool {
    a["id"] == b["id"]
        && (a["videoId"].is_null() || b["videoId"].is_null() || a["videoId"] == b["videoId"])
}

fn party_key(content: &Value) -> String {
    let artwork = &content["artwork"];
    let number = |value: &Value| value.as_u64().map(|n| n.to_string()).unwrap_or_default();
    format!(
        "{}|{}|{}|{}",
        content["contentType"].as_str().unwrap_or_default().to_lowercase(),
        content["id"].as_str().unwrap_or_default(),
        number(&artwork["season"]),
        number(&artwork["episode"]),
    )
}

fn in_sync(state: &RendererState, watch: &Value) -> bool {
    state.watch.shared
        || state
            .player
            .as_ref()
            .and_then(|player| player.watch_content())
            .is_some_and(|content| same_content(&watch["content"], &content))
}

pub(crate) fn intercept(state: &mut RendererState, command: &VideoCommand) -> bool {
    let Some(session) = state.session.as_ref() else {
        return true;
    };
    let watch = session
        .snapshot()
        .get("watchTogether")
        .cloned()
        .unwrap_or(Value::Null);
    if watch["roomCode"].is_null() || !in_sync(state, &watch) {
        return true;
    }
    let Some(status) = state.player.as_ref().map(|player| player.status.clone()) else {
        return true;
    };
    let position_ms = |seconds: f64| (seconds.max(0.0) * 1000.0) as i64;
    let control = match command {
        VideoCommand::TogglePause if status.paused => json!({"kind": "resumed"}),
        VideoCommand::TogglePause => json!({"kind": "paused"}),
        VideoCommand::Seek(delta) => {
            json!({"kind": "seeked", "positionMs": position_ms(status.position + delta)})
        }
        VideoCommand::SeekTo(position) => {
            json!({"kind": "seeked", "positionMs": position_ms(*position)})
        }
        VideoCommand::SetSpeed(rate) => json!({"kind": "speed", "rate": rate}),
        _ => return true,
    };
    if watch["locked"] == true && watch["role"] != "HOST" {
        let text = fluxa_ui::localized("watch.host_controls", state.settings.language());
        state.notice = Some((text, Instant::now()));
        return false;
    }
    if let VideoCommand::SetSpeed(rate) = command {
        state.watch.rate = Some(*rate);
        state.watch.correcting = false;
    }
    state.watch.settle_until = Some(Instant::now() + SETTLE);
    let mut action = control;
    action["type"] = json!("watchTogetherControlRequested");
    action["nowMs"] = json!(now_ms());
    dispatch(state, action);
    true
}

fn dispatch(state: &RendererState, action: Value) {
    if let Some(session) = state.session.as_ref() {
        let _ = session.dispatch(action);
    }
}

pub(crate) fn tick(state: &mut RendererState) {
    crate::preview::watch_panel_demo(state);
    crate::preview::player_demo(state);
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let commands = session.take_watch_commands();
    let snapshot = session.snapshot();
    let watch = snapshot
        .get("watchTogether")
        .cloned()
        .unwrap_or(Value::Null);
    refresh_view(state, &watch);
    let room = watch["roomCode"].as_str().map(ToOwned::to_owned);
    state.settings.watch_room = room.clone();
    if !crate::preview::watch_demo() {
        state.settings.watch_ticket = ticket(&watch);
    }
    if room != state.watch.room {
        if state.watch.room.is_some() && (state.watch.correcting || state.watch.rate.is_some()) {
            let rate = state.player.as_ref().map_or(1.0, |player| player.speed);
            set_speed(state, rate);
        }
        state.watch = State {
            room: room.clone(),
            keep_stream: state.watch.keep_stream,
            ..State::default()
        };
    }
    if room.is_none() {
        return;
    }
    for command in commands {
        apply(state, &command);
    }
    if state.watch.last_tick.is_none_or(|at| at.elapsed() >= TICK) {
        state.watch.last_tick = Some(Instant::now());
        dispatch(
            state,
            json!({"type": "watchTogetherTicked", "nowMs": now_ms()}),
        );
    }
    report(state, &watch);
    announce_waiting(state, &watch);
    announce_event(state, &watch);
    announce_chat(state, &watch);
}

fn announce_chat(state: &mut RendererState, watch: &Value) {
    let Some(last) = watch["chat"].as_array().and_then(|chat| chat.last()) else {
        return;
    };
    let id = last["id"].as_u64().unwrap_or_default();
    if id <= state.watch.chat_seen {
        return;
    }
    state.watch.chat_seen = id;
    if state.watch_panel || last["by"].as_str() == watch["clientId"].as_str() {
        return;
    }
    let text = format!(
        "{}: {}",
        last["name"].as_str().unwrap_or_default(),
        last["text"].as_str().unwrap_or_default()
    );
    state.notice = Some((text, Instant::now()));
}

fn announce_event(state: &mut RendererState, watch: &Value) {
    let event = &watch["lastEvent"];
    let Some(id) = event["id"].as_u64() else {
        return;
    };
    if id <= state.watch.event_seen {
        return;
    }
    state.watch.event_seen = id;
    if event["by"].as_str() == watch["clientId"].as_str() {
        return;
    }
    let language = state.settings.language();
    let key = match event["kind"].as_str() {
        Some("paused") => "watch.event.paused",
        Some("resumed") => "watch.event.resumed",
        Some("seeked") => "watch.event.seeked",
        Some("speed") => "watch.event.speed",
        _ => return,
    };
    let position = event["positionMs"].as_i64().unwrap_or_default() / 1000;
    let time = format!("{}:{:02}", position / 60, position % 60);
    let rate = event["rate"].as_f64().unwrap_or(1.0);
    let text = fluxa_ui::localized(key, language)
        .replace("{name}", event["name"].as_str().unwrap_or_default())
        .replace("{time}", &time)
        .replace("{rate}", &format!("{rate}×"));
    state.notice = Some((text, Instant::now()));
}

fn ticket(watch: &Value) -> fluxa_ui::WatchTicket {
    let content = &watch["content"];
    let artwork = &content["artwork"];
    let text = |value: &Value| {
        value
            .as_str()
            .filter(|text| !text.is_empty())
            .map(ToOwned::to_owned)
    };
    let episode = match (artwork["season"].as_u64(), artwork["episode"].as_u64()) {
        (Some(season), Some(episode)) => Some(match text(&artwork["episodeTitle"]) {
            Some(title) => format!("S{season} · E{episode} · {title}"),
            None => format!("S{season} · E{episode}"),
        }),
        _ => None,
    };
    let members = member_views(watch);
    let host = members.iter().find(|member| member.host).cloned();
    fluxa_ui::WatchTicket {
        host,
        members,
        max_members: watch["maxMembers"].as_u64().unwrap_or_default() as u32,
        room: text(&watch["roomCode"]),
        connecting: watch["connectionState"] == "CONNECTING",
        error: (watch["connectionState"] == "ERROR").then(|| {
            match watch["errorMessage"].as_str() {
                Some("same_profile") => "watch.error.same_profile",
                Some("ended") => "watch.ended",
                _ => "watch.error",
            }
            .to_owned()
        }),
        title: text(&content["title"]),
        logo: text(&artwork["logo"]),
        backdrop: text(&artwork["backdrop"]),
        episode,
    }
}

fn announce_waiting(state: &mut RendererState, watch: &Value) {
    if watch["phase"] != "playing" {
        return;
    }
    let you = watch["clientId"].as_str();
    let waiting = watch["members"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|member| member["buffering"] == true && member["id"].as_str() != you)
        .and_then(|member| member["name"].as_str());
    if let Some(name) = waiting {
        let text = fluxa_ui::localized("watch.waiting_for", state.settings.language());
        state.notice = Some((text.replace("{name}", name), Instant::now()));
    }
}

fn apply(state: &mut RendererState, command: &Value) {
    match command["command"].as_str() {
        Some("pickStream") => {
            let content = &command["content"];
            let mut item = json!({
                "id": content["id"],
                "type": content["contentType"],
                "name": content["title"],
                "pickSource": true,
            });
            if let Some(video) = content["videoId"].as_str() {
                item["lastVideoId"] = video.into();
            }
            open(state, NativeAction::StartPlayback { item });
        }
        Some("playShared") => {
            let url = command["stream"]["url"].as_str().unwrap_or_default();
            let title = command["content"]["title"].as_str().unwrap_or_default();
            if url.is_empty() {
                return;
            }
            state.watch.status_sent = None;
            state.watch.shared = true;
            player::close(state);
            let resolve = player::open_file(state, url, title);
            dispatch(state, resolve);
        }
        Some("remote") => {
            let position = command["positionMs"].as_f64().unwrap_or_default() / 1000.0;
            let playing = command["playing"].as_bool().unwrap_or(true);
            let rate = command["rate"].as_f64().unwrap_or(1.0);
            let local = state.player.as_ref().map(|player| player.status.position);
            if local.is_none_or(|local| (local - position).abs() > POSITION_JUMP) {
                seek_to(state, position);
            }
            set_paused(state, !playing);
            if state.watch.rate != Some(rate) || state.watch.correcting {
                state.watch.rate = Some(rate);
                state.watch.correcting = false;
                if let Some(player) = state.player.as_mut() {
                    player.speed = rate;
                }
                set_speed(state, rate);
            }
        }
        Some("seek") => {
            let position = command["positionMs"].as_f64().unwrap_or_default() / 1000.0;
            seek_to(state, position);
        }
        Some("speed") => {
            let factor = command["value"].as_f64().unwrap_or(1.0);
            state.watch.correcting = true;
            set_speed(state, base_rate(state) * factor);
        }
        Some("resetSpeed") => {
            state.watch.correcting = false;
            set_speed(state, base_rate(state));
        }
        _ => {}
    }
}

fn open(state: &mut RendererState, action: NativeAction) {
    state.watch.status_sent = None;
    state.watch.shared = false;
    player::close(state);
    state.pending_native_actions.push(action);
}

fn resolving(state: &RendererState) -> bool {
    !state.watch.keep_stream
        && state.session.as_ref().is_some_and(|session| {
            session
                .snapshot()
                .pointer("/player/resolvedUrl")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
        })
}

fn shared_stream(state: &RendererState, content: &Value) -> Option<Value> {
    let snapshot = state.session.as_ref()?.snapshot();
    let url = snapshot.pointer("/player/resolvedUrl")?.as_str()?;
    let host = url.split("://").nth(1)?.split(['/', ':']).next()?;
    if !url.starts_with("http") || matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "0.0.0.0") {
        return None;
    }
    let index = snapshot.pointer("/player/currentStreamIndex")?.as_u64()? as usize;
    let headers = snapshot
        .pointer("/player/currentStreams")?
        .get(index)?
        .pointer("/behaviorHints/proxyHeaders/request")
        .and_then(Value::as_object)
        .map(|headers| {
            headers
                .iter()
                .filter_map(|(name, value)| Some((name.clone(), json!(value.as_str()?))))
                .collect::<serde_json::Map<_, _>>()
        })
        .unwrap_or_default();
    Some(json!({"url": url, "title": content["title"], "headers": headers}))
}

fn set_status(
    state: &mut RendererState,
    name: &'static str,
    media_key: Option<String>,
) {
    let next = (name, media_key.clone());
    if state.watch.status_sent.as_ref() == Some(&next) {
        return;
    }
    state.watch.status_sent = Some(next);
    dispatch(
        state,
        json!({
            "type": "watchTogetherStatusChanged",
            "status": name,
            "mediaKey": media_key,
            "nowMs": now_ms(),
        }),
    );
}

fn report(state: &mut RendererState, watch: &Value) {
    let is_host = watch["role"] == "HOST";
    let Some(player) = state.player.as_ref() else {
        set_status(state, "browsing", None);
        return;
    };
    let status = player.status.clone();
    let content = player.watch_content();
    let announceable = |content: &Value| {
        content["contentType"] == "movie" || content["videoId"].as_str().is_some()
    };
    if is_host
        && !resolving(state)
        && let Some(content) = content.clone().filter(announceable)
    {
        let stream = (!state.watch.keep_stream)
            .then(|| shared_stream(state, &content))
            .flatten();
        let key = format!(
            "{}|{}|{}",
            content["id"],
            content["videoId"],
            stream
                .as_ref()
                .map_or("", |stream| stream["url"].as_str().unwrap_or_default())
        );
        if state.watch.selected.as_deref() != Some(key.as_str()) {
            state.watch.selected = Some(key);
            if stream.is_some() || !same_content(&watch["content"], &content) {
                state.watch.published = true;
                let start = state
                    .player
                    .as_ref()
                    .map_or(0, |player| player.start_offset_ms());
                dispatch(
                    state,
                    json!({
                        "type": "watchTogetherContentSelected",
                        "content": content,
                        "stream": stream,
                        "startPositionMs": start,
                        "nowMs": now_ms(),
                    }),
                );
                return;
            }
        }
    }
    let synced = in_sync(state, watch);
    let key = Some(party_key(&watch["content"])).filter(|_| synced);
    if !synced {
        set_status(state, "browsing", None);
        return;
    }
    if !status.has_frame {
        set_status(state, "loading", key);
        return;
    }
    let buffering = status.stalled;
    set_status(state, if buffering { "buffering" } else { "ready" }, key);
    let now = now_ms();
    if is_host && !state.watch.published {
        state.watch.published = true;
        let speed = state.player.as_ref().map_or(1.0, |player| player.speed);
        dispatch(
            state,
            json!({
                "type": "watchTogetherPlaybackPublished",
                "positionMs": (status.position * 1000.0) as i64,
                "playing": !status.paused,
                "rate": speed,
                "nowMs": now,
            }),
        );
    }
    if !buffering
        && state
            .watch
            .last_sample
            .is_none_or(|at| at.elapsed() >= SAMPLE)
        && state
            .watch
            .settle_until
            .is_none_or(|until| Instant::now() >= until)
    {
        state.watch.last_sample = Some(Instant::now());
        dispatch(
            state,
            json!({
                "type": "watchTogetherPositionSampled",
                "positionMs": (status.position * 1000.0) as i64,
                "durationMs": (status.duration * 1000.0) as i64,
                "nowMs": now,
                "speedCorrectionActive": state.watch.correcting,
            }),
        );
    }
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    match node {
        fluxa_ui::NODE_SETTINGS_WATCH_JOIN => {
            let code = std::mem::take(&mut state.settings.watch_code);
            if code.len() != 6 {
                state.settings.watch_code = code;
                return true;
            }
            let _ = state.ui.set_focus(None);
            handle(state, fluxa_ui::WatchTogetherRequest::Join(code));
        }
        _ => return false,
    }
    true
}

pub(crate) fn toggle_panel(state: &mut RendererState) {
    if state.player.is_none() {
        return;
    }
    state.watch_panel = !state.watch_panel;
    if state.watch_panel {
        open_modal(state);
    }
}

pub(crate) fn open_modal(state: &mut RendererState) {
    if state.watch_view.is_none() && state.session.is_some() {
        state.watch_view = Some(fluxa_ui::WatchTogetherModel {
            language: state.settings.language().to_owned(),
            ..Default::default()
        });
    }
}

pub(crate) fn handle(state: &mut RendererState, request: fluxa_ui::WatchTogetherRequest) {
    use fluxa_ui::WatchTogetherRequest as Request;
    match request {
        Request::Close => {
            state.watch_view = None;
            state.watch_panel = false;
        }
        Request::Create => {
            let Some(content) = state.player.as_ref().and_then(|player| player.watch_content())
            else {
                return;
            };
            let (name, profile_id, avatar) = identity(state);
            dispatch(
                state,
                json!({
                    "type": "watchTogetherCreateRequested",
                    "content": content,
                    "displayName": name,
                    "profileId": profile_id,
                    "avatar": avatar,
                    "profileKey": profile_key(&name, avatar.as_deref()),
                }),
            );
        }
        Request::Join(code) => {
            let (name, profile_id, avatar) = identity(state);
            dispatch(
                state,
                json!({
                    "type": "watchTogetherJoinRequested",
                    "code": code,
                    "displayName": name,
                    "profileId": profile_id,
                    "avatar": avatar,
                    "profileKey": profile_key(&name, avatar.as_deref()),
                }),
            );
        }
        Request::Kick(id) => dispatch(
            state,
            json!({"type": "watchTogetherKickRequested", "memberId": id}),
        ),
        Request::SetLocked(locked) => dispatch(
            state,
            json!({"type": "watchTogetherLockChanged", "locked": locked}),
        ),
        Request::SetShareStream(share) => {
            state.watch.keep_stream = !share;
        }
        Request::Leave => dispatch(state, json!({"type": "watchTogetherLeaveRequested"})),
        Request::End => dispatch(state, json!({"type": "watchTogetherEndRequested"})),
        Request::Chat(text) => dispatch(
            state,
            json!({"type": "watchTogetherChatSent", "text": text, "nowMs": now_ms()}),
        ),
        Request::CopyCode => {
            state.copied = state
                .watch_view
                .as_ref()
                .and_then(|view| view.room_code.clone());
        }
    }
}

fn profile_key(name: &str, avatar: Option<&str>) -> Option<String> {
    avatar.map(|avatar| format!("{name}|{avatar}"))
}

fn identity(state: &RendererState) -> (String, Option<String>, Option<String>) {
    let profile = state
        .session
        .as_ref()
        .map_or(Value::Null, |session| session.active_profile());
    let text = |key: &str| {
        profile
            .get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    };
    let id = text("id").filter(|id| is_uuid(id));
    (
        text("name").unwrap_or_else(|| "Guest".to_owned()),
        id,
        text("avatarUrl").filter(|url| url.starts_with("https://")),
    )
}

fn is_uuid(value: &str) -> bool {
    value.len() == 36
        && value.split('-').map(str::len).eq([8, 4, 4, 4, 12])
        && value.chars().all(|c| c == '-' || c.is_ascii_hexdigit())
}

fn member_views(watch: &Value) -> Vec<fluxa_ui::WatchMemberView> {
    let you = watch["clientId"].as_str();
    watch["members"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|member| fluxa_ui::WatchMemberView {
            id: member["id"].as_str().unwrap_or_default().to_owned(),
            name: member["name"].as_str().unwrap_or_default().to_owned(),
            avatar: member["avatar"].as_str().map(ToOwned::to_owned),
            ready: member["ready"].as_bool().unwrap_or(false),
            buffering: member["buffering"].as_bool().unwrap_or(false),
            status: member["status"].as_str().unwrap_or("waiting").to_owned(),
            host: member["isHost"].as_bool().unwrap_or(false),
            you: member["id"].as_str() == you,
        })
        .collect()
}

fn refresh_view(state: &mut RendererState, watch: &Value) {
    let keep_stream = state.watch.keep_stream;
    if crate::preview::watch_demo() {
        return;
    }
    let Some(view) = state.watch_view.as_mut() else {
        return;
    };
    view.room_code = watch["roomCode"].as_str().map(ToOwned::to_owned);
    view.connecting = watch["connectionState"] == "CONNECTING";
    view.error = (watch["connectionState"] == "ERROR").then(|| "watch.error".to_owned());
    let you = watch["clientId"].as_str();
    view.host = watch["role"] == "HOST";
    view.locked = watch["locked"].as_bool().unwrap_or(false);
    view.share_stream = !keep_stream;
    view.max_members = watch["maxMembers"].as_u64().unwrap_or_default() as u32;
    view.members = member_views(watch);
    view.chat = watch["chat"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|message| fluxa_ui::WatchChatView {
            id: message["id"].as_u64().unwrap_or_default(),
            by: message["by"].as_str().unwrap_or_default().to_owned(),
            avatar: view
                .members
                .iter()
                .find(|member| message["by"].as_str() == Some(member.id.as_str()))
                .and_then(|member| member.avatar.clone()),
            name: message["name"].as_str().unwrap_or_default().to_owned(),
            text: message["text"].as_str().unwrap_or_default().to_owned(),
            you: message["by"].as_str() == you,
        })
        .collect();
}
