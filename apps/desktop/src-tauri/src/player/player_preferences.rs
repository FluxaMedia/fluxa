use super::*;

pub(super) fn mpv_options_from_preferences(
    app: Option<&AppHandle>,
    preferences: &serde_json::Value,
) -> (Vec<(String, String)>, bool) {
    let mut options = Vec::new();
    let get = |key: &str| preferences.get(key).and_then(|v| v.as_str());

    if let Some(speed) = get("playbackSpeed").and_then(|v| v.parse::<f64>().ok()) {
        if (0.25..=4.0).contains(&speed) {
            options.push(("speed".to_string(), format!("{speed:.2}")));
        }
    }
    let buffer_request = json!({
        "cacheSizeMb": get("playerBufferCacheMb").and_then(|v| v.parse::<i64>().ok()),
        "forwardBufferSeconds": get("playerForwardBufferSeconds").and_then(|v| v.parse::<i64>().ok()),
        "backBufferSeconds": get("playerBackBufferSeconds").and_then(|v| v.parse::<i64>().ok()),
        "isTorrent": preferences.get("isTorrentPlayback").and_then(Value::as_bool).unwrap_or(false)
    });
    if let Some(targets_json) = FluxaCore::player_buffer_targets_json(&buffer_request.to_string()) {
        if let Ok(targets) = serde_json::from_str::<Value>(&targets_json) {
            if let Some(cache_bytes) = targets.get("cacheSizeBytes").and_then(Value::as_i64) {
                options.push(("demuxer-max-bytes".to_string(), cache_bytes.to_string()));
            }
            if let Some(forward_ms) = targets.get("forwardBufferMs").and_then(Value::as_i64) {
                let seconds = (forward_ms / 1000).max(1).to_string();
                options.push(("cache-secs".to_string(), seconds.clone()));
                options.push(("demuxer-readahead-secs".to_string(), seconds));
            }
            if let Some(back_ms) = targets.get("backBufferMs").and_then(Value::as_i64) {
                let cache_bytes = targets
                    .get("cacheSizeBytes")
                    .and_then(Value::as_i64)
                    .unwrap_or(100_000_000);
                let back_bytes = ((back_ms / 1000) * 1_310_720).clamp(10_000_000, cache_bytes);
                options.push(("demuxer-max-back-bytes".to_string(), back_bytes.to_string()));
            }
        }
    }
    push_audio_options(&mut options, preferences);
    if preferences
        .get("forceSoftwareAudio")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        options.push(("ad".to_string(), "lavc".to_string()));
    }
    if let Some(size) = get("subtitleSize").and_then(|v| v.parse::<f64>().ok()) {
        options.push((
            "sub-scale".to_string(),
            format!("{:.2}", (size / 100.0).clamp(0.5, 2.0)),
        ));
    }
    if let Some(position) = get("subtitlePosition").and_then(|v| v.parse::<f64>().ok()) {
        options.push((
            "sub-pos".to_string(),
            format!("{:.0}", position.clamp(0.0, 100.0)),
        ));
    }
    if let Some(font) = get("subtitleFont") {
        if !font.is_empty() && font != "default" {
            options.push(("sub-font".to_string(), font.to_string()));
        }
    }
    if let Some(app) = app {
        let state = app.state::<DesktopState>();
        if let Ok(dir) = custom_fonts::fonts_dir(&state) {
            options.push((
                "sub-fonts-dir".to_string(),
                dir.to_string_lossy().into_owned(),
            ));
        }
    }
    let anime4k_applied = push_anime_upscaling_options(
        &mut options,
        app,
        get("animeUpscalingMode"),
        get("animeUpscalingQuality"),
        get("animeUpscalingModePreset"),
        preferences
            .get("isAnimePlayback")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    );
    if let Some(app) = app {
        let custom_shaders = crate::player::mpv_shader_paths(app);
        if !custom_shaders.is_empty() {
            let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
            options.push((
                "glsl-shaders".to_string(),
                custom_shaders
                    .iter()
                    .map(|path| path.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(separator),
            ));
        }
    }
    push_frame_interpolation_options(&mut options, get("frameInterpolationMode"));
    let sub_text_opacity = get("subtitleTextOpacity")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if let Some(color) =
        get("subtitleColor").and_then(|v| css_hex_with_alpha_to_mpv_color(v, sub_text_opacity))
    {
        options.push(("sub-color".to_string(), color));
    }
    let sub_border_opacity = get("subtitleOutlineOpacity")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if let Some(color) = get("subtitleOutlineColor")
        .and_then(|v| css_hex_with_alpha_to_mpv_color(v, sub_border_opacity))
    {
        options.push(("sub-border-color".to_string(), color));
    }
    let subtitle_outline_size = get("subtitleOutlineSize")
        .and_then(|v| v.parse::<f64>().ok())
        .map(|size| size.clamp(0.0, 6.0));
    if let Some(size) = subtitle_outline_size {
        options.push((
            "sub-border-size".to_string(),
            format!("{:.1}", size.clamp(0.0, 6.0)),
        ));
    }
    if preferences
        .get("subtitleBold")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        options.push(("sub-bold".to_string(), "yes".to_string()));
    }
    if preferences
        .get("subtitleForceStyle")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        options.push(("sub-ass-override".to_string(), "force".to_string()));
    }
    let sub_bg_opacity = get("subtitleBackgroundOpacity")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    if let Some(color) = get("subtitleBackgroundColor")
        .and_then(|v| css_hex_with_alpha_to_mpv_color(v, sub_bg_opacity))
    {
        options.push(("sub-back-color".to_string(), color));
    }
    let anime_japanese = preferences
        .get("isAnimePlayback")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && preferences
            .get("animePreferJapaneseAudio")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    if !anime_japanese
        && preferences
            .get("autoEnableSubtitles")
            .and_then(|v| v.as_bool())
            == Some(false)
    {
        options.push(("sid".to_string(), "no".to_string()));
    }
    if preferences
        .get("subtitleShadow")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        options.push(("sub-shadow-offset".to_string(), "3".to_string()));
        options.push(("sub-shadow-color".to_string(), "#80000000".to_string()));
    } else {
        options.push(("sub-shadow-offset".to_string(), "0".to_string()));
    }
    match get("subtitleCharacterEdge") {
        Some("none") => {
            options.push(("sub-border-size".to_string(), "0".to_string()));
            options.push(("sub-shadow-offset".to_string(), "0".to_string()));
        }
        Some("raised") => {
            options.push(("sub-border-size".to_string(), "1.5".to_string()));
            options.push(("sub-shadow-offset".to_string(), "-1".to_string()));
        }
        Some("depressed") => {
            options.push(("sub-border-size".to_string(), "1.5".to_string()));
            options.push(("sub-shadow-offset".to_string(), "1".to_string()));
        }
        Some("uniform") => {
            options.push((
                "sub-border-size".to_string(),
                format!("{:.1}", subtitle_outline_size.unwrap_or(3.0)),
            ));
            options.push(("sub-shadow-offset".to_string(), "0".to_string()));
        }
        Some("drop-shadow") => {
            options.push(("sub-border-size".to_string(), "0".to_string()));
            options.push(("sub-shadow-offset".to_string(), "3".to_string()));
            options.push(("sub-shadow-color".to_string(), "#80000000".to_string()));
        }
        _ => {}
    }
    let audio_languages = if anime_japanese {
        language_list(&[
            Some("ja"),
            Some("jpn"),
            get("preferredAudioLanguage"),
            get("secondaryAudioLanguage"),
        ])
    } else {
        language_list(&[get("preferredAudioLanguage"), get("secondaryAudioLanguage")])
    };
    if !audio_languages.is_empty() {
        options.push(("alang".to_string(), audio_languages));
    }
    let mut subtitle_languages = language_list(&[
        get("preferredSubtitleLanguage"),
        get("secondarySubtitleLanguage"),
    ]);
    if anime_japanese && subtitle_languages.is_empty() {
        subtitle_languages = "eng,en".to_string();
    }
    if !subtitle_languages.is_empty() {
        options.push(("slang".to_string(), subtitle_languages));
    }
    if let Some(custom) = get("mpvCustomOptions") {
        for line in custom.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((name, value)) = line.split_once('=') {
                let name = name.trim();
                let value = value.trim();
                if is_safe_mpv_option_name(name)
                    && !is_route_owned_audio_option(name)
                    && !value.is_empty()
                {
                    options.push((name.to_string(), value.to_string()));
                }
            }
        }
    }
    (options, anime4k_applied)
}

