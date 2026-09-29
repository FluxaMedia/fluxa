use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use fluxa_host::FluxaHost;
use serde_json::Value as Json;
use zbus::blocking::{Connection, connection};
use zbus::interface;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

const PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const DRIFT_SECONDS: f64 = 1.5;

struct State {
    status: &'static str,
    title: String,
    subtitle: String,
    poster: String,
    position_us: i64,
    duration_us: i64,
    rate: f64,
    can_next: bool,
    can_seek: bool,
    at: Instant,
}

impl State {
    fn position(&self) -> i64 {
        if self.status == "Playing" {
            self.position_us + (self.at.elapsed().as_secs_f64() * self.rate * 1e6) as i64
        } else {
            self.position_us
        }
    }

    fn metadata(&self) -> HashMap<String, OwnedValue> {
        let mut map = HashMap::new();
        if self.status == "Stopped" {
            return map;
        }
        let mut put = |key: &str, value: Value<'_>| {
            if let Ok(value) = value.try_to_owned() {
                map.insert(key.to_owned(), value);
            }
        };
        put(
            "mpris:trackid",
            Value::ObjectPath(ObjectPath::from_static_str_unchecked(
                "/org/fluxa/playback/current",
            )),
        );
        put("mpris:length", Value::I64(self.duration_us));
        put("xesam:title", Value::from(self.title.clone()));
        if !self.subtitle.is_empty() {
            put("xesam:artist", Value::from(vec![self.subtitle.clone()]));
        }
        if !self.poster.is_empty() {
            put("mpris:artUrl", Value::from(self.poster.clone()));
        }
        map
    }
}

struct Root;

#[interface(name = "org.mpris.MediaPlayer2")]
impl Root {
    fn raise(&self) {}

    fn quit(&self) {}

    #[zbus(property)]
    fn can_quit(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn identity(&self) -> String {
        "Fluxa".to_owned()
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        Vec::new()
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        Vec::new()
    }
}

struct Player {
    host: FluxaHost,
    state: Arc<Mutex<State>>,
}

impl Player {
    fn send(&self, command: &str, value: f64) {
        self.host.media_command(command, value);
    }

    fn read<T>(&self, read: impl FnOnce(&State) -> T) -> T {
        read(&self.state.lock().unwrap_or_else(|error| error.into_inner()))
    }
}

#[interface(name = "org.mpris.MediaPlayer2.Player")]
impl Player {
    fn next(&self) {
        self.send("next", 0.0);
    }

    fn previous(&self) {
        self.send("previous", 0.0);
    }

    fn pause(&self) {
        self.send("pause", 0.0);
    }

    fn play_pause(&self) {
        self.send("toggle", 0.0);
    }

    fn stop(&self) {
        self.send("stop", 0.0);
    }

    fn play(&self) {
        self.send("play", 0.0);
    }

    fn seek(&self, offset: i64) {
        self.send("seekToPosition", offset as f64 / 1e6);
    }

    fn set_position(&self, _track: ObjectPath<'_>, position: i64) {
        self.send("seekTo", position as f64 / 1e6);
    }

    fn open_uri(&self, _uri: String) {}

    #[zbus(property)]
    fn playback_status(&self) -> String {
        self.read(|state| state.status.to_owned())
    }

    #[zbus(property)]
    fn rate(&self) -> f64 {
        self.read(|state| state.rate)
    }

    #[zbus(property)]
    fn metadata(&self) -> HashMap<String, OwnedValue> {
        self.read(State::metadata)
    }

    #[zbus(property)]
    fn volume(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    fn position(&self) -> i64 {
        self.read(State::position)
    }

    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        self.read(|state| state.can_next)
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_play(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_seek(&self) -> bool {
        self.read(|state| state.can_seek)
    }

    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }
}

struct Mpris {
    connection: Connection,
    state: Arc<Mutex<State>>,
}

static MPRIS: OnceLock<Mpris> = OnceLock::new();

pub fn start(host: FluxaHost) {
    let state = Arc::new(Mutex::new(State {
        status: "Stopped",
        title: String::new(),
        subtitle: String::new(),
        poster: String::new(),
        position_us: 0,
        duration_us: 0,
        rate: 1.0,
        can_next: false,
        can_seek: false,
        at: Instant::now(),
    }));
    let player = Player {
        host,
        state: state.clone(),
    };
    let built = connection::Builder::session()
        .and_then(|builder| builder.name("org.mpris.MediaPlayer2.fluxa"))
        .and_then(|builder| builder.serve_at(PATH, Root))
        .and_then(|builder| builder.serve_at(PATH, player))
        .and_then(|builder| builder.build());
    match built {
        Ok(connection) => {
            let _ = MPRIS.set(Mpris { connection, state });
        }
        Err(error) => eprintln!("[fluxa-desktop] media controls unavailable: {error}"),
    }
}

pub fn publish(plan: &Json) {
    let Some(mpris) = MPRIS.get() else {
        return;
    };
    let text = |key: &str| plan[key].as_str().unwrap_or_default().to_owned();
    let actions: Vec<&str> = plan["actions"]
        .as_array()
        .map(|list| list.iter().filter_map(Json::as_str).collect())
        .unwrap_or_default();
    let status = match plan["state"].as_str() {
        Some("playing") => "Playing",
        _ => "Paused",
    };
    let position_us = plan["positionMs"].as_i64().unwrap_or(0) * 1000;
    let jumped = {
        let mut state = mpris
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let expected = state.position();
        let jumped = state.status != "Stopped"
            && ((expected - position_us).abs() as f64) > DRIFT_SECONDS * 1e6;
        *state = State {
            status,
            title: text("title"),
            subtitle: text("subtitle"),
            poster: text("poster"),
            position_us,
            duration_us: plan["durationMs"].as_i64().unwrap_or(0) * 1000,
            rate: plan["speed"].as_f64().unwrap_or(1.0),
            can_next: actions.contains(&"next"),
            can_seek: actions.contains(&"seekTo"),
            at: Instant::now(),
        };
        jumped
    };
    changed(mpris);
    if jumped {
        let _ = mpris
            .connection
            .emit_signal(None::<&str>, PATH, PLAYER, "Seeked", &(position_us,));
    }
}

pub fn clear() {
    let Some(mpris) = MPRIS.get() else {
        return;
    };
    {
        let mut state = mpris
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if state.status == "Stopped" {
            return;
        }
        state.status = "Stopped";
        state.can_next = false;
        state.can_seek = false;
    }
    changed(mpris);
}

fn changed(mpris: &Mpris) {
    let (status, rate, metadata, can_next, can_seek) = {
        let state = mpris
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        (
            state.status,
            state.rate,
            state.metadata(),
            state.can_next,
            state.can_seek,
        )
    };
    let mut props: HashMap<&str, Value<'_>> = HashMap::new();
    props.insert("PlaybackStatus", Value::from(status));
    props.insert("Rate", Value::F64(rate));
    props.insert("CanGoNext", Value::Bool(can_next));
    props.insert("CanSeek", Value::Bool(can_seek));
    props.insert("Metadata", Value::from(metadata));
    let _ = mpris.connection.emit_signal(
        None::<&str>,
        PATH,
        "org.freedesktop.DBus.Properties",
        "PropertiesChanged",
        &(PLAYER, props, Vec::<&str>::new()),
    );
}
