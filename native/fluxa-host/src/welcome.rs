use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use fluxa_effects::Storage;
use fluxa_ui::{UiFormFactorJson, WelcomeModel, WelcomeQr, WelcomeRequest};
use serde_json::{Value, json};
use web_time::Instant;

use crate::{RendererState, core_value, host_log, profile_language, profiles};

type Pending = Receiver<Result<Value, String>>;

const RETRY: Duration = Duration::from_secs(5);
const WAIT: Duration = Duration::from_millis(250);
const OFFLINE_KEY: &str = "welcome_offline";

#[derive(Default)]
pub(crate) struct Flow {
    sign_in: Option<Pending>,
    start: Option<Pending>,
    poll: Option<Pending>,
    pairing: Option<Pairing>,
    retry_at: Option<Instant>,
}

struct Pairing {
    code: String,
    nonce: String,
    expires_at: i64,
    interval: Duration,
    next_poll: Instant,
}

pub(crate) fn required(storage: &Storage) -> bool {
    let offline = profiles::read(storage, OFFLINE_KEY)
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    core_value(
        "welcomeRequired",
        json!({"profiles": profiles::stored_profiles(storage), "offlineChosen": offline}),
    )
    .and_then(|value| value.as_bool())
    .unwrap_or(false)
}

pub(crate) fn open(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    state.welcome = Some(WelcomeModel {
        language: profile_language(&session.active_profile()),
        ..WelcomeModel::default()
    });
    state.welcome_flow = Some(Flow::default());
}

pub(crate) fn handle(state: &mut RendererState, request: WelcomeRequest) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    match request {
        WelcomeRequest::Offline => {
            profiles::write(session.storage(), OFFLINE_KEY, &json!(true));
            state.welcome = None;
            state.welcome_flow = None;
        }
        WelcomeRequest::SignIn => {
            let (Some(model), Some(flow)) = (state.welcome.as_mut(), state.welcome_flow.as_mut())
            else {
                return;
            };
            let email = model.email.trim().to_owned();
            if email.is_empty() || model.password.is_empty() {
                model.error = Some("auth.error.fill_required".to_owned());
                return;
            }
            model.busy = true;
            model.error = None;
            flow.sign_in = Some(
                session
                    .executor()
                    .nuvio_sign_in(email, model.password.clone()),
            );
        }
    }
}

fn received(slot: &mut Option<Pending>) -> Option<Result<Value, String>> {
    let outcome = match slot.as_ref()?.try_recv() {
        Ok(outcome) => outcome,
        Err(TryRecvError::Empty) => return None,
        Err(TryRecvError::Disconnected) => Err("auth.error.unknown".to_owned()),
    };
    *slot = None;
    Some(outcome)
}

fn qr(url: &str) -> Option<WelcomeQr> {
    let code = qrcode::QrCode::new(url.as_bytes()).ok()?;
    Some(WelcomeQr {
        width: code.width(),
        modules: code
            .to_colors()
            .into_iter()
            .map(|color| color == qrcode::Color::Dark)
            .collect(),
    })
}

pub(crate) fn poll(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let (Some(model), Some(flow)) = (state.welcome.as_mut(), state.welcome_flow.as_mut()) else {
        return;
    };
    let executor = session.executor();
    let now = Instant::now();
    let mut signed_in = None;

    if let Some(outcome) = received(&mut flow.sign_in) {
        model.busy = false;
        match outcome {
            Ok(profile) => signed_in = Some(profile),
            Err(key) => model.error = Some(key),
        }
    }

    if state.home.form_factor == UiFormFactorJson::Tv && signed_in.is_none() {
        if let Some(outcome) = received(&mut flow.start) {
            match outcome {
                Ok(started) => {
                    let text = |key: &str| {
                        started
                            .get(key)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned()
                    };
                    model.qr = qr(&text("webUrl"));
                    let shown = text("verificationUri")
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .to_owned();
                    model.pair_code =
                        Some((shown, text("userCode"))).filter(|(_, code)| !code.is_empty());
                    model.error = None;
                    let interval = started.get("interval").and_then(Value::as_u64).unwrap_or(3);
                    flow.pairing = Some(Pairing {
                        code: text("code"),
                        nonce: text("nonce"),
                        expires_at: started
                            .get("expiresAt")
                            .and_then(Value::as_i64)
                            .unwrap_or_default(),
                        interval: Duration::from_secs(interval),
                        next_poll: now + Duration::from_secs(interval),
                    });
                }
                Err(key) => {
                    model.error = Some(key);
                    flow.retry_at = Some(now + RETRY);
                }
            }
        }
        if let Some(outcome) = received(&mut flow.poll) {
            match outcome {
                Ok(result) if result.get("state").and_then(Value::as_str) == Some("success") => {
                    signed_in = result.get("profile").cloned();
                }
                Ok(result) if result.get("state").and_then(Value::as_str) == Some("ended") => {
                    model.qr = None;
                    model.pair_code = None;
                    flow.pairing = None;
                }
                Ok(_) => {}
                Err(key) => {
                    model.error = Some(key);
                    model.qr = None;
                    model.pair_code = None;
                    flow.pairing = None;
                    flow.retry_at = Some(now + RETRY);
                }
            }
        }
        let expired = flow
            .pairing
            .as_ref()
            .is_some_and(|pairing| pairing.expires_at <= chrono::Utc::now().timestamp());
        if expired {
            flow.pairing = None;
            model.qr = None;
            model.pair_code = None;
        }
        if signed_in.is_none() {
            if flow.pairing.is_none()
                && flow.start.is_none()
                && flow.retry_at.is_none_or(|at| at <= now)
            {
                flow.retry_at = None;
                flow.start = Some(executor.nuvio_tv_start());
            }
            if let Some(pairing) = flow.pairing.as_mut()
                && flow.poll.is_none()
                && pairing.next_poll <= now
            {
                pairing.next_poll = now + pairing.interval;
                flow.poll =
                    Some(executor.nuvio_tv_poll(pairing.code.clone(), pairing.nonce.clone()));
            }
        }
    }

    if let Some(profile) = signed_in {
        finish(state, &profile);
    }
}

pub(crate) fn schedule(state: &mut RendererState) {
    let Some(flow) = state.welcome_flow.as_ref() else {
        return;
    };
    let now = Instant::now();
    let waiting = flow.sign_in.is_some() || flow.start.is_some() || flow.poll.is_some();
    let due = flow
        .pairing
        .as_ref()
        .map(|pairing| pairing.next_poll)
        .into_iter()
        .chain(flow.retry_at)
        .chain(waiting.then_some(now + WAIT))
        .min();
    if let Some(due) = due {
        state.redraw_at = Some(state.redraw_at.map_or(due, |at| at.min(due)));
    }
}

fn finish(state: &mut RendererState, profile: &Value) {
    let Some(id) = profile.get("id").and_then(Value::as_str) else {
        host_log("Nuvio sign-in returned a profile without an id".to_owned());
        return;
    };
    let Some(storage) = state
        .session
        .as_ref()
        .map(|session| session.storage().clone())
    else {
        return;
    };
    state.welcome = None;
    state.welcome_flow = None;
    if profiles::stored_profiles(&storage).len() > 1 {
        profiles::open(state);
    } else {
        profiles::activate(state, &storage, id);
    }
}
