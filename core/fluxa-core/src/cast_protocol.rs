use serde_json::json;

pub(crate) const AVTRANSPORT_URN: &str = "urn:schemas-upnp-org:service:AVTransport:1";
pub(crate) const RENDERING_CONTROL_URN: &str = "urn:schemas-upnp-org:service:RenderingControl:1";

pub(crate) fn validate_stream_url(url: &str) -> bool {
    let trimmed = url.trim();
    let Some(scheme_end) = trimmed.find("://") else {
        return false;
    };
    let scheme = trimmed[..scheme_end].to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return false;
    }
    !trimmed[scheme_end + 3..].is_empty()
}

pub(crate) fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

fn resolve_url(base_url: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let base = base_url.trim_end_matches('/');
    if path.starts_with('/') {
        if let Some(scheme_end) = base.find("://")
            && let Some(host_end) = base[scheme_end + 3..].find('/')
        {
            return format!("{}{}", &base[..scheme_end + 3 + host_end], path);
        }
        return format!("{base}{path}");
    }
    format!("{base}/{path}")
}

fn extract_service_control_url(xml: &str, base_url: &str, urn: &str) -> Option<String> {
    for service_block in xml.split("<service>").skip(1) {
        let service_type = extract_tag(service_block, "serviceType")?;
        if service_type != urn {
            continue;
        }
        let control_path = extract_tag(service_block, "controlURL")?;
        return Some(resolve_url(base_url, &control_path));
    }
    None
}

pub(crate) fn dlna_parse_device_description_json(xml: &str, base_url: &str) -> Option<String> {
    let name = extract_tag(xml, "friendlyName").unwrap_or_else(|| "Unknown device".to_string());
    let control_url = extract_service_control_url(xml, base_url, AVTRANSPORT_URN)?;
    let rendering_control_url = extract_service_control_url(xml, base_url, RENDERING_CONTROL_URN);
    Some(json!({"name": name, "controlUrl": control_url, "renderingControlUrl": rendering_control_url}).to_string())
}

pub(crate) fn soap_action_body(urn: &str, action: &str, args: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\
<s:Body><u:{action} xmlns:u=\"{urn}\">{args}</u:{action}></s:Body></s:Envelope>"
    )
}

fn format_didl_metadata(title: &str, subtitle_url: Option<&str>) -> String {
    let subtitle_res = subtitle_url
        .map(|url| {
            format!(
                "<res protocolInfo=\"http-get:*:text/srt:*\">{}</res>",
                xml_escape(url)
            )
        })
        .unwrap_or_default();
    let didl = format!(
        "<DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\">\
<item id=\"0\" parentID=\"-1\" restricted=\"1\"><dc:title>{}</dc:title><upnp:class>object.item.videoItem</upnp:class>{subtitle_res}</item></DIDL-Lite>",
        xml_escape(title)
    );
    xml_escape(&didl)
}

pub(crate) fn dlna_set_av_transport_args(
    media_url: &str,
    title: &str,
    subtitle_url: Option<&str>,
) -> Option<String> {
    if !validate_stream_url(media_url) {
        return None;
    }
    let metadata = format_didl_metadata(title, subtitle_url);
    Some(format!(
        "<InstanceID>0</InstanceID><CurrentURI>{}</CurrentURI><CurrentURIMetaData>{metadata}</CurrentURIMetaData>",
        xml_escape(media_url)
    ))
}

pub(crate) fn format_hms(total_secs: f64) -> String {
    let total = total_secs.max(0.0) as u64;
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}

pub(crate) fn dlna_seek_args(position_secs: f64) -> String {
    format!(
        "<InstanceID>0</InstanceID><Unit>ABS_TIME</Unit><Target>{}</Target>",
        format_hms(position_secs)
    )
}

pub(crate) fn dlna_set_volume_args(level: f64) -> String {
    let volume = (level.clamp(0.0, 1.0) * 100.0).round() as u32;
    format!(
        "<InstanceID>0</InstanceID><Channel>Master</Channel><DesiredVolume>{volume}</DesiredVolume>"
    )
}

pub(crate) fn resolve_loopback_url(stream_url: &str, lan_ip: &str) -> String {
    if let Some(rest) = stream_url.strip_prefix("http://127.0.0.1") {
        return format!("http://{lan_ip}{rest}");
    }
    stream_url.to_string()
}

pub(crate) fn guess_cast_content_type(media_url: &str) -> &'static str {
    let path = media_url
        .split(['?', '#'])
        .next()
        .unwrap_or(media_url)
        .to_ascii_lowercase();
    if path.ends_with(".m3u8") {
        "application/x-mpegurl"
    } else if path.ends_with(".mpd") {
        "application/dash+xml"
    } else if path.ends_with(".mkv") {
        "video/x-matroska"
    } else if path.ends_with(".webm") {
        "video/webm"
    } else {
        "video/mp4"
    }
}

fn write_varint(buf: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
}

