use serde_json::{Value, json};

use crate::service::HttpRequest;

const ENDPOINT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";
pub const MAX_TITLES: usize = 8;

pub fn request(key: &str, query: &str, anime: bool, movie: bool) -> HttpRequest {
    let kind = if anime {
        "anime series or films"
    } else if movie {
        "movies"
    } else {
        "TV series"
    };
    let prompt = format!(
        "List up to {MAX_TITLES} well-known {kind} that best match this search: \"{}\". \
         Answer with a JSON array of title strings only.",
        query.replace('"', "'")
    );
    HttpRequest {
        url: format!("{ENDPOINT}?key={key}"),
        post_body: Some(
            json!({
                "contents": [{"parts": [{"text": prompt}]}],
                "generationConfig": {"responseMimeType": "application/json"}
            })
            .to_string(),
        ),
        headers: vec![("Content-Type".into(), "application/json".into())],
    }
}

pub fn titles(body: &Value) -> Vec<String> {
    let text = body
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(Value::as_str)
        .unwrap_or("");
    serde_json::from_str::<Vec<String>>(text)
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(MAX_TITLES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_come_from_the_first_candidate() {
        let body = json!({"candidates": [{"content": {"parts": [
            {"text": "[\"Example One\", \" \", \"Example Two\"]"}
        ]}}]});
        assert_eq!(titles(&body), vec!["Example One", "Example Two"]);
    }

    #[test]
    fn malformed_answers_give_no_titles() {
        assert!(titles(&json!({"candidates": []})).is_empty());
        assert!(
            titles(&json!({"candidates": [{"content": {"parts": [{"text": "not json"}]}}]}))
                .is_empty()
        );
    }

    #[test]
    fn query_quotes_are_neutralised_in_the_prompt() {
        let req = request("k", "a \"b\"", false, true);
        assert!(req.post_body.unwrap().contains("a 'b'"));
    }
}
