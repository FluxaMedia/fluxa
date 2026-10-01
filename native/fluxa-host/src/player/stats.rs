use std::time::Duration;

use fluxa_ui::StatsSection;
use serde_json::Value;

pub(super) const REFRESH: Duration = Duration::from_millis(500);
pub(super) const SAMPLE: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Default)]
pub struct PlaybackStats {
    pub container: Option<String>,
    pub video_codec: Option<String>,
    pub hwdec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub container_fps: Option<f64>,
    pub display_fps: Option<f64>,
    pub video_bitrate: Option<f64>,
    pub primaries: Option<String>,
    pub gamma: Option<String>,
    pub out_primaries: Option<String>,
    pub out_gamma: Option<String>,
    pub peak: Option<f64>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<String>,
    pub audio_bitrate: Option<f64>,
    pub video_decoder: Option<String>,
    pub audio: Option<Value>,
    pub cache_seconds: Option<f64>,
    pub cache_speed: Option<f64>,
    pub decoder_drops: Option<u64>,
    pub renderer_drops: Option<u64>,
    pub mistimed: Option<u64>,
    pub vo_delayed: Option<u64>,
    pub avsync: Option<f64>,
    pub dovi: Option<Value>,
}

fn bitrate(bits: f64) -> String {
    if bits >= 1_000_000.0 {
        format!("{:.1} Mbps", bits / 1_000_000.0)
    } else {
        format!("{:.0} kbps", bits / 1000.0)
    }
}

fn speed(bytes: f64) -> String {
    bitrate(bytes * 8.0)
}

