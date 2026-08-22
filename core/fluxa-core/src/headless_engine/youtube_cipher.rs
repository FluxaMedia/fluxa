use regex::Regex;
use rquickjs::{Context, Runtime};
use std::time::{Duration, Instant};

pub(super) fn decipher_signature(player_js: &str, signature: &str) -> Option<String> {
    let function = signature_function(player_js)?;
    execute_transform(player_js, &function, signature)
}

pub(super) fn decipher_url(cipher: &str, player_js: &str) -> Option<String> {
    let values = query_pairs(cipher);
    let url = values.iter().find(|(key, _)| key == "url")?.1.clone();
    let signature = values.iter().find(|(key, _)| key == "s")?.1.clone();
    let parameter = values
        .iter()
        .find(|(key, _)| key == "sp")
        .map(|(_, value)| value.as_str())
        .unwrap_or("signature");
    let resolved = decipher_signature(player_js, &signature)?;
    let separator = if url.contains('?') { '&' } else { '?' };
    decipher_n_parameter(
        &format!(
            "{}{}{}={}",
            url,
            separator,
            parameter,
            encode_component(&resolved)
        ),
        player_js,
    )
}

pub(super) fn resolve_url(url: &str, player_js: Option<&str>) -> Option<String> {
    if !has_query_parameter(url, "n") {
        return Some(url.to_string());
    }
    player_js.map_or_else(
        || Some(url.to_string()),
        |script| decipher_n_parameter(url, script),
    )
}

fn decipher_n_parameter(url: &str, player_js: &str) -> Option<String> {
    let Some(n) = query_parameter(url, "n") else {
        return Some(url.to_string());
    };
    let Some(function) = n_function(player_js) else {
        return Some(url.to_string());
    };
    let resolved = execute_transform(player_js, &function, &n)?;
    replace_query_parameter(url, "n", &resolved)
}

fn query_pairs(value: &str) -> Vec<(String, String)> {
    value
        .split('&')
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            Some((decode_component(key)?, decode_component(value)?))
        })
        .collect()
}

fn decode_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index) == Some(&b'%') && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            output.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            output.push(if bytes.get(index) == Some(&b'+') {
                b' '
            } else {
                *bytes.get(index)?
            });
            index += 1;
        }
    }
    String::from_utf8(output).ok()
}

fn encode_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                vec![byte as char].into_iter().collect::<Vec<_>>()
            } else {
                format!("%{:02X}", byte).chars().collect()
            }
        })
        .collect()
}

fn signature_function(player_js: &str) -> Option<String> {
    transform_function(
        player_js,
        &[
            r#"(?:\bsignature\b|\bsig\b)\s*:\s*([\w$]+)"#,
            r#"\.set\([^,]+,\s*([\w$]+)\("#,
        ],
    )
    .or_else(|| function_literal_candidate(player_js))
}

fn n_function(player_js: &str) -> Option<String> {
    transform_function(
        player_js,
        &[
            r#"(?:\bn\b|\"n\")\s*[:=]\s*([\w$]+)"#,
            r#"(?:get\(\"n\"\)|get\('n'\))[^;]{0,200}?=\s*([\w$]+)\("#,
            r#"([\w$]+)\([^)]*\)[^;]{0,120}?(?:\bn\b|\"n\")"#,
        ],
    )
}