/// Keep the desktop MPV path aligned with the shared audio policy:
/// Reference preserves the encoded bitstream when the selected audio output
/// can handle it. Any DSP mode must decode to PCM first, so passthrough is
/// explicitly cleared. Custom MPV options remain an expert escape hatch for
/// unrelated settings, but cannot override the route-owned audio invariant.
fn push_audio_options(options: &mut Vec<(String, String)>, preferences: &Value) {
    const PASSTHROUGH_CODECS: &str = "ac3,eac3,ac4,dts,dts-hd,truehd";
    let requested_mode = preferences
        .get("audioProcessingMode")
        .and_then(Value::as_str)
        .unwrap_or("reference")
        .to_ascii_lowercase();
    let legacy_stable_volume = preferences
        .get("stableVolume")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let force_software_audio = preferences
        .get("forceSoftwareAudio")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mode = match requested_mode.as_str() {
        "balanced" | "night" if !legacy_stable_volume => requested_mode.as_str(),
        _ if legacy_stable_volume => "balanced",
        _ => "reference",
    };

    // auto-safe keeps the source channel layout where the output supports it
    // and only folds down when the sink cannot accept the layout.
    options.push(("audio-channels".to_string(), "auto-safe".to_string()));
    match mode {
        "reference" => {
            // Software-audio mode is an explicit PCM request. Keeping SPDIF
            // enabled here would let MPV bypass the decoder and silently
            // defeat the user's setting.
            options.push((
                "audio-spdif".to_string(),
                if force_software_audio {
                    String::new()
                } else {
                    PASSTHROUGH_CODECS.to_string()
                },
            ));
            options.push(("af".to_string(), String::new()));
        }
        "balanced" => {
            options.push(("audio-spdif".to_string(), String::new()));
            if legacy_stable_volume {
                options.push((
                    "af".to_string(),
                    "lavfi=[dynaudnorm,alimiter=limit=0.98]".to_string(),
                ));
            } else {
                options.push((
                    "af".to_string(),
                    "lavfi=[acompressor=threshold=0.78:ratio=1.5:attack=30:release=300:link=maximum,alimiter=limit=0.98]".to_string(),
                ));
            }
        }
        "night" => {
            options.push(("audio-spdif".to_string(), String::new()));
            options.push((
                "af".to_string(),
                "lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]".to_string(),
            ));
        }
        _ => unreachable!("audio mode is normalized above"),
    }
}

