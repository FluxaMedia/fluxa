use super::*;
use crate::player::VideoCommand;

const HOST_REPORT: Duration = Duration::from_secs(10);
const SAMPLE: Duration = Duration::from_millis(500);
const POSITION_JUMP: f64 = 2.0;

#[derive(Default)]
pub(crate) struct State {
    room: Option<String>,
    opened: Option<String>,
    ready_sent: bool,
    buffering_sent: bool,
    last_report: Option<(Instant, bool, f64)>,
    last_sample: Option<Instant>,
    correcting: bool,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}

fn set_paused(state: &mut RendererState, paused: bool) {
    let value = if paused { "yes" } else { "no" };
    player::command(
        state,
        VideoCommand::Mpv(vec!["set".to_owned(), "pause".to_owned(), value.to_owned()]),
    );
}

fn set_speed(state: &mut RendererState, rate: f64) {
    if let Some(video) = state.video.as_mut() {
        video.command(VideoCommand::SetSpeed(rate));
    }
}

fn dispatch(state: &RendererState, action: Value) {
    if let Some(session) = state.session.as_ref() {
        let _ = session.dispatch(action);
    }
}

pub(crate) fn tick(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let commands = session.take_watch_commands();
    let snapshot = session.snapshot();
    let watch = snapshot
        .get("watchTogether")
        .cloned()
        .unwrap_or(Value::Null);
    let room = watch["roomCode"].as_str().map(ToOwned::to_owned);
    if room != state.watch.room {
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
            player::close(state);
            let resolve = player::open_file(state, url, title);
            dispatch(state, resolve);
        }
        Some("start") => {
            let position = command["positionMs"].as_f64().unwrap_or_default() / 1000.0;
            player::command(state, VideoCommand::SeekTo(position));
            set_paused(state, false);
        }
        Some("remote") => {
            let position = command["positionMs"].as_f64().unwrap_or_default() / 1000.0;
            let playing = command["playing"].as_bool().unwrap_or(true);
            let local = state.player.as_ref().map(|player| player.status.position);
            if local.is_none_or(|local| (local - position).abs() > POSITION_JUMP) {
                player::command(state, VideoCommand::SeekTo(position));
            }
            set_paused(state, !playing);
        }
        Some("seek") => {
            let position = command["positionMs"].as_f64().unwrap_or_default() / 1000.0;
            player::command(state, VideoCommand::SeekTo(position));
        }
        Some("speed") => {
            let rate = command["value"].as_f64().unwrap_or(1.0);
            state.watch.correcting = true;
            set_speed(state, rate);
        }
        Some("resetSpeed") => {
            state.watch.correcting = false;
            set_speed(state, 1.0);
        }
        _ => {}
    }
}

fn open(state: &mut RendererState, content: &Value, action: NativeAction) {
    state.watch.opened = content["id"].as_str().map(ToOwned::to_owned);
    state.watch.ready_sent = false;
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
    let content = player.watch_content();
    if is_host
        && !playing
        && let Some(content) = content
        && state.watch.opened.as_deref() != content["id"].as_str()
    {
        state.watch.opened = content["id"].as_str().map(ToOwned::to_owned);
        dispatch(
            state,
            json!({"type": "watchTogetherContentSelected", "content": content, "startPositionMs": 0}),
        );
    }
    if state.watch.opened.is_none() {
        return;
    }
    let now = now_ms();
    if !playing {
        if status.has_frame && !state.watch.ready_sent {
            state.watch.ready_sent = true;
            set_paused(state, true);
            dispatch(
                state,
                json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": now}),
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
            Some((at, paused, position)) => {
                at.elapsed() >= HOST_REPORT
                    || paused != status.paused
                    || (status.position - position).abs() > POSITION_JUMP + HOST_REPORT_SLACK
            }
        };
        if due {
            state.watch.last_report = Some((Instant::now(), status.paused, status.position));
            dispatch(
                state,
                json!({
                    "type": "watchTogetherPlaybackReported",
                    "positionMs": position_ms,
                    "durationMs": (status.duration * 1000.0) as i64,
                    "playing": !status.paused,
                    "buffering": buffering,
                    "nowMs": now,
                }),
            );
        }
    } else if state
        .watch
        .last_sample
        .is_none_or(|at| at.elapsed() >= SAMPLE)
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

const HOST_REPORT_SLACK: f64 = 1.0;
