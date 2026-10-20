mod scrub;
#[cfg(unix)]
mod signals;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DSN: Option<&str> = option_env!("FLUXA_SENTRY_DSN");

static CLIENT: Mutex<Option<sentry::ClientInitGuard>> = Mutex::new(None);

pub fn configure(enabled: bool, data_dir: &Path, variant: &str) {
    let Ok(mut client) = CLIENT.lock() else {
        return;
    };
    if !enabled {
        *client = None;
        return;
    }
    if client.is_some() || sentry::Hub::current().client().is_some() {
        return;
    }
    let Some(dsn) = DSN.filter(|dsn| !dsn.is_empty()) else {
        return;
    };
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut options = sentry::ClientOptions::default();
    options.release = Some(env!("CARGO_PKG_VERSION").into());
    options.attach_stacktrace = true;
    options.send_default_pii = false;
    options.before_send = Some(Arc::new(|event| Some(scrub::event(event))));
    let guard = sentry::init((dsn, options));
    sentry::configure_scope(|scope| {
        scope.set_tag("os", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);
        if !variant.is_empty() {
            scope.set_tag("variant", variant);
        }
    });
    #[cfg(unix)]
    {
        signals::report_previous(data_dir);
        signals::arm(data_dir);
    }
    #[cfg(not(unix))]
    let _ = data_dir;
    *client = Some(guard);
}

pub fn flush() {
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(Duration::from_secs(2)));
    }
}