fn write_tag(buf: &mut Vec<u8>, field: u32, wire_type: u8) {
    write_varint(buf, ((field << 3) | wire_type as u32) as u64);
}

fn write_string_field(buf: &mut Vec<u8>, field: u32, value: &str) {
    write_tag(buf, field, 2);
    write_varint(buf, value.len() as u64);
    buf.extend_from_slice(value.as_bytes());
}

pub(crate) fn encode_cast_message(
    source_id: &str,
    destination_id: &str,
    namespace: &str,
    payload_utf8: &str,
) -> Vec<u8> {
    let mut buf = Vec::new();
    write_tag(&mut buf, 1, 0);
    write_varint(&mut buf, 0);
    write_string_field(&mut buf, 2, source_id);
    write_string_field(&mut buf, 3, destination_id);
    write_string_field(&mut buf, 4, namespace);
    write_tag(&mut buf, 5, 0);
    write_varint(&mut buf, 0);
    write_string_field(&mut buf, 6, payload_utf8);
    buf
}

fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let byte = *buf.get(*pos)?;
        *pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    Some(result)
}

pub(crate) struct DecodedCastMessage {
    pub namespace: String,
    pub payload_utf8: String,
}

pub(crate) fn decode_cast_message(buf: &[u8]) -> Option<DecodedCastMessage> {
    let mut pos = 0;
    let mut namespace = String::new();
    let mut payload_utf8 = String::new();
    while pos < buf.len() {
        let tag = read_varint(buf, &mut pos)?;
        let field = (tag >> 3) as u32;
        let wire_type = (tag & 0x7) as u8;
        match wire_type {
            0 => {
                read_varint(buf, &mut pos)?;
            }
            2 => {
                let len = read_varint(buf, &mut pos)? as usize;
                let end = pos.checked_add(len)?;
                let slice = buf.get(pos..end)?;
                if field == 4 {
                    namespace = String::from_utf8_lossy(slice).to_string();
                } else if field == 6 {
                    payload_utf8 = String::from_utf8_lossy(slice).to_string();
                }
                pos = end;
            }
            _ => return None,
        }
    }
    Some(DecodedCastMessage {
        namespace,
        payload_utf8,
    })
}

fn roku_url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub(crate) fn roku_device_name(xml: &str) -> Option<String> {
    extract_tag(xml, "friendly-device-name")
}

pub(crate) fn roku_launch_url(
    host: &str,
    media_url: &str,
    subtitle_url: Option<&str>,
) -> Option<String> {
    if !validate_stream_url(media_url) {
        return None;
    }
    if let Some(sub) = subtitle_url
        && !validate_stream_url(sub)
    {
        return None;
    }
    const ROKU_MEDIA_PLAYER_APP_ID: &str = "2213";
    let mut url = format!(
        "http://{host}:8060/launch/{ROKU_MEDIA_PLAYER_APP_ID}?t=v&u={}",
        roku_url_encode(media_url)
    );
    if let Some(sub) = subtitle_url {
        url.push_str(&format!("&k={}", roku_url_encode(sub)));
    }
    Some(url)
}

pub(crate) fn airplay_volume_db(level: f64) -> f64 {
    if level <= 0.0 {
        -30.0
    } else {
        (20.0 * level.clamp(0.0, 1.0).log10()).max(-30.0)
    }
}

pub(crate) fn airplay_play_body(media_url: &str) -> Option<String> {
    if !validate_stream_url(media_url) {
        return None;
    }
    Some(format!(
        "Content-Location: {media_url}\nStart-Position: 0\n"
    ))
}

pub(crate) const FCAST_OP_PLAY: u8 = 1;
pub(crate) const FCAST_OP_PAUSE: u8 = 2;
pub(crate) const FCAST_OP_RESUME: u8 = 3;
pub(crate) const FCAST_OP_STOP: u8 = 4;
pub(crate) const FCAST_OP_SEEK: u8 = 5;
pub(crate) const FCAST_OP_PLAYBACK_UPDATE: u8 = 6;
pub(crate) const FCAST_OP_VOLUME_UPDATE: u8 = 7;
pub(crate) const FCAST_OP_SET_VOLUME: u8 = 8;
pub(crate) const FCAST_OP_PLAYBACK_ERROR: u8 = 9;
pub(crate) const FCAST_OP_SET_SPEED: u8 = 10;
pub(crate) const FCAST_OP_VERSION: u8 = 11;
pub(crate) const FCAST_OP_PING: u8 = 12;
pub(crate) const FCAST_OP_PONG: u8 = 13;

pub(crate) const FCAST_MAX_MESSAGE_BYTES: usize = 32 * 1024 - 4;

pub(crate) fn fcast_encode_message(opcode: u8, body_json: &str) -> Option<Vec<u8>> {
    if body_json.len() + 1 > FCAST_MAX_MESSAGE_BYTES {
        return None;
    }
    let mut message = Vec::with_capacity(body_json.len() + 1);
    message.push(opcode);
    message.extend_from_slice(body_json.as_bytes());
    Some(message)
}

