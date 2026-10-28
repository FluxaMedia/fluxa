use sentry::protocol::{Event, Value};

fn text(input: &str) -> String {
    input
        .split_inclusive(char::is_whitespace)
        .map(|word| {
            let body = word.trim_end();
            if body.starts_with("http://") || body.starts_with("https://") {
                let end = body.find(['?', '#']).unwrap_or(body.len());
                format!("{}{}", &body[..end], &word[body.len()..])
            } else {
                word.to_owned()
            }
        })
        .collect()
}

pub(crate) fn line(input: &str) -> String {
    let mut redact = false;
    input
        .split_inclusive(char::is_whitespace)
        .map(|word| {
            let body = word.trim_end();
            let lower = body.to_ascii_lowercase();
            let secret = std::mem::take(&mut redact);
            redact = lower == "bearer" || lower == "authorization:";
            if secret
                || ["token=", "api_key=", "apikey=", "password=", "secret="]
                    .iter()
                    .any(|key| lower.contains(key))
            {
                format!("[redacted]{}", &word[body.len()..])
            } else {
                text(word)
            }
        })
        .collect()
}

fn value(input: &mut Value) {
    match input {
        Value::String(string) => *string = text(string),
        Value::Array(items) => items.iter_mut().for_each(value),
        Value::Object(map) => map.values_mut().for_each(value),
        _ => {}
    }
}

pub(crate) fn event(mut event: Event<'static>) -> Event<'static> {
    event.server_name = None;
    event.user = None;
    if let Some(message) = event.message.as_mut() {
        *message = text(message);
    }
    for exception in event.exception.values.iter_mut() {
        if let Some(message) = exception.value.as_mut() {
            *message = text(message);
        }
    }
    event.extra.values_mut().for_each(value);
    event
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_and_fragment_are_dropped_from_urls() {
        assert_eq!(
            text("open https://cdn.example.com/a.mkv?token=abc#t=1 failed"),
            "open https://cdn.example.com/a.mkv failed"
        );
    }

    #[test]
    fn bearer_and_token_values_are_redacted() {
        assert_eq!(
            line("auth Bearer abc123 then token=xyz ok"),
            "auth Bearer [redacted] then [redacted] ok"
        );
    }

    #[test]
    fn plain_text_is_untouched() {
        assert_eq!(text("decoder lost sync\n"), "decoder lost sync\n");
    }
}
