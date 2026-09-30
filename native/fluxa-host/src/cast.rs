use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use fluxa_ui::{PanelRow, PlayerPanel, localized};

#[cfg(not(target_arch = "wasm32"))]
mod net;

#[cfg(target_arch = "wasm32")]
mod net {
    use std::sync::mpsc::{Receiver, Sender};

    use super::{Cmd, Device, Event};

    pub(super) fn discover(_: Sender<Device>) {}

    pub(super) fn lan_url(url: &str) -> String {
        url.to_owned()
    }

    pub(super) fn start(
        _: Device,
        _: String,
        _: String,
        _: f64,
        events: Sender<Event>,
        _: Receiver<Cmd>,
    ) {
        let _ = events.send(Event::Failed(
            "casting is not available in the browser".to_owned(),
        ));
    }
}

const SEEK_STEP: f64 = 10.0;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Protocol {
    Dlna,
    Chromecast,
    Fcast,
    Roku,
    Airplay,
}

impl Protocol {
    fn label(self) -> &'static str {
        match self {
            Protocol::Dlna => "DLNA",
            Protocol::Chromecast => "Chromecast",
            Protocol::Fcast => "FCast",
            Protocol::Roku => "Roku",
            Protocol::Airplay => "AirPlay",
        }
    }
}

#[derive(Clone)]
pub(crate) struct Device {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) protocol: Protocol,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) control_url: Option<String>,
}

pub(crate) enum Cmd {
    Pause,
    Resume,
    Seek(f64),
    Stop,
}

pub(crate) enum Event {
    Ready,
    Failed(String),
    Position { seconds: f64, playing: bool },
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Row {
    Toggle,
    Back,
    Forward,
    Stop,
    Device(usize),
}

struct Active {
    name: String,
    ready: bool,
    playing: bool,
    position: f64,
    since: Instant,
}

#[derive(Default)]
pub(crate) struct Cast {
    devices: Vec<Device>,
    discovery: Option<Receiver<Device>>,
    scan_until: Option<Instant>,
    events: Option<Receiver<Event>>,
    control: Option<Sender<Cmd>>,
    active: Option<Active>,
    error: Option<String>,
}

impl Cast {
    pub(crate) fn scan(&mut self) {
        if self.scan_until.is_some_and(|until| until > Instant::now()) {
            return;
        }
        let (tx, rx) = channel();
        self.devices.clear();
        self.discovery = Some(rx);
        self.scan_until = Some(Instant::now() + Duration::from_secs(5));
        net::discover(tx);
    }

    fn scanning(&self) -> bool {
        self.scan_until.is_some_and(|until| until > Instant::now())
    }

    pub(crate) fn poll(&mut self) {
        if let Some(discovery) = &self.discovery {
            while let Ok(device) = discovery.try_recv() {
                if !self.devices.iter().any(|known| known.id == device.id) {
                    self.devices.push(device);
                }
            }
        }
        let Some(events) = &self.events else {
            return;
        };
        while let Ok(event) = events.try_recv() {
            match event {
                Event::Ready => {
                    if let Some(active) = self.active.as_mut() {
                        active.ready = true;
                        active.since = Instant::now();
                    }
                }
                Event::Failed(error) => {
                    self.error = Some(error);
                    if self.active.as_ref().is_some_and(|active| !active.ready) {
                        self.active = None;
                        self.control = None;
                    }
                }
                Event::Position { seconds, playing } => {
                    if let Some(active) = self.active.as_mut() {
                        active.position = seconds;
                        active.playing = playing;
                        active.since = Instant::now();
                    }
                }
            }
        }
    }

    pub(crate) fn start(&mut self, device: Device, media_url: &str, title: &str, resume: f64) {
        self.stop();
        let (events_tx, events_rx) = channel();
        let (control_tx, control_rx) = channel();
        self.error = None;
        self.events = Some(events_rx);
        self.control = Some(control_tx);
        self.active = Some(Active {
            name: device.name.clone(),
            ready: false,
            playing: true,
            position: resume,
            since: Instant::now(),
        });
        net::start(
            device,
            net::lan_url(media_url),
            title.to_owned(),
            resume,
            events_tx,
            control_rx,
        );
    }

    pub(crate) fn stop(&mut self) {
        if let Some(control) = self.control.take() {
            let _ = control.send(Cmd::Stop);
        }
        self.active = None;
    }

    fn position(&self) -> f64 {
        match &self.active {
            Some(active) if active.playing => {
                active.position + active.since.elapsed().as_secs_f64()
            }
            Some(active) => active.position,
            None => 0.0,
        }
    }

    fn send(&mut self, command: Cmd) {
        if let Some(control) = &self.control {
            let _ = control.send(command);
        }
    }

    pub(crate) fn toggle(&mut self) {
        let position = self.position();
        let Some(active) = self.active.as_mut() else {
            return;
        };
        let playing = !active.playing;
        active.position = position;
        active.since = Instant::now();
        active.playing = playing;
        self.send(if playing { Cmd::Resume } else { Cmd::Pause });
    }

    fn seek_by(&mut self, delta: f64) {
        let target = (self.position() + delta).max(0.0);
        if let Some(active) = self.active.as_mut() {
            active.position = target;
            active.since = Instant::now();
        }
        self.send(Cmd::Seek(target));
    }

    pub(crate) fn rows(&self) -> Vec<Row> {
        if self.active.is_some() {
            return vec![Row::Toggle, Row::Back, Row::Forward, Row::Stop];
        }
        (0..self.devices.len()).map(Row::Device).collect()
    }

    pub(crate) fn device(&self, index: usize) -> Option<Device> {
        self.devices.get(index).cloned()
    }

    pub(crate) fn apply(&mut self, row: Row) {
        match row {
            Row::Toggle => self.toggle(),
            Row::Back => self.seek_by(-SEEK_STEP),
            Row::Forward => self.seek_by(SEEK_STEP),
            Row::Stop => self.stop(),
            Row::Device(_) => {}
        }
    }

    pub(crate) fn model(&self, language: &str) -> PlayerPanel {
        let text = |key: &str| localized(key, language);
        let message = match (&self.active, &self.error) {
            (_, Some(error)) => Some(error.clone()),
            (Some(active), None) if !active.ready => Some(text("player.cast_connecting")),
            (Some(active), None) => Some(text("player.casting_to").replace("%s", &active.name)),
            (None, None) if self.scanning() => Some(text("player.cast_searching")),
            (None, None) if self.devices.is_empty() => Some(text("player.cast_no_devices")),
            (None, None) => None,
        };
        let rows = self
            .rows()
            .into_iter()
            .map(|row| match row {
                Row::Toggle => PanelRow {
                    label: text(
                        if self.active.as_ref().is_some_and(|active| active.playing) {
                            "player.pause"
                        } else {
                            "player.play"
                        },
                    ),
                    value: None,
                    primary: true,
                },
                Row::Back => PanelRow {
                    label: format!("-{SEEK_STEP:.0}s"),
                    value: None,
                    primary: false,
                },
                Row::Forward => PanelRow {
                    label: format!("+{SEEK_STEP:.0}s"),
                    value: None,
                    primary: false,
                },
                Row::Stop => PanelRow {
                    label: text("player.stop_casting"),
                    value: None,
                    primary: false,
                },
                Row::Device(index) => PanelRow {
                    label: self.devices[index].name.clone(),
                    value: Some(self.devices[index].protocol.label().to_owned()),
                    primary: false,
                },
            })
            .collect();
        PlayerPanel {
            title: text("player.cast"),
            message,
            rows,
        }
    }
}