pub(super) fn push_anime_upscaling_options(
    options: &mut Vec<(String, String)>,
    app: Option<&AppHandle>,
    mode: Option<&str>,
    quality: Option<&str>,
    mode_preset: Option<&str>,
    is_anime_playback: bool,
) -> bool {
    options.push(("glsl-shaders".to_string(), String::new()));

    let quality = match mode.unwrap_or("off") {
        "auto" if is_anime_playback => quality.unwrap_or("anime4k_m"),
        "anime4k_s" | "anime4k_m" | "anime4k_l" if is_anime_playback => mode.unwrap_or("off"),
        _ => return false,
    };
    let mode_preset = mode_preset.unwrap_or("a");
    let Some(chain_path) = resolve_anime4k_chain(app, quality, mode_preset) else {
        log::warn!("Anime4K shader chain for '{quality}'/'{mode_preset}' was not found");
        return false;
    };

    options.push(("scale".to_string(), "ewa_lanczossharp".to_string()));
    options.push(("cscale".to_string(), "ewa_lanczos".to_string()));
    options.push(("dscale".to_string(), "mitchell".to_string()));
    options.push(("correct-downscaling".to_string(), "yes".to_string()));
    options.push(("linear-downscaling".to_string(), "yes".to_string()));
    options.push(("glsl-shaders".to_string(), chain_path));
    true
}

pub(super) fn anime4k_thin_shader(tier: &str) -> &'static str {
    match tier {
        "anime4k_s" => "Anime4K_Thin_VeryFast.glsl",
        "anime4k_l" => "Anime4K_Thin_HQ.glsl",
        _ => "Anime4K_Thin_Fast.glsl",
    }
}

