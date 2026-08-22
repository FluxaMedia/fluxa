use std::collections::HashMap;

pub(super) fn fetch(
    client: &reqwest::blocking::Client,
    url: &str,
    stream_headers: &HashMap<String, String>,
    request_headers: &HashMap<String, String>,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let mut request = client.get(url);
    for (key, value) in stream_headers {
        request = request.header(key, value);
    }
    if let Some(range) = request_headers.get("range") {
        request = request.header("Range", range);
    }
    request.send()
}

pub(super) fn rewrite_manifest(manifest: &str, base_url: &str, proxy_segment_base: &str) -> String {
    manifest
        .lines()
        .map(|line| {
            if line.is_empty() {
                return line.to_string();
            }
            if line.starts_with('#') {
                rewrite_uri_attributes(&rewrite_p7_codecs(line), base_url, proxy_segment_base)
            } else {
                let absolute_url = resolve_url(base_url, line);
                format!("{}{}", proxy_segment_base, percent_encode(&absolute_url))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn percent_decode(value: &str) -> String {
    let mut output = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = char::from(bytes[index + 1]).to_digit(16);
            let low = char::from(bytes[index + 2]).to_digit(16);
            if let (Some(high), Some(low)) = (high, low) {
                output.push(((high << 4) | low) as u8);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn rewrite_p7_codecs(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("dvhe.07") && !lower.contains("dvh1.07") {
        return line.to_string();
    }
    let mut output = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let lower_bytes = lower.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if lower_bytes[index..].starts_with(b"dvhe.07") {
            output.push_str("dvhe.08");
            index += 7;
        } else if lower_bytes[index..].starts_with(b"dvh1.07") {
            output.push_str("dvh1.08");
            index += 7;
        } else {
            let character = line[index..].chars().next().unwrap();
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

fn rewrite_uri_attributes(line: &str, base_url: &str, proxy_segment_base: &str) -> String {
    if !line.to_ascii_lowercase().contains("uri=\"") {
        return line.to_string();
    }
    let mut output = String::with_capacity(line.len() + 128);
    let mut remaining = line;
    loop {
        let lower = remaining.to_ascii_lowercase();
        let Some(position) = lower.find("uri=\"") else {
            output.push_str(remaining);
            break;
        };
        output.push_str(&remaining[..position]);
        let after_uri = &remaining[position + 5..];
        if let Some(end) = after_uri.find('"') {
            let absolute_url = resolve_url(base_url, &after_uri[..end]);
            output.push_str("URI=\"");
            output.push_str(proxy_segment_base);
            output.push_str(&percent_encode(&absolute_url));
            output.push('"');
            remaining = &after_uri[end + 1..];
        } else {
            output.push_str(&remaining[position..]);
            break;
        }
    }
    output
}

fn resolve_url(base_url: &str, relative: &str) -> String {
    let relative_lower = relative.to_ascii_lowercase();
    if relative_lower.starts_with("http://") || relative_lower.starts_with("https://") {
        return relative.to_string();
    }
    if relative.starts_with('/')
        && let Some(scheme_end) = base_url.find("://")
    {
        let remainder = &base_url[scheme_end + 3..];
        let authority_end = remainder.find('/').unwrap_or(remainder.len());
        return format!(
            "{}{}",
            &base_url[..scheme_end + 3 + authority_end],
            relative
        );
    }
    let base_directory = base_url
        .rfind('/')
        .map(|position| &base_url[..position + 1])
        .unwrap_or(base_url);
    normalize_url_path(format!("{}{}", base_directory, relative))
}

fn normalize_url_path(url: String) -> String {
    let path_start = if let Some(position) = url.find("://") {
        let remainder = &url[position + 3..];
        position + 3 + remainder.find('/').unwrap_or(remainder.len())
    } else {
        return url;
    };
    let (prefix, path) = url.split_at(path_start);
    let mut parts = Vec::new();
    for segment in path.split('/') {
        match segment {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            value => parts.push(value),
        }
    }
    format!(
        "{}/{}{}",
        prefix,
        parts.join("/"),
        if path.ends_with('/') { "/" } else { "" }
    )
}

fn percent_encode(value: &str) -> String {
    let mut output = String::with_capacity(value.len() * 3);
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~'
            | b':'
            | b'/'
            | b'?'
            | b'#'
            | b'@'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'=' => output.push(byte as char),
            value => output.push_str(&format!("%{value:02X}")),
        }
    }
    output
}