pub(crate) fn fcast_decode_message(buf: &[u8]) -> Option<(u8, String)> {
    let (opcode, body) = buf.split_first()?;
    let body = if body.is_empty() {
        String::new()
    } else {
        String::from_utf8(body.to_vec()).ok()?
    };
    Some((*opcode, body))
}

pub(crate) fn fcast_play_body(media_url: &str, resume_position_secs: f64) -> Option<String> {
    if !validate_stream_url(media_url) {
        return None;
    }
    let mut body = json!({
        "container": guess_cast_content_type(media_url),
        "url": media_url,
    });
    if resume_position_secs > 0.0 {
        body["time"] = json!(resume_position_secs);
    }
    Some(body.to_string())
}

pub(crate) fn fcast_seek_body(position_secs: f64) -> String {
    json!({ "time": position_secs.max(0.0) }).to_string()
}

pub(crate) fn fcast_set_volume_body(level: f64) -> String {
    json!({ "volume": level.clamp(0.0, 1.0) }).to_string()
}

pub(crate) fn fcast_set_speed_body(speed: f64) -> String {
    json!({ "speed": speed.clamp(0.1, 4.0) }).to_string()
}

pub(crate) fn fcast_version_body(version: u32) -> String {
    json!({ "version": version }).to_string()
}

pub(crate) fn fcast_playback_update(body_json: &str) -> Option<(u8, f64, f64, f64)> {
    let value: serde_json::Value = serde_json::from_str(body_json).ok()?;
    let state = value.get("state")?.as_u64()? as u8;
    let time = value.get("time").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let duration = value.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let speed = value.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0);
    Some((state, time, duration, speed))
}

pub(crate) fn fcast_error_message(body_json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body_json).ok()?;
    Some(value.get("message")?.as_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http_schemes() {
        assert!(!validate_stream_url("file:///etc/passwd"));
        assert!(!validate_stream_url("javascript:alert(1)"));
        assert!(!validate_stream_url("not a url"));
    }

    #[test]
    fn accepts_http_and_https() {
        assert!(validate_stream_url("http://192.168.1.5:11470/stream"));
        assert!(validate_stream_url("https://example.com/movie.mkv"));
    }

    #[test]
    fn didl_title_with_markup_does_not_break_out_of_the_item_tag() {
        let args = dlna_set_av_transport_args("http://192.168.1.5/a.mkv", "</item><script>", None)
            .unwrap();
        assert!(!args.contains("<script>"));
    }

    #[test]
    fn rejects_local_file_media_url_for_every_protocol() {
        assert!(dlna_set_av_transport_args("file:///etc/passwd", "t", None).is_none());
        assert!(roku_launch_url("10.0.0.5", "file:///etc/passwd", None).is_none());
        assert!(airplay_play_body("file:///etc/passwd").is_none());
    }

    #[test]
    fn fcast_message_puts_the_opcode_before_the_body() {
        let message = fcast_encode_message(FCAST_OP_SEEK, &fcast_seek_body(12.5)).unwrap();
        assert_eq!(message[0], FCAST_OP_SEEK);
        assert_eq!(&message[1..], br#"{"time":12.5}"#);
        assert_eq!(
            fcast_decode_message(&message).unwrap(),
            (FCAST_OP_SEEK, r#"{"time":12.5}"#.to_string())
        );
    }

    #[test]
    fn fcast_bodyless_message_decodes_to_an_empty_body() {
        let message = fcast_encode_message(FCAST_OP_PAUSE, "").unwrap();
        assert_eq!(message, vec![FCAST_OP_PAUSE]);
        assert_eq!(
            fcast_decode_message(&message).unwrap(),
            (FCAST_OP_PAUSE, String::new())
        );
    }

    #[test]
    fn fcast_play_body_carries_the_container_and_omits_a_zero_resume() {
        let body = fcast_play_body("https://h/a.m3u8?t=1", 0.0).unwrap();
        assert!(body.contains(r#""container":"application/x-mpegurl""#));
        assert!(!body.contains("time"));
        let resumed = fcast_play_body("https://h/a.mp4", 42.0).unwrap();
        assert!(resumed.contains(r#""time":42.0"#));
        assert!(fcast_play_body("file:///etc/passwd", 0.0).is_none());
    }

    #[test]
    fn fcast_playback_update_needs_a_state() {
        assert_eq!(
            fcast_playback_update(r#"{"state":1,"time":10,"duration":100,"speed":1}"#),
            Some((1, 10.0, 100.0, 1.0))
        );
        assert_eq!(fcast_playback_update(r#"{"time":10}"#), None);
    }

    #[test]
    fn dash_urls_get_the_dash_container() {
        assert_eq!(guess_cast_content_type("https://h/a.mpd"), "application/dash+xml");
    }

}