pub(super) fn anime4k_chain_shaders(tier: &str, mode: &str) -> Vec<String> {
    let mut chain = vec!["Anime4K_Clamp_Highlights.glsl".to_string()];

    if tier == "anime4k_s" {
        match mode {
            "b" => {
                chain.push("Anime4K_Restore_CNN_Soft_S.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
            }
            "bb" => {
                chain.push("Anime4K_Restore_CNN_Soft_S.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push("Anime4K_Restore_CNN_Soft_S.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
            }
            "c" => {
                chain.push("Anime4K_Upscale_Denoise_CNN_x2_S.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
            }
            "ca" => {
                chain.push("Anime4K_Upscale_Denoise_CNN_x2_S.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push("Anime4K_Restore_CNN_S.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
            }
            "aa" => {
                chain.push("Anime4K_Restore_CNN_S.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
                chain.push("Anime4K_Restore_CNN_S.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
            }
            _ => {
                chain.push("Anime4K_Restore_CNN_S.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push("Anime4K_Upscale_CNN_x2_S.glsl".to_string());
            }
        }
    } else {
        let (primary, secondary) = if tier == "anime4k_l" {
            ("VL", "M")
        } else {
            ("M", "S")
        };
        match mode {
            "b" => {
                chain.push(format!("Anime4K_Restore_CNN_Soft_{primary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{primary}.glsl"));
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push(format!("Anime4K_Upscale_CNN_x2_{secondary}.glsl"));
            }
            "bb" => {
                chain.push(format!("Anime4K_Restore_CNN_Soft_{primary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{primary}.glsl"));
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push(format!("Anime4K_Restore_CNN_Soft_{secondary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{secondary}.glsl"));
            }
            "c" => {
                chain.push(format!("Anime4K_Upscale_Denoise_CNN_x2_{primary}.glsl"));
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push(format!("Anime4K_Upscale_CNN_x2_{secondary}.glsl"));
            }
            "ca" => {
                chain.push(format!("Anime4K_Upscale_Denoise_CNN_x2_{primary}.glsl"));
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push(format!("Anime4K_Restore_CNN_{secondary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{secondary}.glsl"));
            }
            "aa" => {
                chain.push(format!("Anime4K_Restore_CNN_{primary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{primary}.glsl"));
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push(format!("Anime4K_Restore_CNN_{secondary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{secondary}.glsl"));
            }
            _ => {
                chain.push(format!("Anime4K_Restore_CNN_{primary}.glsl"));
                chain.push(format!("Anime4K_Upscale_CNN_x2_{primary}.glsl"));
                chain.push("Anime4K_AutoDownscalePre_x2.glsl".to_string());
                chain.push("Anime4K_AutoDownscalePre_x4.glsl".to_string());
                chain.push(format!("Anime4K_Upscale_CNN_x2_{secondary}.glsl"));
            }
        }
    }

    chain.push(anime4k_thin_shader(tier).to_string());
    chain
}

pub(super) fn resolve_anime4k_chain(
    app: Option<&AppHandle>,
    tier: &str,
    mode: &str,
) -> Option<String> {
    let shader_names = anime4k_chain_shaders(tier, mode);
    let mut paths = Vec::with_capacity(shader_names.len());
    for shader_name in &shader_names {
        let path = resolve_shader_path(app, shader_name)?;
        paths.push(path.replace('\\', "/"));
    }
    let separator = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    Some(paths.join(separator))
}

pub(super) fn anime4k_should_apply(preferences: &Value) -> bool {
    let is_anime_playback = preferences
        .get("isAnimePlayback")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    matches!(
        preferences
            .get("animeUpscalingMode")
            .and_then(Value::as_str),
        Some("auto" | "anime4k_s" | "anime4k_m" | "anime4k_l")
    ) && is_anime_playback
}

pub(super) fn resolve_shader_path(app: Option<&AppHandle>, shader_name: &str) -> Option<String> {
    let resource_path = format!("assets/mpv-shaders/anime4k/{shader_name}");
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&resource_path);
    if cfg!(debug_assertions) && dev_path.exists() {
        return Some(dev_path.to_string_lossy().into_owned());
    }
    if let Some(app) = app {
        if let Ok(path) = app.path().resolve(&resource_path, BaseDirectory::Resource) {
            if path.exists() {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }

    if dev_path.exists() {
        return Some(dev_path.to_string_lossy().into_owned());
    }
    None
}

pub(super) fn push_frame_interpolation_options(
    options: &mut Vec<(String, String)>,
    mode: Option<&str>,
) {
    match mode.unwrap_or("off") {
        "display_resample" => {
            options.push(("video-sync".to_string(), "display-resample".to_string()));
            options.push(("interpolation".to_string(), "yes".to_string()));
            options.push(("tscale".to_string(), "oversample".to_string()));
        }
        "smooth" => {
            options.push(("video-sync".to_string(), "display-resample".to_string()));
            options.push(("interpolation".to_string(), "yes".to_string()));
            options.push(("tscale".to_string(), "mitchell".to_string()));
            options.push(("tscale-clamp".to_string(), "0.0".to_string()));
        }
        _ => {
            options.push(("video-sync".to_string(), "audio".to_string()));
            options.push(("interpolation".to_string(), "no".to_string()));
        }
    }
}

pub(super) fn css_hex_with_alpha_to_mpv_color(value: &str, opacity: f64) -> Option<String> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some(format!("#{alpha:02X}{hex}"))
    } else {
        None
    }
}

pub(super) fn is_safe_mpv_option_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '/')
}

/// These options are the single source of truth for the shared audio policy.
/// Allowing a later custom option to replace them can re-enable passthrough
/// during DSP or disable the route-selected channel layout.
fn is_route_owned_audio_option(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "audio-spdif" | "audio-channels" | "af")
}

pub(super) fn language_list(values: &[Option<&str>]) -> String {
    values
        .iter()
        .filter_map(|v| v.map(str::trim))
        .filter(|v| !v.is_empty() && *v != "none")
        .filter(|v| v.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-'))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::mpv_options_from_preferences;
    use serde_json::json;

    fn option_value(options: &[(String, String)], name: &str) -> Option<String> {
        options
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    }

    #[test]
    fn force_software_audio_clears_reference_passthrough() {
        let (options, _) = mpv_options_from_preferences(
            None,
            &json!({
                "audioProcessingMode": "reference",
                "forceSoftwareAudio": true
            }),
        );

        assert_eq!(option_value(&options, "audio-spdif"), Some(String::new()));
        assert_eq!(option_value(&options, "ad"), Some("lavc".to_string()));
    }

    #[test]
    fn reference_keeps_passthrough_when_software_audio_is_not_forced() {
        let (options, _) = mpv_options_from_preferences(
            None,
            &json!({ "audioProcessingMode": "reference" }),
        );

        assert_eq!(
            option_value(&options, "audio-spdif"),
            Some("ac3,eac3,ac4,dts,dts-hd,truehd".to_string())
        );
    }

    #[test]
    fn legacy_stable_volume_keeps_limiter_headroom() {
        let (options, _) = mpv_options_from_preferences(
            None,
            &json!({
                "audioProcessingMode": "balanced",
                "stableVolume": true
            }),
        );

        assert_eq!(
            option_value(&options, "af"),
            Some("lavfi=[dynaudnorm,alimiter=limit=0.98]".to_string())
        );
    }

    #[test]
    fn custom_options_cannot_override_the_shared_audio_policy() {
        let (options, _) = mpv_options_from_preferences(
            None,
            &json!({
                "audioProcessingMode": "night",
                "mpvCustomOptions": "audio-spdif=ac3\naudio-channels=stereo\naf=\nvideo-sync=display-resample"
            }),
        );

        assert_eq!(
            options.iter().filter(|(key, _)| key == "audio-spdif").count(),
            1
        );
        assert_eq!(
            option_value(&options, "audio-spdif"),
            Some(String::new())
        );
        assert_eq!(option_value(&options, "audio-channels"), Some("auto-safe".to_string()));
        assert_eq!(option_value(&options, "af"), Some("lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]".to_string()));
        assert!(options.iter().any(|(key, value)| {
            key == "video-sync" && value == "display-resample"
        }));
    }
}
