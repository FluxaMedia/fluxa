use std::sync::{Arc, mpsc::Receiver};
use std::time::Duration;

use fluxa_core::player::stream_policy;
use fluxa_ui::{PlayerModel, SettingsModel};
use serde_json::{Value, json};
use web_time::Instant;

use crate::{
    NativeAction, RendererState, Route, SessionHandle, core_value, host_log, profile_language,
};

mod backend;
mod casting;
mod filmstrip;
mod frame;
mod input;
mod lifecycle;
mod logos;
mod menu;
mod mini;
mod open;
mod options;
mod overlay;
mod pump;
mod queue;
mod resolve;
mod seek_previews;
mod session;
mod shuffle;
mod sources;
mod stats;
mod submit;
mod subtitle_sync;
mod trailer;
mod trickplay;
mod upnext;
pub use backend::*;
pub(crate) use frame::*;
pub(crate) use input::*;
pub(crate) use lifecycle::*;
pub(crate) use logos::*;
pub(crate) use mini::*;
pub(crate) use open::*;
pub(crate) use options::*;
use overlay::Overlay;
pub(crate) use pump::*;
pub(crate) use queue::*;
use resolve::*;
pub(crate) use session::*;
pub(crate) use shuffle::*;
pub(crate) use sources::*;
pub use stats::PlaybackStats;
use upnext::*;

const CONTROLS_TIMEOUT: Duration = Duration::from_secs(3);
const DEFAULT_SEEK_STEP: f64 = 10.0;
const SCRUB_COMMIT: Duration = Duration::from_millis(1800);
const TORRENT_UNRESPONSIVE_AFTER: Duration = Duration::from_secs(20);
const MIN_BRIGHTNESS: f32 = 0.15;
const MEDIA_DRIFT: f64 = 1.5;
const FAST_SPEED: f64 = 2.0;
const SEEK_RUN: Duration = Duration::from_millis(1400);
const TOAST_FADE: Duration = Duration::from_millis(250);
const THUMBNAIL_INTERVAL: Duration = Duration::from_millis(75);
