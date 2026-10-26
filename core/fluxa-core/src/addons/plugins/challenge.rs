use std::collections::HashMap;

pub const CLEARANCE_COOKIE: &str = "cf_clearance";
pub const CLEARANCE_TTL_SECS: u64 = 25 * 60;

pub fn is_cloudflare_challenge(status: u16, headers: &HashMap<String, String>, body: &str) -> bool {
    if !matches!(status, 403 | 429 | 503) {
        return false;
    }
    let header = |name: &str| {
        headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.to_ascii_lowercase())
    };
    if header("cf-mitigated").is_some_and(|value| value.contains("challenge")) {
        return true;
    }
    let cloudflare = header("server").is_some_and(|value| value.contains("cloudflare"))
        || header("cf-ray").is_some();
    if !cloudflare {
        return false;
    }
    let head: String = body
        .chars()
        .take(16 * 1024)
        .collect::<String>()
        .to_lowercase();
    [
        "just a moment",
        "cf-chl",
        "challenge-platform",
        "cf_chl_opt",
        "attention required! | cloudflare",
        "checking your browser",
    ]
    .iter()
    .any(|marker| head.contains(marker))
}

pub fn merge_cookie_header(existing: Option<&str>, clearance: &[(String, String)]) -> String {
    let mut pairs: Vec<(String, String)> = existing
        .unwrap_or_default()
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            Some((name.trim().to_owned(), value.trim().to_owned()))
        })
        .collect();
    for (name, value) in clearance {
        match pairs.iter_mut().find(|(known, _)| known == name) {
            Some(pair) => pair.1 = value.clone(),
            None => pairs.push((name.clone(), value.clone())),
        }
    }
    pairs
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn has_clearance(cookies: &[(String, String)]) -> bool {
    cookies.iter().any(|(name, _)| name == CLEARANCE_COOKIE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn mitigated_header_marks_challenge() {
        let headers = headers(&[("Cf-Mitigated", "challenge")]);
        assert!(is_cloudflare_challenge(403, &headers, ""));
    }

    #[test]
    fn interstitial_body_needs_cloudflare_server() {
        let body = "<title>Just a moment...</title>";
        let cloudflare = headers(&[("server", "cloudflare")]);
        assert!(is_cloudflare_challenge(503, &cloudflare, body));
        assert!(!is_cloudflare_challenge(503, &HashMap::new(), body));
    }

    #[test]
    fn plain_forbidden_from_cloudflare_is_not_a_challenge() {
        let cloudflare = headers(&[("server", "cloudflare")]);
        assert!(!is_cloudflare_challenge(403, &cloudflare, "access denied"));
        assert!(!is_cloudflare_challenge(200, &cloudflare, "just a moment"));
    }

    #[test]
    fn clearance_replaces_stale_cookie_and_keeps_others() {
        let merged = merge_cookie_header(
            Some("a=1; cf_clearance=old"),
            &[("cf_clearance".to_owned(), "new".to_owned())],
        );
        assert_eq!(merged, "a=1; cf_clearance=new");
    }
}
