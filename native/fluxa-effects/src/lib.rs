use std::sync::OnceLock;

static LOGGER: OnceLock<fn(&str)> = OnceLock::new();
static REPORTER: OnceLock<fn(&str, &str)> = OnceLock::new();

pub fn set_reporter(reporter: fn(&str, &str)) {
    let _ = REPORTER.set(reporter);
}

#[doc(hidden)]
pub fn write_report(template: &str, message: String) {
    if let Some(reporter) = REPORTER.get() {
        reporter(template, &message);
    }
    write_log(message);
}

pub fn set_logger(logger: fn(&str)) {
    let _ = LOGGER.set(logger);
    #[cfg(not(target_arch = "wasm32"))]
    {
        static BRIDGE: LogBridge = LogBridge;
        if log::set_logger(&BRIDGE).is_ok() {
            log::set_max_level(log::LevelFilter::Info);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct LogBridge;

#[cfg(not(target_arch = "wasm32"))]
impl log::Log for LogBridge {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        let text = record.args().to_string();
        if text.starts_with("[plugin]") || record.target().starts_with("fluxa_mpv") {
            write_log(format!("[fluxa-native] {text}"));
        }
    }

    fn flush(&self) {}
}

#[doc(hidden)]
pub fn write_log(message: String) {
    match LOGGER.get() {
        Some(logger) => logger(&message),
        None => eprintln!("{message}"),
    }
}

pub fn flow(label: impl FnOnce() -> String) {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ENABLED.get_or_init(|| std::env::var("FLUXA_FLOW_TRACE").is_ok_and(|value| value != "0")) {
        write_log(format!("[flow] {}", label()));
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::write_log(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! report {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::write_report($fmt, format!($fmt $(, $arg)*))
    };
}

pub(crate) struct Stopwatch(#[cfg(not(target_arch = "wasm32"))] std::time::Instant);

impl Stopwatch {
    pub(crate) fn start() -> Self {
        Self(
            #[cfg(not(target_arch = "wasm32"))]
            std::time::Instant::now(),
        )
    }

    pub(crate) fn ms(&self) -> u128 {
        #[cfg(not(target_arch = "wasm32"))]
        return self.0.elapsed().as_millis();
        #[cfg(target_arch = "wasm32")]
        0
    }
}

pub(crate) fn url_label(url: &str) -> String {
    let base = url.split(['?', '#']).next().unwrap_or(url);
    let Some((scheme, rest)) = base.split_once("://") else {
        return base.to_owned();
    };
    match rest.split_once('/') {
        Some((host, path)) => {
            let last = path.rsplit('/').next().unwrap_or_default();
            if path.contains('/') {
                format!("{scheme}://{host}/…/{last}")
            } else {
                format!("{scheme}://{host}/{last}")
            }
        }
        None => format!("{scheme}://{rest}"),
    }
}

mod addon_health;
#[cfg(not(target_arch = "wasm32"))]
mod challenge;
mod executor;
#[cfg(not(target_arch = "wasm32"))]
mod fetch_text;
#[cfg(not(target_arch = "wasm32"))]
mod range_proxy;
mod session;
pub mod storage;

pub use addon_health::unreachable_addon_hosts;
#[cfg(not(target_arch = "wasm32"))]
pub use challenge::{
    complete_challenge, next_challenge_request, set_challenge_solver_available, set_stream_headers,
};
pub use executor::ActionSink;
#[cfg(not(target_arch = "wasm32"))]
pub use executor::cached_stream_headers;
pub use executor::{
    EffectCompletion, EffectExecutor, NUVIO_OFFLINE, core_value, media_servers,
    remove_media_server, set_simkl_rewatch_intent, set_youtube_proxy, simkl_plan_lacks_rewatch,
};
#[cfg(not(target_arch = "wasm32"))]
pub use fetch_text::fetch_text;
pub use fluxa_app::SliceGenerations;
#[cfg(not(target_arch = "wasm32"))]
pub use range_proxy::range_proxy;
pub use session::{SessionHandle, SnapshotView, persisted_runtime_state};
pub use storage::Storage;