fn fps(value: f64) -> String {
    format!("{value:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

fn colors(primaries: &Option<String>, gamma: &Option<String>, peak: Option<f64>) -> Option<String> {
    let mut parts: Vec<String> = [primaries, gamma].into_iter().flatten().cloned().collect();
    if parts.is_empty() {
        return None;
    }
    if let Some(peak) = peak.filter(|peak| *peak > 1.0) {
        parts.push(format!("{:.0} nits", peak * 203.0));
    }
    Some(parts.join(" · "))
}

fn section(title: &str, rows: Vec<(&'static str, Option<String>)>) -> Option<StatsSection> {
    let rows: Vec<(String, String)> = rows
        .into_iter()
        .filter_map(|(label, value)| Some((label.to_owned(), value?)))
        .collect();
    (!rows.is_empty()).then(|| StatsSection {
        title: title.to_owned(),
        rows,
    })
}

impl PlaybackStats {
    fn rows(&self) -> Rows {
        let size = self
            .width
            .zip(self.height)
            .map(|(width, height)| format!("{width}×{height}"));
        let frame_rate = self.fps.or(self.container_fps).map(|value| {
            match self.display_fps.filter(|display| *display > 0.0) {
                Some(display) => format!("{} / {}", fps(value), fps(display)),
                None => fps(value),
            }
        });
        let decoder = match (
            self.video_decoder.as_ref().or(self.video_codec.as_ref()),
            &self.hwdec,
        ) {
            (Some(name), Some(hw)) if hw != "no" => Some(format!("{name} → {hw}")),
            (Some(name), _) => Some(name.clone()),
            _ => None,
        };
        let format = match (self.sample_rate, &self.channels) {
            (Some(rate), Some(channels)) => {
                Some(format!("{:.1} kHz · {channels}", rate as f64 / 1000.0))
            }
            (Some(rate), None) => Some(format!("{:.1} kHz", rate as f64 / 1000.0)),
            (None, channels) => channels.clone(),
        };
        let count = |value: Option<u64>| value.map(|value| value.to_string());
        let video = vec![
            ("player.stats_container", self.container.clone()),
            ("player.stats_decoder", decoder),
            ("player.stats_resolution", size),
            ("player.stats_fps", frame_rate),
            (
                "player.stats_video_bitrate",
                self.video_bitrate.filter(|b| *b > 0.0).map(bitrate),
            ),
            (
                "player.stats_color_in",
                colors(&self.primaries, &self.gamma, self.peak),
            ),
            (
                "player.stats_color_out",
                colors(&self.out_primaries, &self.out_gamma, None),
            ),
        ];
        let audio: Vec<(&'static str, Option<String>)> = vec![
            ("player.stats_codec", self.audio_codec.clone()),
            ("player.stats_format", format),
            (
                "player.stats_audio_bitrate",
                self.audio_bitrate.filter(|b| *b > 0.0).map(bitrate),
            ),
        ];
        let audio = [
            audio,
            self.audio.as_ref().map(audio_rows).unwrap_or_default(),
        ]
        .concat();
        let network = vec![
            (
                "player.stats_buffer",
                self.cache_seconds.map(|seconds| format!("{seconds:.1}s")),
            ),
            (
                "player.stats_net",
                self.cache_speed.filter(|s| *s > 0.0).map(speed),
            ),
        ];
        let timing = vec![
            ("player.stats_decoder_drops", count(self.decoder_drops)),
            ("player.stats_renderer_drops", count(self.renderer_drops)),
            ("player.stats_mistimed", count(self.mistimed)),
            ("player.stats_vo_delayed", count(self.vo_delayed)),
            (
                "player.stats_avsync",
                self.avsync
                    .map(|value| format!("{:+.0} ms", value * 1000.0)),
            ),
        ];
        Rows {
            dovi: self.dovi.as_ref().and_then(dolby_vision),
            video,
            audio,
            network,
            timing,
        }
    }

    pub(super) fn sections(&self) -> Vec<StatsSection> {
        let rows = self.rows();
        let playback = [rows.network, rows.timing].concat();
        [
            rows.dovi,
            section("player.stats_video", rows.video),
            section("player.stats_audio", rows.audio),
            section("player.stats_playback", playback),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub(super) fn summary(
        &self,
        details: Vec<(&'static str, Option<String>)>,
    ) -> Vec<StatsSection> {
        let rows = self.rows();
        let video = [rows.video, rows.network].concat();
        let other = [details, rows.timing].concat();
        [
            section("settings.summary_video_network", video),
            section("settings.summary_audio", rows.audio),
            rows.dovi.map(|dovi| StatsSection {
                title: "settings.summary_dolby_vision".to_owned(),
                rows: dovi.rows,
            }),
            section("settings.summary_other", other),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

struct Rows {
    dovi: Option<StatsSection>,
    video: Vec<(&'static str, Option<String>)>,
    audio: Vec<(&'static str, Option<String>)>,
    network: Vec<(&'static str, Option<String>)>,
    timing: Vec<(&'static str, Option<String>)>,
}

fn dolby_vision(map: &Value) -> Option<StatsSection> {
    let text = |key: &str| {
        map[key]
            .as_str()
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let number = |key: &str| map[key].as_f64();
    let int = |key: &str| map[key].as_i64();
    let nits = |key: &str| number(key).filter(|value| *value > 0.0);
    let profile = int("profile")
        .filter(|profile| *profile >= 0)
        .map(|profile| {
            let mut label = match int("compatibility").filter(|compat| *compat > 0) {
                Some(compat) => format!("{profile}.{compat}"),
                None => profile.to_string(),
            };
            if let Some(level) = int("level").filter(|level| *level > 0) {
                label.push_str(&format!(" · L{level}"));
            }
            if let Some(layer) = text("el-type") {
                label.push_str(&format!(" · {layer}"));
            }
            label
        });
    let mode = text("path").map(|path| match text("output") {
        Some(output) => format!("{path} → {output}"),
        None => path,
    });
    let mapping = match (text("cm-version"), text("trim-used")) {
        (Some(version), Some(trim)) => Some(format!("{version} · {trim}")),
        (version, trim) => version.or(trim),
    };
    let brightness = map["l1"]["max"].as_f64().map(|max| {
        let avg = map["l1"]["avg"].as_f64().unwrap_or(0.0);
        format!("{max:.0} / {avg:.0} nits")
    });
    let tonemap = nits("tonemap-target")
        .zip(nits("tonemap-source"))
        .map(|(target, source)| format!("{source:.0} → {target:.0} nits"));
    let mastering = nits("mastering-max").map(|max| {
        format!(
            "{:.4}–{max:.0} nits",
            number("mastering-min").unwrap_or(0.0)
        )
    });
    let light = nits("max-cll")
        .map(|cll| format!("{cll:.0} / {:.0} nits", number("max-fall").unwrap_or(0.0)));
    let hdr10_plus = map["hdr10-plus"]["peak"]
        .as_f64()
        .map(|peak| format!("{peak:.0} nits"));
    let rpus = int("rpus").filter(|rpus| *rpus > 0).map(|rpus| {
        let mut label = rpus.to_string();
        for (key, name) in [("rpu-errors", "errors"), ("missing-rpu", "missing")] {
            if let Some(count) = int(key).filter(|count| *count > 0) {
                label.push_str(&format!(" · {count} {name}"));
            }
        }
        label
    });
    let gpu =
        number("gpu-ms")
            .or_else(|| number("gpu-avg-ms"))
            .map(|ms| match number("gpu-p99-ms") {
                Some(p99) => format!("{ms:.2} ms · p99 {p99:.1} ms"),
                None => format!("{ms:.2} ms"),
            });
    section(
        "player.stats_dolby_vision",
        vec![
            ("player.stats_dv_profile", profile),
            ("player.stats_dv_path", mode),
            ("player.stats_dv_reason", text("reason")),
            ("player.stats_dv_mapping", mapping),
            ("player.stats_dv_brightness", brightness),
            ("player.stats_dv_tonemap", tonemap),
            ("player.stats_dv_mastering", mastering),
            ("player.stats_dv_light_level", light),
            ("player.stats_dv_hdr10_plus", hdr10_plus),
            ("player.stats_dv_rpus", rpus),
            ("player.stats_dv_gpu", gpu),
            ("player.stats_dv_error", text("error")),
        ],
    )
}

fn audio_rows(map: &Value) -> Vec<(&'static str, Option<String>)> {
    let text = |value: &Value| value.as_str().filter(|v| !v.is_empty()).map(str::to_owned);
    let mode =
        text(&map["mode"]).map(
            |mode| match (mode.as_str(), text(&map["transcode-codec"])) {
                ("passthrough", _) => "Passthrough (sent as-is)".to_owned(),
                ("transcode", Some(codec)) => format!("Transcoded to {}", codec.to_uppercase()),
                _ => "Decoded to PCM".to_owned(),
            },
        );
    let source = &map["source"];
    let source_label = text(&source["codec"]).map(|codec| match text(&source["profile"]) {
        Some(profile) => format!("{codec} · {profile}"),
        None => codec,
    });
    let output = &map["output"];
    let output_label = text(&output["format"]).map(|format| {
        let mut label = format;
        if let Some(channels) = output["channels"].as_i64() {
            label.push_str(&format!(" · {channels} ch"));
        }
        if let Some(rate) = output["samplerate"].as_i64() {
            label.push_str(&format!(" · {:.1} kHz", rate as f64 / 1000.0));
        }
        if let Some(ao) = text(&output["ao"]) {
            label.push_str(&format!(" · {ao}"));
        }
        label
    });
    let failed = map["passthrough-failed"]
        .as_bool()
        .filter(|failed| *failed)
        .map(|_| "Yes".to_owned());
    vec![
        ("player.stats_audio_source", source_label),
        ("player.stats_audio_decoder", text(&map["decoder"])),
        ("player.stats_audio_mode", mode),
        ("player.stats_audio_output", output_label),
        ("player.stats_audio_reason", text(&map["reason"])),
        ("player.stats_audio_passthrough_failed", failed),
    ]
}
