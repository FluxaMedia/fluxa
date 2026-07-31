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
    if preferences
        .get("forceSoftwareAudio")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        options.push(("ad".to_string(), "lavc".to_string()));
    }
    if preferences
        .get("stableVolume")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        options.push(("af".to_string(), "dynaudnorm".to_string()));
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
                if is_safe_mpv_option_name(name) && !value.is_empty() {
                    options.push((name.to_string(), value.to_string()));
                }
            }
        }
    }
    (options, anime4k_applied)
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

pub(super) fn language_list(values: &[Option<&str>]) -> String {
    values
        .iter()
        .filter_map(|v| v.map(str::trim))
        .filter(|v| !v.is_empty() && *v != "none")
        .filter(|v| v.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-'))
        .collect::<Vec<_>>()
        .join(",")
}
