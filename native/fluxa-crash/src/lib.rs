mod scrub;
#[cfg(unix)]
mod signals;

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const LOG_LINES: usize = 80;
const PENDING_LIMIT: usize = 8;

const DSN: Option<&str> = option_env!("FLUXA_SENTRY_DSN");

static CLIENT: Mutex<Option<sentry::ClientInitGuard>> = Mutex::new(None);
static LOG: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static PENDING: Mutex<Vec<Pending>> = Mutex::new(Vec::new());

struct Pending {
    message: String,
    tags: Vec<(String, String)>,
    log: Vec<String>,
}

pub fn configure(enabled: bool, data_dir: &Path, variant: &str) {
    let Ok(mut client) = CLIENT.lock() else {
        return;
    };
    if !enabled {
        *client = None;
        if let Ok(mut pending) = PENDING.lock() {
            pending.clear();
        }
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
    drop(client);
    let queued = PENDING
        .lock()
        .map(|mut pending| std::mem::take(&mut *pending))
        .unwrap_or_default();
    for event in queued {
        send(event);
    }
}

fn send(event: Pending) {
    sentry::with_scope(
        |scope| {
            for (key, value) in &event.tags {
                scope.set_tag(key, value);
            }
            scope.set_extra("recent_log", event.log.clone().into());
        },
        || sentry::capture_message(&event.message, sentry::Level::Error),
    );
}

pub fn log(line: &str) {
    if let Ok(mut lines) = LOG.lock() {
        if lines.len() == LOG_LINES {
            lines.pop_front();
        }
        lines.push_back(line.to_owned());
    }
}

pub fn capture(message: &str, tags: &[(&str, &str)]) {
    let event = Pending {
        message: message.to_owned(),
        tags: tags
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect(),
        log: LOG
            .lock()
            .map(|lines| lines.iter().cloned().collect())
            .unwrap_or_default(),
    };
    if sentry::Hub::current().client().is_some() {
        send(event);
    } else if let Ok(mut pending) = PENDING.lock()
        && pending.len() < PENDING_LIMIT
    {
        pending.push(event);
    }
}

pub fn flush() {
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(Duration::from_secs(2)));
    }
}
