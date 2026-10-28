mod scrub;
#[cfg(unix)]
mod signals;

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod logbook;

pub use logbook::Level;

const LOG_LINES: usize = 4000;
const PENDING_LIMIT: usize = 8;

const DSN: Option<&str> = option_env!("FLUXA_SENTRY_DSN");

static CLIENT: Mutex<Option<sentry::ClientInitGuard>> = Mutex::new(None);
static LOG: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static PENDING: Mutex<Vec<Pending>> = Mutex::new(Vec::new());

struct Pending {
    level: sentry::Level,
    message: String,
    tags: Vec<(String, String)>,
    extras: Vec<(String, String)>,
    files: Vec<(String, Vec<u8>)>,
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
    let guard = init_client(dsn, variant);
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

fn init_client(dsn: &str, variant: &str) -> sentry::ClientInitGuard {
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
    guard
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
            for (key, value) in &event.extras {
                scope.set_extra(key, value.clone().into());
            }
            for (name, buffer) in &event.files {
                scope.add_attachment(sentry::protocol::Attachment {
                    buffer: buffer.clone(),
                    filename: name.clone(),
                    content_type: Some("text/plain".to_owned()),
                    ty: None,
                });
            }
        },
        || sentry::capture_message(&event.message, event.level),
    );
}

pub fn log(line: &str) {
    record(Level::Info, "host", line);
}

pub fn record(level: Level, target: &str, message: &str) {
    let line = logbook::format(level, target, &scrub::line(message));
    logbook::append(&line);
    if let Ok(mut lines) = LOG.lock() {
        if lines.len() == LOG_LINES {
            lines.pop_front();
        }
        lines.push_back(line);
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::record($crate::Level::Debug, module_path!(), &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::record($crate::Level::Info, module_path!(), &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::record($crate::Level::Warn, module_path!(), &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::record($crate::Level::Error, module_path!(), &format!($($arg)*))
    };
}

pub fn snapshot() -> String {
    LOG.lock()
        .map(|lines| lines.iter().cloned().collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

pub fn reporting_available() -> bool {
    DSN.is_some_and(|dsn| !dsn.is_empty())
}

pub fn open_session_log(data_dir: &Path) {
    let backlog = LOG
        .lock()
        .map(|lines| lines.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    logbook::open(data_dir, &backlog);
}

pub fn report_problem(title: &str, body: &str, tags: &[(&str, &str)]) -> bool {
    let Some(dsn) = DSN.filter(|dsn| !dsn.is_empty()) else {
        return false;
    };
    info!("user submitted a problem report: {title}");
    let event = logbook::pending(
        sentry::Level::Info,
        title,
        tags,
        &[("report_body", body)],
        LOG.lock()
            .map(|lines| lines.iter().cloned().collect())
            .unwrap_or_default(),
    );
    if sentry::Hub::current().client().is_some() {
        send(event);
        flush();
    } else {
        let guard = init_client(dsn, "");
        send(event);
        guard.flush(Some(Duration::from_secs(5)));
    }
    true
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
    let event = logbook::pending(
        level,
        message,
        tags,
        &[],
        LOG.lock()
            .map(|lines| lines.iter().cloned().collect())
            .unwrap_or_default(),
    );
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
