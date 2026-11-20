use super::*;

#[derive(Clone, Debug, Default)]
pub struct WelcomeQr {
    pub modules: Vec<bool>,
    pub width: usize,
}

#[derive(Clone, Debug, Default)]
pub struct WelcomeModel {
    pub language: String,
    pub email: String,
    pub password: String,
    pub show_password: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub qr: Option<WelcomeQr>,
    pub pair_code: Option<(String, String)>,
    pub pairing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeRequest {
    SignIn,
    Pair,
    CancelPair,
    Offline,
}

pub fn welcome_data(model: &WelcomeModel) -> serde_json::Value {
    let language = model.language.as_str();
    serde_json::json!({
        "busy": model.busy,
        "error": model.error.as_deref().map(|key| localized(key, language)),
        "signInKey": if model.busy { "auth.nuvio.signing_in" } else { "auth.log_in" },
        "pairing": model.pairing,
        "url": model.pair_code.as_ref().map(|(url, _)| url.as_str()),
        "code": model.pair_code.as_ref().map(|(_, code)| code.as_str()),
        "qr": model.qr.as_ref().map(|qr| serde_json::json!({
            "modules": qr.modules.iter().map(|dark| if *dark { '1' } else { '0' }).collect::<String>(),
            "count": qr.width,
        })),
    })
}
