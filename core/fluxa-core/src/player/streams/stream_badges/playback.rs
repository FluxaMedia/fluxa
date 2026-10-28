use super::{BadgeKind, Options, Rules, StreamBadge, parse};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playing {
    pub width: Option<u32>,
    pub video_codec: Option<String>,
    pub gamma: Option<String>,
    pub dolby_vision: bool,
    pub hdr10_plus: bool,
    pub audio_codec: Option<String>,
    pub audio_profile: Option<String>,
    pub channels: Option<String>,
}

impl Playing {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    pub fn text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(width) = self.width {
            parts.push(
                match width {
                    3500.. => "2160p",
                    2500.. => "1440p",
                    1800.. => "1080p",
                    1200.. => "720p",
                    _ => "480p",
                }
                .to_owned(),
            );
        }
        parts.extend(self.video_codec.clone());
        match self
            .gamma
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("pq") if self.hdr10_plus => parts.push("HDR10+".to_owned()),
            Some("pq") => parts.push("HDR10".to_owned()),
            Some("hlg") => parts.push("HLG".to_owned()),
            _ => {}
        }
        if self.dolby_vision {
            parts.push("DV".to_owned());
        }
        parts.extend(self.audio_codec.clone());
        parts.extend(self.audio_profile.clone());
        parts.extend(self.channels.as_deref().and_then(channel_layout));
        parts.join(" ")
    }

    fn knows(&self, kind: BadgeKind) -> bool {
        match kind {
            BadgeKind::Resolution => self.width.is_some(),
            BadgeKind::Codec => self.video_codec.is_some(),
            BadgeKind::Hdr => self.gamma.is_some(),
            BadgeKind::Audio => self.audio_codec.is_some(),
            _ => false,
        }
    }
}

fn channel_layout(channels: &str) -> Option<String> {
    let layout: String = channels
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    layout.contains('.').then_some(layout)
}

pub fn parse_playing(
    text: &str,
    video_size: Option<i64>,
    options: Options,
    rules: &Rules,
    playing: &Playing,
) -> Vec<StreamBadge> {
    let declared = parse(text, video_size, options, rules);
    if playing.is_empty() {
        return declared;
    }
    let live = parse(
        &playing.text(),
        None,
        Options {
            file_size: false,
            ..options
        },
        rules,
    );
    let atmos_carrier = live
        .iter()
        .any(|badge| matches!(badge.label.as_str(), "TrueHD" | "DD+"));
    let mut badges: Vec<StreamBadge> = declared
        .into_iter()
        .filter(|badge| !playing.knows(badge.kind) || (badge.label == "Atmos" && atmos_carrier))
        .collect();
    for badge in live {
        let wanted = playing.knows(badge.kind) || badge.kind == BadgeKind::Custom;
        if wanted && !badges.iter().any(|known| known.label == badge.label) {
            badges.push(badge);
        }
    }
    badges.sort_by_key(|badge| badge.kind as u8);
    badges
}
