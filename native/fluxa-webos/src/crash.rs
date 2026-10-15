use crate::diagnostics::{Report, trace_dump};

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
