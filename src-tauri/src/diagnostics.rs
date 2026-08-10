use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

static POPUP_SHOWING: AtomicBool = AtomicBool::new(false);
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub(crate) fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

fn ui_is_turkish(app: &AppHandle) -> bool {
    let state = app.state::<crate::DesktopState>();
    crate::storage::read_pref_field(state, "language").as_deref() == Some("tr")
}

pub(crate) fn report(app: &AppHandle, message: String, level: sentry::Level) {
    report_with_scope(app, message, level, |_| {});
}

/// For call sites that don't have an `AppHandle` in scope (e.g. deep inside
/// the mpv event loop). Silently drops the report if the app hasn't finished
/// setup yet, same as the pre-ask-popup behavior for diagnostic-mode-off.
pub(crate) fn report_global_with_scope(
    message: String,
    level: sentry::Level,
    configure_scope: impl Fn(&mut sentry::Scope) + Send + Sync + 'static,
) {
    match APP_HANDLE.get() {
        Some(app) => report_with_scope(app, message, level, configure_scope),
        None if crate::DIAGNOSTIC_MODE.load(Ordering::Relaxed) => {
            sentry::with_scope(configure_scope, || sentry::capture_message(&message, level));
        }
        None => {}
    }
}

pub(crate) fn report_with_scope(
    app: &AppHandle,
    message: String,
    level: sentry::Level,
    configure_scope: impl Fn(&mut sentry::Scope) + Send + Sync + 'static,
) {
    if crate::DIAGNOSTIC_MODE.load(Ordering::Relaxed) {
        sentry::with_scope(configure_scope, || sentry::capture_message(&message, level));
        return;
    }
    if POPUP_SHOWING.swap(true, Ordering::AcqRel) {
        return;
    }
    let turkish = ui_is_turkish(app);
    let (title, body, send_label, dont_send_label) = if turkish {
        (
            "Bir sorun oluştu",
            format!("Fluxa'da teknik bir sorun oluştu. Anonim bir hata raporu gönderilsin mi?\n\n{message}"),
            "Rapor gönder",
            "Gönderme",
        )
    } else {
        (
            "Something went wrong",
            format!("Fluxa ran into a technical problem. Send an anonymous error report?\n\n{message}"),
            "Send report",
            "Don't send",
        )
    };
    app.dialog()
        .message(body)
        .title(title)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            send_label.to_string(),
            dont_send_label.to_string(),
        ))
        .show(move |send_report| {
            POPUP_SHOWING.store(false, Ordering::Release);
            if send_report {
                send_one_shot(&message, level, configure_scope);
            }
        });
}

fn send_one_shot(
    message: &str,
    level: sentry::Level,
    configure_scope: impl Fn(&mut sentry::Scope) + Send + Sync + 'static,
) {
    let guard = sentry::init(sentry::ClientOptions {
        dsn: crate::sentry_dsn(),
        release: sentry::release_name!(),
        ..Default::default()
    });
    sentry::with_scope(configure_scope, || sentry::capture_message(message, level));
    drop(guard);
}
