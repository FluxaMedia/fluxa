use std::sync::OnceLock;

static LOGGER: OnceLock<fn(&str)> = OnceLock::new();

pub fn set_logger(logger: fn(&str)) {
    let _ = LOGGER.set(logger);
}

#[doc(hidden)]
pub fn write_log(message: String) {
    match LOGGER.get() {
        Some(logger) => logger(&message),
        None => eprintln!("{message}"),
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::write_log(format!($($arg)*))
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
#[cfg(not(target_arch = "wasm32"))]
pub use range_proxy::range_proxy;
pub use session::{SessionHandle, persisted_runtime_state};
pub use storage::Storage;
