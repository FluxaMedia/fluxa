use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::diagnostics::{Report, trace_dump, trace_try_dump};

static CRASH_FILE: OnceLock<PathBuf> = OnceLock::new();

const FATAL: [libc::c_int; 5] = [libc::SIGSEGV, libc::SIGBUS, libc::SIGILL, libc::SIGFPE, libc::SIGABRT];

pub fn install(dsn: &str, release: &str) -> sentry::ClientInitGuard {
    let mut options = sentry::ClientOptions::default();
    options.release = Some(release.to_owned().into());
    options.attach_stacktrace = true;
    let guard = sentry::init((dsn, options));
    let report = Report::collect();
    sentry::configure_scope(|scope| {
        for (key, value) in &report.tags {
            scope.set_tag(key, value);
        }
        for (key, value) in &report.extras {
            scope.set_extra(key, value.clone().into());
        }
    });
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        sentry::configure_scope(|scope| scope.set_extra("starfish.trace", trace_dump().into()));
        previous(info);
    }));
    guard
}

pub fn capture(message: &str) {
    sentry::with_scope(
        |scope| scope.set_extra("starfish.trace", trace_dump().into()),
        || sentry::capture_message(message, sentry::Level::Error),
    );
}

pub fn install_default(release: &str) -> Option<sentry::ClientInitGuard> {
    option_env!("FLUXA_SENTRY_DSN")
        .filter(|dsn| !dsn.is_empty())
        .map(|dsn| install(dsn, release))
}

pub fn report_compatibility(consent: bool, marker: &std::path::Path, release: &str) -> bool {
    if !consent || crate::diagnostics::already_reported(marker, release) {
        return false;
    }
    let sent = sentry::Hub::current().client().is_some();
    if sent {
        sentry::with_scope(
            |scope| scope.set_tag("report.kind", "compatibility"),
            || sentry::capture_message("webOS compatibility report", sentry::Level::Info),
        );
        crate::diagnostics::mark_reported(marker, release);
    }
    sent
}

extern "C" fn on_fatal(signal: libc::c_int) {
    if let Some(path) = CRASH_FILE.get() {
        let mut body = format!("{signal}\n");
        for line in trace_try_dump() {
            body.push_str(&line);
            body.push('\n');
        }
        let _ = std::fs::write(path, body);
    }
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
}

pub fn install_signal_handlers(path: &Path) {
    if CRASH_FILE.set(path.to_owned()).is_err() {
        return;
    }
    for signal in FATAL {
        unsafe {
            libc::signal(signal, on_fatal as extern "C" fn(libc::c_int) as libc::sighandler_t);
        }
    }
}

pub fn report_previous_crash(path: &Path) -> bool {
    let Ok(body) = std::fs::read_to_string(path) else {
        return false;
    };
    let _ = std::fs::remove_file(path);
    let mut lines = body.lines();
    let signal = lines.next().unwrap_or("?").to_owned();
    let trace: Vec<String> = lines.map(str::to_owned).collect();
    sentry::with_scope(
        |scope| {
            scope.set_tag("crash.signal", &signal);
            scope.set_extra("starfish.trace", trace.clone().into());
        },
        || sentry::capture_message(&format!("native crash in previous run (signal {signal})"), sentry::Level::Fatal),
    );
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(None);
    }
    true
}
