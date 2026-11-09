use std::sync::mpsc;

use web_time::SystemTime;

use super::*;

const PROBE_AFTER: Duration = Duration::from_secs(3);
const WAKE_GAP: Duration = Duration::from_secs(5);
const PROBE_RETRY: Duration = Duration::from_secs(3);
const FALLBACK_PROBE: &str = "1.1.1.1:443";
const MISSES_TO_LOSE: u32 = 3;

pub(super) struct Recovery {
    last_tick: Instant,
    last_wall: SystemTime,
    trouble_since: Option<Instant>,
    net_lost: bool,
    misses: u32,
    woke: bool,
    last_good: f64,
    probe: Option<mpsc::Receiver<bool>>,
    probe_at: Instant,
}

impl Recovery {
    pub(super) fn stalled_for(&self) -> Duration {
        self.trouble_since
            .map_or(Duration::ZERO, |since| since.elapsed())
    }

    pub(super) fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            last_wall: SystemTime::now(),
            trouble_since: None,
            net_lost: false,
            misses: 0,
            woke: false,
            last_good: 0.0,
            probe: None,
            probe_at: Instant::now(),
        }
    }
}

pub(super) fn tick(player: &mut PlayerSession, snapshot: &Value) -> Option<f64> {
    let now = Instant::now();
    let wall = SystemTime::now();
    let mono = now.duration_since(player.recovery.last_tick);
    let suspended = wall
        .duration_since(player.recovery.last_wall)
        .unwrap_or_default()
        .saturating_sub(mono);
    player.recovery.last_tick = now;
    player.recovery.last_wall = wall;
    if player.local || player.trailer.is_some() || player.cast_target.is_some() {
        return None;
    }
    let status = &player.status;
    if status.has_frame && !status.stalled && status.position > 1.0 {
        player.recovery.last_good = status.position;
    }
    if player.loaded_url.is_none() {
        return None;
    }
    let slept = suspended >= WAKE_GAP && status.has_frame && !status.paused;
    let trouble = !status.has_frame || status.stalled || status.error.is_some();
    if !trouble && !slept {
        player.recovery.trouble_since = None;
        player.recovery.probe = None;
        player.recovery.net_lost = false;
        player.recovery.misses = 0;
        player.recovery.woke = false;
        return None;
    }
    player.recovery.woke |= slept;
    let since = *player.recovery.trouble_since.get_or_insert(now);
    if let Some(probe) = player.recovery.probe.as_ref() {
        return match probe.try_recv() {
            Ok(true) => {
                player.recovery.probe = None;
                player.recovery.misses = 0;
                let recover = player.recovery.net_lost
                    || player.recovery.woke
                    || player.status.error.is_some();
                if recover {
                    Some(resume_point(player))
                } else {
                    player.recovery.probe_at = now + PROBE_RETRY;
                    None
                }
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Ok(false) | Err(mpsc::TryRecvError::Disconnected) => {
                player.recovery.probe = None;
                player.recovery.misses += 1;
                player.recovery.net_lost |= player.recovery.misses >= MISSES_TO_LOSE;
                player.recovery.probe_at = now + PROBE_RETRY;
                None
            }
        };
    }
    if now < player.recovery.probe_at
        || (!player.recovery.woke && now.duration_since(since) < PROBE_AFTER)
    {
        return None;
    }
    let target = probe_target(player, snapshot).unwrap_or_else(|| FALLBACK_PROBE.to_owned());
    player.recovery.probe = Some(spawn_probe(target));
    None
}

fn resume_point(player: &PlayerSession) -> f64 {
    if player.status.has_frame && player.status.position > 1.0 {
        player.status.position
    } else {
        player
            .recovery
            .last_good
            .max(player.resume_at.unwrap_or(0.0))
    }
}

fn probe_target(player: &PlayerSession, snapshot: &Value) -> Option<String> {
    let stream = overlay::current_stream(player, snapshot)?;
    let url = stream
        .get("playableUrl")
        .or_else(|| stream.get("url"))
        .and_then(Value::as_str)?;
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?']).next()?;
    let authority = authority.rsplit('@').next()?;
    if authority.starts_with("127.") || authority.starts_with("localhost") {
        return None;
    }
    let port = match scheme {
        "https" => 443,
        "http" => 80,
        _ => return None,
    };
    Some(if authority.contains(':') {
        authority.to_owned()
    } else {
        format!("{authority}:{port}")
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_probe(target: String) -> mpsc::Receiver<bool> {
    use std::net::ToSocketAddrs;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reachable = [target.as_str(), FALLBACK_PROBE, "8.8.8.8:443"]
            .into_iter()
            .any(|host| {
                host.to_socket_addrs().is_ok_and(|mut addrs| {
                    addrs.any(|addr| {
                        std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(4)).is_ok()
                    })
                })
            });
        let _ = tx.send(reachable);
    });
    rx
}

#[cfg(target_arch = "wasm32")]
fn spawn_probe(_target: String) -> mpsc::Receiver<bool> {
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(true);
    rx
}

pub(super) fn restart(state: &mut RendererState, at: f64, snapshot: &Value) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let stream = overlay::current_stream(player, snapshot);
    let woke = player.recovery.woke;
    let mut item = player.meta.clone();
    if let Some(fields) = item.as_object_mut() {
        fields.remove("pickSource");
        fields.insert("timeOffset".to_owned(), json!(at as i64));
        fields.insert(
            "lastVideoId".to_owned(),
            snapshot
                .pointer("/player/currentVideoId")
                .cloned()
                .unwrap_or(Value::Null),
        );
        fields.insert(
            "lastStreamIndex".to_owned(),
            snapshot
                .pointer("/player/currentStreamIndex")
                .cloned()
                .unwrap_or(Value::Null),
        );
        fields.insert(
            "lastStreamUrl".to_owned(),
            stream
                .and_then(|stream| stream.get("playableUrl").or_else(|| stream.get("url")))
                .cloned()
                .unwrap_or(Value::Null),
        );
        fields.insert(
            "lastStream".to_owned(),
            stream.cloned().unwrap_or(Value::Null),
        );
        fields.insert("recovering".to_owned(), Value::Bool(true));
        fields.insert("startPaused".to_owned(), Value::Bool(woke));
        fields.insert(
            "lastStreamTitle".to_owned(),
            stream
                .and_then(|stream| stream.get("title").or_else(|| stream.get("name")))
                .cloned()
                .unwrap_or(Value::Null),
        );
    }
    host_log(format!("player restarting playback at {at:.1}s"));
    close(state);
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
}
