use super::*;

pub(super) const STALE_GRACE: Duration = Duration::from_secs(3);

pub(crate) fn direct_playback_command(item: &Value, profile: &Value) -> Value {
    if stream_policy::resumes_saved_stream(item, Some(profile)) {
        return json!({
            "type": "continueWatchingPlaybackRequested",
            "item": item,
            "language": profile_language(profile),
            "profile": profile,
        });
    }
    json!({
        "type": "directPlaybackRequested",
        "meta": item,
        "language": profile_language(profile),
        "profile": profile,
    })
}

pub(crate) fn preview_scene(state: &mut RendererState, scene: &str) {
    let track =
        |id: &str, subtitle: bool, language: &str, title: &str, selected: bool| VideoTrack {
            id: id.to_owned(),
            subtitle,
            language: Some(language.to_owned()),
            title: Some(title.to_owned()),
            selected,
            external: false,
            forced: false,
            format: (!subtitle).then(|| "Stereo · AAC".to_owned()),
        };
    let style = state.settings.values.clone();
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.tracks = vec![
        track("a1", false, "jpn", "Japonca 5.1", true),
        track("a2", false, "eng", "İngilizce 5.1", false),
        track("a3", false, "tur", "Türkçe 2.0", false),
        track("s1", true, "tur", "Türkçe", true),
        track("s2", true, "eng", "İngilizce", false),
        track("s3", true, "jpn", "Japonca", false),
    ];
    player.style = style;
    player.status.chapters = vec![
        (0.0, "Açılış".to_owned()),
        (95.0, "Fener Bekçisi".to_owned()),
        (610.0, "Fırtına".to_owned()),
        (1380.0, "Kayıp Defter".to_owned()),
        (2400.0, "Jenerik".to_owned()),
    ];
    player.status.duration = 2712.0;
    player.status.position = 754.0;
    player.panel = match scene {
        "chapters" => Some(Panel::Menu(menu::Menu::Chapters)),
        "tracks" => Some(Panel::Menu(menu::Menu::Tracks)),
        "speed" => Some(Panel::Menu(menu::Menu::Speed)),
        "episodes" => Some(Panel::Menu(menu::Menu::Episodes)),
        "style" => Some(Panel::Menu(menu::Menu::Style)),
        "sync" | "sync-open" => {
            player.overlay.addon_subtitles = ["tur", "tur", "por", "kor", "ara", "ara"]
                .iter()
                .enumerate()
                .map(|(index, lang)| {
                    json!({"url": format!("https://example.com/{index}.srt"), "lang": lang})
                })
                .collect();
            player.sync.matches_open = scene == "sync-open";
            if scene == "sync" {
                player.sync.message = Some(("player.sync_unavailable", None));
            }
            player.subtitle_delay = -2.4;
            Some(Panel::Menu(menu::Menu::Sync))
        }
        _ => None,
    };
}

pub(crate) fn open_file(state: &mut RendererState, uri: &str, title: &str) -> Value {
    let mut player = PlayerSession::new(json!({"name": title, "type": "movie"}));
    player.local = true;
    player.started = true;
    state.player = Some(player);
    state.shuffle = None;
    state.screen_scroll_offsets.remove(&Route::Player);
    crate::reset_ui(state);
    json!({"type": "playerResolvePlaybackRequested", "url": uri, "title": title})
}
