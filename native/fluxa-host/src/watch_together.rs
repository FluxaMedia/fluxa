use super::*;
use crate::player::VideoCommand;

const HOST_REPORT: Duration = Duration::from_secs(10);
const SAMPLE: Duration = Duration::from_millis(500);
const POSITION_JUMP: f64 = 2.0;const SETTLE: Duration = Duration::from_millis(1500);

#[derive(Default)]
pub(crate) struct State {
    room: Option<String>,
    opened: Option<String>,
    ready_sent: bool,
    ready_key: Option<String>,
    shared: bool,
    buffering_sent: bool,
    last_report: Option<(Instant, bool, f64, f64)>,
    last_sample: Option<Instant>,
    correcting: bool,
    rate: Option<f64>,
    selected: Option<String>,
    settle_until: Option<Instant>,
    event_seen: u64,
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

pub(crate) fn intercept(state: &mut RendererState, command: &VideoCommand) -> bool {
    let Some(session) = state.session.as_ref() else {
        return true;
    };
    let watch = session.snapshot().get("watchTogether").cloned().unwrap_or(Value::Null);
    if watch["phase"] != "playing" || watch["role"] != "GUEST" {
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
    if watch["locked"] == true {
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
            ..State::default()
        };
    }
    if room.is_none() {
        return;
    }
    for command in commands {
        apply(state, &command);
    }
    report(state, &watch);
    announce_waiting(state, &watch);
    announce_mismatch(state, &watch);
    announce_event(state, &watch);
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
    fluxa_ui::WatchTicket {
        room: text(&watch["roomCode"]),
        connecting: watch["connectionState"] == "CONNECTING",
        error: (watch["connectionState"] == "ERROR").then(|| "watch.error".to_owned()),
        title: text(&content["title"]),
        logo: text(&artwork["logo"]),
        backdrop: text(&artwork["backdrop"]),
        episode,
        people: watch["members"].as_array().map_or(0, Vec::len),
    }
}

fn announce_mismatch(state: &mut RendererState, watch: &Value) {
    if watch["contentMismatch"] != true {
        return;
    }
    let title = watch["content"]["title"].as_str().unwrap_or_default();
    let text = fluxa_ui::localized("watch.content_mismatch", state.settings.language());
    state.notice = Some((text.replace("{title}", title), Instant::now()));
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
            open(state, content, NativeAction::StartPlayback { item });
        }
        Some("playShared") => {
            let url = command["stream"]["url"].as_str().unwrap_or_default();
            let title = command["content"]["title"].as_str().unwrap_or_default();
            if url.is_empty() {
                return;
            }
            let content = command["content"].clone();
            state.watch.opened = content["id"].as_str().map(ToOwned::to_owned);
            state.watch.ready_sent = false;
            state.watch.shared = true;
            player::close(state);
            let resolve = player::open_file(state, url, title);
            dispatch(state, resolve);
        }
        Some("start") => {
            let position = command["positionMs"].as_f64().unwrap_or_default() / 1000.0;
            seek_to(state, position);
            set_paused(state, false);
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

fn open(state: &mut RendererState, content: &Value, action: NativeAction) {
    state.watch.opened = content["id"].as_str().map(ToOwned::to_owned);
    state.watch.ready_sent = false;
    state.watch.shared = false;
    player::close(state);
    state.pending_native_actions.push(action);
}

fn report(state: &mut RendererState, watch: &Value) {
    let playing = watch["phase"] == "playing";
    let is_host = watch["role"] == "HOST";
    let Some(player) = state.player.as_ref() else {
        if state.watch.ready_sent {
            state.watch.ready_sent = false;
            dispatch(
                state,
                json!({"type": "watchTogetherReadyChanged", "ready": false, "nowMs": now_ms()}),
            );
        }
        return;
    };
    let status = player.status.clone();
    let speed = player.speed;
    let content = player.watch_content();
    if is_host && let Some(content) = content.clone() {
        let key = format!("{}|{}", content["id"], content["videoId"]);
        if state.watch.selected.as_deref() != Some(key.as_str()) {
            state.watch.selected = Some(key);
            state.watch.opened = content["id"].as_str().map(ToOwned::to_owned);
            state.watch.ready_sent = false;
            state.watch.last_report = None;
            dispatch(
                state,
                json!({"type": "watchTogetherContentSelected", "content": content, "startPositionMs": 0}),
            );
            return;
        }
    }
    if state.watch.opened.is_none() {
        return;
    }
    let now = now_ms();
    if !playing {
        let checked = !is_host && !state.watch.shared;
        let local = content.clone().filter(|_| checked);
        if checked && local.is_none() {
            return;
        }
        let key = local
            .as_ref()
            .map(|local| format!("{}|{}", local["id"], local["videoId"]));
        if status.has_frame && (!state.watch.ready_sent || state.watch.ready_key != key) {
            state.watch.ready_sent = true;
            state.watch.ready_key = key;
            set_paused(state, true);
            dispatch(
                state,
                json!({"type": "watchTogetherReadyChanged", "ready": true, "content": local, "nowMs": now}),
            );
        } else if state.watch.ready_sent && !status.paused {
            set_paused(state, true);
        }
        return;
    }
    let buffering = status.buffering.is_some();
    if buffering != state.watch.buffering_sent {
        state.watch.buffering_sent = buffering;
        dispatch(
            state,
            json!({"type": "watchTogetherBufferingChanged", "buffering": buffering, "nowMs": now}),
        );
    }
    let position_ms = (status.position * 1000.0) as i64;
    if is_host {
        let due = match state.watch.last_report {
            None => true,
            Some((at, paused, position, rate)) => {
                let expected = if paused {
                    position
                } else {
                    position + at.elapsed().as_secs_f64() * rate
                };
                at.elapsed() >= HOST_REPORT
                    || paused != status.paused
                    || (rate - speed).abs() > 0.01
                    || (status.position - expected).abs() > POSITION_JUMP
            }
        };
        if due {
            state.watch.last_report = Some((Instant::now(), status.paused, status.position, speed));
            dispatch(
                state,
                json!({
                    "type": "watchTogetherPlaybackReported",
                    "positionMs": position_ms,
                    "durationMs": (status.duration * 1000.0) as i64,
                    "playing": !status.paused,
                    "buffering": buffering,
                    "rate": speed,
                    "nowMs": now,
                }),
            );
        }
    } else if state
        .watch
        .last_sample
        .is_none_or(|at| at.elapsed() >= SAMPLE)
        && state
            .watch
            .settle_until
            .is_none_or(|until| Instant::now() >= until)
        && !buffering
    {
        state.watch.last_sample = Some(Instant::now());
        dispatch(
            state,
            json!({
                "type": "watchTogetherPositionSampled",
                "positionMs": position_ms,
                "nowMs": now,
                "speedCorrectionActive": state.watch.correcting,
            }),
        );
    }
}


pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    match node {
        fluxa_ui::NODE_SETTINGS_WATCH_PARTY => open_modal(state),
        fluxa_ui::NODE_SETTINGS_WATCH_JOIN => {
            let code = std::mem::take(&mut state.settings.watch_code);
            if code.len() != 6 {
                state.settings.watch_code = code;
                return true;
            }
            handle(state, fluxa_ui::WatchTogetherRequest::Join(code));
        }
        fluxa_ui::NODE_SETTINGS_WATCH_LEAVE => {
            handle(state, fluxa_ui::WatchTogetherRequest::Leave)
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
            let (name, profile_id, avatar) = identity(state);
            dispatch(
                state,
                json!({
                    "type": "watchTogetherCreateRequested",
                    "displayName": name,
                    "profileId": profile_id,
                    "avatar": avatar,
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
        Request::Leave => dispatch(state, json!({"type": "watchTogetherLeaveRequested"})),
        Request::CopyCode => {
            state.copied = state
                .watch_view
                .as_ref()
                .and_then(|view| view.room_code.clone());
        }
    }
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

fn refresh_view(state: &mut RendererState, watch: &Value) {
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
    view.members = watch["members"]
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
        .collect();
}