fn function_literal_candidate(player_js: &str) -> Option<String> {
    let pattern = Regex::new(r#"(?s)(?:function\s+([\w$]+)|([\w$]+)\s*=\s*function)\s*\([^)]*\)\s*\{.*?\.split\(\"\"\).*?\.join\(\"\"\)"#).ok()?;
    let captures = pattern.captures(player_js)?;
    captures
        .get(1)
        .or_else(|| captures.get(2))
        .map(|value| value.as_str().to_string())
}

fn transform_function(player_js: &str, patterns: &[&str]) -> Option<String> {
    for expression in patterns {
        let pattern = Regex::new(expression).ok()?;
        for captures in pattern.captures_iter(player_js) {
            let Some(name) = captures.get(1).map(|value| value.as_str()) else {
                continue;
            };
            let Some(source) = extract_function(player_js, name) else {
                continue;
            };
            if source.contains(".split(\"\")") && source.contains(".join(\"\")") {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn execute_transform(player_js: &str, function_name: &str, input: &str) -> Option<String> {
    let function_source = extract_function(player_js, function_name)?;
    let helper_sources = helper_objects(player_js, &function_source);
    let escaped_input = serde_json::to_string(input).ok()?;
    let script = format!(
        "{};{};{}({})",
        helper_sources.join(";"),
        function_source,
        function_name,
        escaped_input
    );
    let runtime = Runtime::new().ok()?;
    runtime.set_memory_limit(8 * 1024 * 1024);
    let deadline = Instant::now() + Duration::from_millis(100);
    runtime.set_interrupt_handler(Some(Box::new(move || Instant::now() > deadline)));
    let context = Context::full(&runtime).ok()?;
    context.with(|ctx| ctx.eval::<String, _>(script).ok())
}

fn has_query_parameter(url: &str, name: &str) -> bool {
    query_parameter(url, name).is_some()
}

fn query_parameter(url: &str, name: &str) -> Option<String> {
    url.split_once('?')?
        .1
        .split('#')
        .next()?
        .split('&')
        .find_map(|entry| {
            let (key, value) = entry.split_once('=')?;
            (decode_component(key).as_deref() == Some(name))
                .then(|| decode_component(value))
                .flatten()
        })
}

fn replace_query_parameter(url: &str, name: &str, replacement: &str) -> Option<String> {
    let (before_query, query_and_fragment) = url.split_once('?')?;
    let (query, fragment) = query_and_fragment
        .split_once('#')
        .map(|(query, fragment)| (query, Some(fragment)))
        .unwrap_or((query_and_fragment, None));
    let mut replaced = false;
    let query = query
        .split('&')
        .map(|entry| {
            let Some((key, _value)) = entry.split_once('=') else {
                return entry.to_string();
            };
            if decode_component(key).as_deref() != Some(name) {
                return entry.to_string();
            }
            replaced = true;
            format!("{}={}", key, encode_component(replacement))
        })
        .collect::<Vec<_>>()
        .join("&");
    replaced.then(|| match fragment {
        Some(fragment) => format!("{before_query}?{query}#{fragment}"),
        None => format!("{before_query}?{query}"),
    })
}

fn extract_function(source: &str, name: &str) -> Option<String> {
    let pattern = Regex::new(&format!(
        r#"(?:function\s+{}\s*\(|{}\s*=\s*function\s*\()"#,
        regex::escape(name),
        regex::escape(name)
    ))
    .ok()?;
    let start = pattern.find(source)?.start();
    let body_start = source[start..].find('{')? + start;
    let body_end = balanced_end(source, body_start, '{', '}')?;
    let result = source[start..=body_end].to_string();
    if result.starts_with("function ") {
        return Some(format!("var {}={}", name, result));
    }
    Some(format!("var {}", result))
}

fn helper_objects(source: &str, function_source: &str) -> Vec<String> {
    let Some(pattern) = Regex::new(r#"\b([\w$]+)\.([\w$]+)\("#).ok() else {
        return Vec::new();
    };
    pattern
        .captures_iter(function_source)
        .filter_map(|capture| {
            let name = capture.get(1)?.as_str();
            let marker = format!("{}={{", name);
            let start = source.find(&marker)?;
            let body_start = start + marker.len() - 1;
            let end = balanced_end(source, body_start, '{', '}')?;
            Some(format!("var {}", &source[start..=end]))
        })
        .collect()
}

fn balanced_end(source: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0_i32;
    let mut quote = None;
    let mut escaped = false;
    for (offset, character) in source[start..].char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '\"' | '`') {
            quote = Some(character);
            continue;
        }
        if character == open {
            depth += 1;
        }
        if character == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_player_signature_transform_with_helper_object() {
        let player = r#"var XY={rv:function(a){a.reverse()},sw:function(a,b){var c=a[0];a[0]=a[b%a.length];a[b]=c}};sig=function(a){a=a.split("");XY.sw(a,2);XY.rv(a);return a.join("")}"#;
        assert_eq!(signature_function(player), Some("sig".to_string()));
        assert_eq!(
            extract_function(player, "sig"),
            Some(
                "var sig=function(a){a=a.split(\"\");XY.sw(a,2);XY.rv(a);return a.join(\"\")}"
                    .to_string()
            )
        );
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();
        let result = context.with(|ctx| ctx.eval::<String, _>("var XY={rv:function(a){a.reverse()},sw:function(a,b){var c=a[0];a[0]=a[b%a.length];a[b]=c}};var sig=function(a){a=a.split(\"\");XY.sw(a,2);XY.rv(a);return a.join(\"\")};sig(\"abcdef\")"));
        assert_eq!(result.unwrap(), "fedabc");
        assert_eq!(
            decipher_signature(player, "abcdef"),
            Some("fedabc".to_string())
        );
        assert_eq!(
            decipher_url(
                "url=https%3A%2F%2Fvideo.example%2Fplay%3Fn%3D1&s=abcdef&sp=sig",
                player
            ),
            Some("https://video.example/play?n=1&sig=fedabc".to_string())
        );
    }

    #[test]
    fn resolves_n_parameter_with_the_player_transform() {
        let player = r#"var XY={rv:function(a){a.reverse()}};nfunc=function(a){a=a.split("");XY.rv(a);return a.join("")};var route={n:nfunc}"#;
        assert_eq!(
            resolve_url(
                "https://video.example/play?foo=1&n=abcdef#fragment",
                Some(player)
            ),
            Some("https://video.example/play?foo=1&n=fedcba#fragment".to_string())
        );
    }
}
