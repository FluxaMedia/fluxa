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

mod executor;
#[cfg(not(target_arch = "wasm32"))]
mod fetch_text;
#[cfg(not(target_arch = "wasm32"))]
mod range_proxy;
mod session;
pub mod storage;

pub use executor::{
    EffectCompletion, EffectExecutor, core_value, media_servers, remove_media_server,
};
#[cfg(not(target_arch = "wasm32"))]
pub use fetch_text::fetch_text;
#[cfg(not(target_arch = "wasm32"))]
pub use range_proxy::range_proxy;
pub use session::{SessionHandle, persisted_runtime_state};
pub use storage::Storage;
