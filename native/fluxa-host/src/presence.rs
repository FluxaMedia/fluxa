use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq)]
pub(crate) enum Presence {
    Off,
    Browsing(String),
    Viewing {
        title: String,
        poster: Option<String>,
    },
    Playing {
        title: String,
        detail: Option<String>,
        paused: bool,
        position: f64,
        duration: f64,
        poster: Option<String>,
    },
}

impl Presence {
    fn key(&self) -> String {
        match self {
            Presence::Playing {
                title,
                detail,
                paused,
                duration,
                ..
            } => format!("play|{title}|{detail:?}|{paused}|{}", *duration as i64),
            Presence::Viewing { title, .. } => format!("view|{title}"),
            Presence::Browsing(label) => format!("browse|{label}"),
            Presence::Off => "off".to_owned(),
        }
    }
}

struct Tracker {
    tx: Sender<Presence>,
    last_key: String,
    last_position: f64,
    last_sent: Instant,
}

static TRACKER: OnceLock<std::sync::Mutex<Tracker>> = OnceLock::new();

pub(crate) fn update(presence: Presence) {
    let tracker = TRACKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("fluxa-discord".to_owned())
            .spawn(move || worker(rx))
            .ok();
        std::sync::Mutex::new(Tracker {
            tx,
            last_key: String::new(),
            last_position: 0.0,
            last_sent: Instant::now(),
        })
    });
    let Ok(mut tracker) = tracker.lock() else {
        return;
    };
    let key = presence.key();
    let position = match &presence {
        Presence::Playing { position, .. } => *position,
        _ => 0.0,
    };
    let elapsed = tracker.last_sent.elapsed().as_secs_f64();
    let drifted = (position - (tracker.last_position + elapsed)).abs() > 5.0;
    let seeked = matches!(&presence, Presence::Playing { paused: false, .. }) && drifted;
    if key == tracker.last_key && !seeked {
        return;
    }
    tracker.last_key = key;
    tracker.last_position = position;
    tracker.last_sent = Instant::now();
    let _ = tracker.tx.send(presence);
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn worker(rx: Receiver<Presence>) {
    use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};

    const APP_ID: &str = "1518004842860122174";
    let mut client: Option<DiscordIpcClient> = None;
    let mut current = Presence::Off;
    let mut last_attempt: Option<Instant> = None;
    loop {
        match rx.recv_timeout(Duration::from_secs(15)) {
            Ok(next) => current = next,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if current == Presence::Off {
            if let Some(mut old) = client.take() {
                let _ = old.clear_activity();
                let _ = old.close();
            }
            continue;
        }
        if client.is_none() && last_attempt.is_none_or(|at| at.elapsed() >= Duration::from_secs(15)) {
            last_attempt = Some(Instant::now());
            let mut fresh = DiscordIpcClient::new(APP_ID);
            if fresh.connect().is_ok() {
                client = Some(fresh);
            }
        }
        let Some(ipc) = client.as_mut() else {
            continue;
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_secs() as i64)
            .unwrap_or_default();
        let result = match &current {
            Presence::Off => Ok(()),
            Presence::Browsing(label) => ipc.set_activity(
                activity::Activity::new()
                    .state(label)
                    .assets(activity::Assets::new().large_image("logo").large_text("Fluxa")),
            ),
            Presence::Viewing { title, poster } => ipc.set_activity(
                activity::Activity::new()
                    .details(title)
                    .state("Viewing details")
                    .assets(
                        activity::Assets::new()
                            .large_image(poster.as_deref().unwrap_or("logo"))
                            .large_text(title),
                    ),
            ),
            Presence::Playing {
                title,
                detail,
                paused,
                position,
                duration,
                poster,
            } => {
                if *paused {
                    let _ = ipc.clear_activity();
                }
                let details = format!("{title} · {}", if *paused { "Paused" } else { "Watching" });
                let mut act = activity::Activity::new()
                    .name("on Fluxa")
                    .details(&details)
                    .activity_type(activity::ActivityType::Watching)
                    .status_display_type(activity::StatusDisplayType::Details)
                    .assets(
                        activity::Assets::new()
                            .large_image(poster.as_deref().unwrap_or("logo"))
                            .large_text(title),
                    );
                if let Some(detail) = detail {
                    act = act.state(detail);
                }
                if !*paused && *duration > 0.0 {
                    let start = now - *position as i64;
                    act = act.timestamps(
                        activity::Timestamps::new()
                            .start(start)
                            .end(start + *duration as i64),
                    );
                }
                ipc.set_activity(act)
            }
        };
        if result.is_err() {
            client = None;
            last_attempt = None;
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn worker(rx: Receiver<Presence>) {
    while rx.recv().is_ok() {}
}
