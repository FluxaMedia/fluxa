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
    level: sentry::Level,
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
        if !variant.is_empty() {
            sentry::configure_scope(|scope| scope.set_tag("variant", variant));
        }
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
    #[cfg(target_os = "android")]
    {
        options.transport = Some(Arc::new(|options: &sentry::ClientOptions| {
            Arc::new(android_transport(options)) as Arc<dyn sentry::Transport>
        }));
    }
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

#[cfg(target_os = "android")]
#[allow(deprecated)]
fn android_transport(options: &sentry::ClientOptions) -> sentry::transports::ReqwestHttpTransport {
    let roots = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .filter_map(|der| reqwest::Certificate::from_der(der).ok());
    let client = reqwest::Client::builder()
        .tls_certs_only(roots)
        .build()
        .unwrap_or_default();
    sentry::transports::ReqwestHttpTransport::with_client(options, client)
}

fn send(event: Pending) {
    sentry::with_scope(
        |scope| {
            for (key, value) in &event.tags {
                scope.set_tag(key, value);
            }
            scope.set_extra("recent_log", event.log.clone().into());
        },
        || sentry::capture_message(&event.message, event.level),
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
    submit(sentry::Level::Error, message, tags);
}

pub fn fatal(message: &str, tags: &[(&str, &str)]) {
    if sentry::Hub::current().client().is_none() {
        return;
    }
    submit(sentry::Level::Fatal, message, tags);
    flush();
}

fn submit(level: sentry::Level, message: &str, tags: &[(&str, &str)]) {
    let event = Pending {
        level,
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
