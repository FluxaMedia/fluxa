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
mod session;
pub mod storage;

pub use executor::{
    EffectCompletion, EffectExecutor, core_value, media_servers, remove_media_server,
};
pub use session::{SessionHandle, persisted_runtime_state};
pub use storage::Storage;
