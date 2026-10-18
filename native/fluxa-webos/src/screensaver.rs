use std::time::Duration;

use serde_json::{Value, json};

use crate::luna::{self, Subscription};

const REGISTER: &str = "luna://com.webos.service.tvpower/power/registerScreenSaverRequest";
const RESPOND: &str = "luna://com.webos.service.tvpower/power/responseScreenSaverRequest";

pub struct Inhibitor {
    _subscription: Subscription,
}

fn refusal(line: &str, client: &str) -> Option<String> {
    let message: Value = serde_json::from_str(line).ok()?;
    (message["state"] == "Active").then(|| {
        json!({ "clientName": client, "ack": false, "timestamp": message["timestamp"] }).to_string()
    })
}

pub fn inhibit(client: &str) -> Option<Inhibitor> {
    let request = json!({ "subscribe": true, "clientName": client }).to_string();
    let (subscription, lines) = luna::subscribe(REGISTER, &request)?;
    let client = client.to_owned();
    std::thread::spawn(move || {
        for line in lines {
            if let Some(answer) = refusal(&line, &client) {
                luna::call(RESPOND, &answer, Duration::from_secs(3));
            }
        }
    });
    Some(Inhibitor { _subscription: subscription })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_state_is_refused_with_the_same_timestamp() {
        let answer = refusal(r#"{"returnValue":true,"state":"Active","timestamp":"123"}"#, "app").unwrap();
        let answer: Value = serde_json::from_str(&answer).unwrap();
        assert_eq!(answer["ack"], false);
        assert_eq!(answer["timestamp"], "123");
        assert_eq!(answer["clientName"], "app");
    }

    #[test]
    fn registration_acks_and_garbage_are_ignored() {
        assert!(refusal(r#"{"returnValue":true,"subscribed":true}"#, "app").is_none());
        assert!(refusal("not json", "app").is_none());
    }
}
