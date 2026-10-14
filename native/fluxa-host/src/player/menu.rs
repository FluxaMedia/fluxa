use super::*;
use fluxa_ui::{PanelRow, PlayerPanel, localized};

const SPEEDS: [f64; 7] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const ASPECTS: [&str; 5] = ["-1", "16:9", "4:3", "21:9", "1:1"];
const SLEEP_MINUTES: [u32; 5] = [0, 15, 30, 45, 60];
const DELAY_STEP: f64 = 0.1;
const DELAY_COARSE_STEP: f64 = 0.5;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Menu {
    Tracks,
    Speed,
    Episodes,
    Settings,
    Delay(bool),
    Style,
}

enum Item {
    None,
    Audio(String),
    Subtitle(Option<String>),
    Speed(f64),
    Episode(String),
    Aspect,
    AudioDelay(f64),
    SubtitleDelay(f64),
    OpenDelay(bool),
    DelayReset(bool),
    Sleep,
    Upscaling,
    UpscalingMode,
    Fit,
    Stats,
    OpenStyle,
    Style(&'static str),
    Language(bool, String),
    LanguageBack,
}

fn row(label: String, value: Option<String>, selected: bool, item: Item) -> (PanelRow, Item) {
    (
        PanelRow {
            label,
            value,
            selected,
            ..Default::default()
        },
        item,
    )
}

fn heading(label: String) -> (PanelRow, Item) {
    (
        PanelRow {
            label,
            heading: true,
            ..Default::default()
        },
        Item::None,
    )
}

fn language_key(code: &str) -> Option<&'static str> {
    let key = match code.to_ascii_lowercase().get(..2)? {
        "tr" => "language.turkish",
        "en" => "language.english",
        "ar" => "language.arabic",
        "zh" | "ch" => "language.chinese",
        "fr" => "language.french",
        "de" | "ge" => "language.german",
        "hi" => "language.hindi",
        "it" => "language.italian",
        "ja" | "jp" => "language.japanese",
        "ko" | "kr" => "language.korean",
        "pt" => "language.portuguese",
        "ru" => "language.russian",
        "es" | "sp" => "language.spanish",
        _ => return None,
    };
    Some(key)
}

fn group_label(track: &VideoTrack, language: &str) -> String {
    let code = track.language.as_deref().filter(|code| !code.is_empty());
    match code {
        Some(code) => match language_key(code) {
            Some(key) => localized(key, language),
            None => code.to_uppercase(),
        },
        None => localized("player.language_unknown", language),
    }
}

fn track_row(
    track: &VideoTrack,
    index: usize,
    language: &str,
    subtitles: bool,
) -> (PanelRow, Item) {
    let title = track.title.as_deref().filter(|title| !title.is_empty());
    let label = match title {
        Some(title) => title.to_owned(),
        None => format!(
            "{} {}",
            localized("player.audio_track", language),
            index + 1
        ),
    };
    let item = if subtitles {
        Item::Subtitle(Some(track.id.clone()))
    } else {
        Item::Audio(track.id.clone())
    };
    row(
        label,
        track
            .external
            .then(|| localized("player.external", language)),
        track.selected,
        item,
    )
}

fn language_groups<'a>(
    tracks: Vec<&'a VideoTrack>,
    language: &str,
) -> Vec<(String, Vec<&'a VideoTrack>)> {
    let mut groups: Vec<(String, Vec<&VideoTrack>)> = Vec::new();
    for track in tracks {
        let label = group_label(track, language);
        match groups.iter_mut().find(|(name, _)| *name == label) {
            Some((_, members)) => members.push(track),
            None => groups.push((label, vec![track])),
        }
    }
    groups.sort_by_key(|(_, members)| !members.iter().any(|track| track.selected));
    groups
}

fn grouped(
    rows: &mut Vec<(PanelRow, Item)>,
    tracks: Vec<&VideoTrack>,
    language: &str,
    subtitles: bool,
) {
    let groups = language_groups(tracks, language);
    if groups.len() <= 1 {
        for (index, track) in groups.iter().flat_map(|(_, members)| members).enumerate() {
            rows.push(track_row(track, index, language, subtitles));
        }
        return;
    }
    for (name, members) in groups {
        let selected = members.iter().any(|track| track.selected);
        rows.push(row(
            name.clone(),
            Some(members.len().to_string()),
            selected,
            Item::Language(subtitles, name),
        ));
    }
}

pub(super) fn back(player: &mut PlayerSession) -> bool {
    player.track_lang.take().is_some()
}

fn episodes(player: &PlayerSession) -> Vec<Value> {
    let videos = player.meta.get("videos").cloned().unwrap_or(Value::Null);
    match core_value("playerEpisodes", json!({"videos": videos})) {
        Some(Value::Array(videos)) => videos,
        _ => Vec::new(),
    }
}

pub(super) fn has_episodes(player: &PlayerSession) -> bool {
    !episodes(player).is_empty()
}

fn adjacent(player: &PlayerSession, offset: isize) -> Option<String> {
    let current = player.overlay.current_id()?;
    let list = episodes(player);
    let index = list
        .iter()
        .position(|video| video.get("id").and_then(Value::as_str) == Some(current))?;
    let target = list.get(index.checked_add_signed(offset)?)?;
    target.get("id").and_then(Value::as_str).map(str::to_owned)
}

pub(super) fn has_previous(player: &PlayerSession) -> bool {
    adjacent(player, -1).is_some()
}

fn items(player: &PlayerSession, menu: Menu) -> (String, Vec<(PanelRow, Item)>) {
    let language = player.language.as_str();
    let text = |key: &str| localized(key, language);
    match menu {
        Menu::Tracks if player.track_lang.is_some() => {
            let (subtitles, name) = player.track_lang.clone().unwrap_or_default();
            let mut rows = vec![row(text("player.back"), None, false, Item::LanguageBack)];
            rows.push(heading(name.clone()));
            let members = player
                .tracks
                .iter()
                .filter(|track| track.subtitle == subtitles && group_label(track, language) == name)
                .enumerate()
                .map(|(index, track)| track_row(track, index, language, subtitles));
            rows.extend(members);
            (String::new(), rows)
        }
        Menu::Tracks => {
            let mut rows = vec![heading(text("player.audio"))];
            grouped(
                &mut rows,
                player.tracks.iter().filter(|t| !t.subtitle).collect(),
                language,
                false,
            );
            rows.push(heading(text("player.subtitles")));
            let subtitles: Vec<&VideoTrack> = player.tracks.iter().filter(|t| t.subtitle).collect();
            rows.push(row(
                text("player.off"),
                None,
                !subtitles.iter().any(|track| track.selected),
                Item::Subtitle(None),
            ));
            grouped(&mut rows, subtitles, language, true);
            rows.push(row(
                text("player.subtitle_style"),
                None,
                false,
                Item::OpenStyle,
            ));
            (String::new(), rows)
        }
        Menu::Speed => {
            let rows = SPEEDS
                .iter()
                .map(|speed| {
                    let label = if *speed == 1.0 {
                        text("player.speed_standard")
                    } else {
                        format!("{speed}x")
                    };
                    row(
                        label,
                        None,
                        (player.speed - speed).abs() < 0.01,
                        Item::Speed(*speed),
                    )
                })
                .collect();
            (text("player.speed"), rows)
        }
        Menu::Episodes => {
            let rows = episodes(player)
                .into_iter()
                .filter_map(|video| {
                    let id = video.get("id").and_then(Value::as_str)?;
                    let number = |key: &str| video.get(key).and_then(Value::as_i64).unwrap_or(0);
                    let episode = video
                        .get("episode")
                        .and_then(Value::as_i64)
                        .unwrap_or_else(|| number("number"));
                    let name = video
                        .get("name")
                        .or_else(|| video.get("title"))
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    let mut entry = row(
                        format!("{}. {}", episode, name),
                        None,
                        player.overlay.current_id() == Some(id),
                        Item::Episode(id.to_owned()),
                    );
                    entry.0.thumbnail = video
                        .get("thumbnail")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    entry.0.detail = ["overview", "description"]
                        .iter()
                        .find_map(|key| video.get(*key).and_then(Value::as_str))
                        .filter(|detail| !detail.is_empty())
                        .map(str::to_owned);
                    Some(entry)
                })
                .collect();
            (text("player.episodes"), rows)
        }
        Menu::Settings => {
            let delay = |value: f64| format!("{value:+.1}s");
            let aspect = match ASPECTS[player.aspect] {
                "-1" => text("player.aspect_auto"),
                other => other.to_owned(),
            };
            let sleep = match SLEEP_MINUTES[player.sleep] {
                0 => text("player.off"),
                minutes => text("player.minutes_format").replace("%s", &minutes.to_string()),
            };
            let mut anime4k = row(text("player.anime4k"), None, false, Item::Upscaling);
            anime4k.0.switch = Some(player.anime4k);
            let mode = match player.upscaling.as_str() {
                "b" => text("player.anime4k_mode_b"),
                "c" => text("player.anime4k_mode_c"),
                _ => text("player.anime4k_mode_a"),
            };
            let mut stats = row(text("player.playback_summary"), None, false, Item::Stats);
            stats.0.switch = Some(player.stats_visible);
            let mut rows = vec![anime4k];
            if player.anime4k {
                rows.push(row(
                    text("player.anime4k_mode"),
                    Some(mode),
                    false,
                    Item::UpscalingMode,
                ));
            }
            rows.extend([
                row(
                    text("player.video_fit"),
                    Some(text(&format!("player.fit_{}", player.fit))),
                    false,
                    Item::Fit,
                ),
                row(
                    text("player.aspect_ratio"),
                    Some(aspect),
                    false,
                    Item::Aspect,
                ),
                row(
                    text("player.audio_delay"),
                    Some(delay(player.audio_delay)),
                    false,
                    Item::OpenDelay(false),
                ),
                row(
                    text("player.subtitle_delay"),
                    Some(delay(player.subtitle_delay)),
                    false,
                    Item::OpenDelay(true),
                ),
                row(text("player.sleep_timer"), Some(sleep), false, Item::Sleep),
                stats,
            ]);
            (text("player.settings"), rows)
        }
        Menu::Style => {
            let rows = fluxa_ui::subtitle_style_rows(language)
                .into_iter()
                .map(|(key, label, options)| {
                    let current = player.style.get(key);
                    if options.is_empty() {
                        let mut entry = row(label, None, false, Item::Style(key));
                        entry.0.switch = Some(current.and_then(Value::as_bool).unwrap_or(false));
                        entry
                    } else {
                        let value = current.and_then(Value::as_str).unwrap_or(options[0]);
                        row(
                            label,
                            Some(fluxa_ui::option_label(key, value, language)),
                            false,
                            Item::Style(key),
                        )
                    }
                })
                .collect();
            (text("player.subtitle_style"), rows)
        }
        Menu::Delay(subtitles) => {
            let current = if subtitles {
                player.subtitle_delay
            } else {
                player.audio_delay
            };
            let step = |value: f64| {
                let item = if subtitles {
                    Item::SubtitleDelay(value)
                } else {
                    Item::AudioDelay(value)
                };
                row(format!("{value:+.1}s"), None, false, item)
            };
            let rows = vec![
                heading(format!("{current:+.1}s")),
                step(DELAY_COARSE_STEP),
                step(DELAY_STEP),
                row(
                    text("player.delay_reset"),
                    None,
                    false,
                    Item::DelayReset(subtitles),
                ),
                step(-DELAY_STEP),
                step(-DELAY_COARSE_STEP),
            ];
            let title = if subtitles {
                "player.subtitle_delay"
            } else {
                "player.audio_delay"
            };
            (text(title), rows)
        }
    }
}

pub(super) fn model(player: &PlayerSession, menu: Menu) -> PlayerPanel {
    let (title, rows) = items(player, menu);
    PlayerPanel {
        title,
        message: None,
        rows: rows.into_iter().map(|(row, _)| row).collect(),
        list: true,
        look: (menu == Menu::Style).then(|| fluxa_ui::SubtitleLook::from_values(&player.style)),
    }
}

pub(super) fn open(state: &mut RendererState, menu: Menu) {
    if matches!(menu, Menu::Tracks)
        && let Some(video) = state.video.as_mut()
        && let Some(player) = state.player.as_mut()
    {
        player.tracks = video.tracks();
    }
    let style = state.settings.values.clone();
    if let Some(player) = state.player.as_mut() {
        player.style = style;
        player.track_lang = None;
        let toggled = matches!(player.panel, Some(Panel::Menu(open)) if open == menu);
        player.panel = (!toggled).then_some(Panel::Menu(menu));
        player.touch();
    }
}

pub(super) fn activate_row(state: &mut RendererState, menu: Menu, index: usize) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let Some((_, item)) = items(player, menu).1.into_iter().nth(index) else {
        return;
    };
    player.touch();
    match item {
        Item::None => {}
        Item::Language(subtitles, name) => player.track_lang = Some((subtitles, name)),
        Item::LanguageBack => player.track_lang = None,
        Item::Audio(id) => select(
            state,
            TrackSelection {
                audio: Some(id),
                ..Default::default()
            },
        ),
        Item::Subtitle(Some(id)) => select(
            state,
            TrackSelection {
                subtitle: Some(id),
                ..Default::default()
            },
        ),
        Item::Subtitle(None) => select(
            state,
            TrackSelection {
                subtitles_off: true,
                ..Default::default()
            },
        ),
        Item::Speed(speed) => {
            player.speed = speed;
            player.panel = None;
            command(state, VideoCommand::SetSpeed(speed));
        }
        Item::Episode(id) => start_episode(state, &id),
        Item::Aspect => {
            player.aspect = (player.aspect + 1) % ASPECTS.len();
            let value = ASPECTS[player.aspect];
            mpv(state, "video-aspect-override", value);
        }
        Item::AudioDelay(step) => {
            player.audio_delay = ((player.audio_delay + step) * 10.0).round() / 10.0;
            let value = player.audio_delay.to_string();
            mpv(state, "audio-delay", &value);
        }
        Item::SubtitleDelay(step) => {
            player.subtitle_delay = ((player.subtitle_delay + step) * 10.0).round() / 10.0;
            let value = player.subtitle_delay.to_string();
            mpv(state, "sub-delay", &value);
        }
        Item::OpenDelay(subtitles) => player.panel = Some(Panel::Menu(Menu::Delay(subtitles))),
        Item::DelayReset(subtitles) => {
            if subtitles {
                player.subtitle_delay = 0.0;
                mpv(state, "sub-delay", "0");
            } else {
                player.audio_delay = 0.0;
                mpv(state, "audio-delay", "0");
            }
        }
        Item::OpenStyle => player.panel = Some(Panel::Menu(Menu::Style)),
        Item::Style(key) => step_style(state, key),
        Item::Upscaling => super::toggle_upscaling(state),
        Item::UpscalingMode => super::step_upscaling_mode(state, 1),
        Item::Fit => super::step_fit(state, 1, true),
        Item::Stats => super::toggle_stats(state),
        Item::Sleep => {
            player.sleep = (player.sleep + 1) % SLEEP_MINUTES.len();
            player.sleep_at = (SLEEP_MINUTES[player.sleep] > 0).then(|| {
                Instant::now() + Duration::from_secs(u64::from(SLEEP_MINUTES[player.sleep]) * 60)
            });
        }
    }
}

fn step_style(state: &mut RendererState, key: &'static str) {
    let options = fluxa_ui::subtitle_style_rows("en")
        .into_iter()
        .find_map(|(name, _, options)| (name == key).then_some(options))
        .unwrap_or_default();
    let current = state.settings.values.get(key).cloned();
    let value = if options.is_empty() {
        json!(!current.and_then(|value| value.as_bool()).unwrap_or(false))
    } else {
        let at = current
            .as_ref()
            .and_then(Value::as_str)
            .and_then(|current| options.iter().position(|option| *option == current))
            .map_or(0, |at| (at + 1) % options.len());
        json!(options[at])
    };
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(key.to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: key.to_owned(),
            value,
        });
    let style = state.settings.values.clone();
    if let Some(video) = state.video.as_mut() {
        super::apply_subtitle_style(video.as_mut(), &style);
    }
    if let Some(player) = state.player.as_mut() {
        player.style = style;
    }
}

pub(super) fn tick_style(state: &mut RendererState) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let open = matches!(player.panel, Some(Panel::Menu(Menu::Style)));
    if open && !player.style_hidden && player.tracks.iter().any(|t| t.subtitle && t.selected) {
        player.style_hidden = true;
        mpv(state, "sub-visibility", "no");
    } else if !open && player.style_hidden {
        player.style_hidden = false;
        mpv(state, "sub-visibility", "yes");
    }
}

fn mpv(state: &mut RendererState, name: &str, value: &str) {
    command(
        state,
        VideoCommand::Mpv(vec!["set".to_owned(), name.to_owned(), value.to_owned()]),
    );
}

fn select(state: &mut RendererState, selection: TrackSelection) {
    command(state, VideoCommand::SelectTracks(selection));
    if let (Some(video), Some(player)) = (state.video.as_mut(), state.player.as_mut()) {
        player.tracks = video.tracks();
    }
    remember_tracks(state);
}

const MEMORY_KEY: &str = "playerTrackMemory";
const MEMORY_LIMIT: usize = 200;

fn language_of(track: &VideoTrack) -> Option<String> {
    track
        .language
        .as_deref()
        .filter(|code| !code.is_empty())
        .map(str::to_lowercase)
}

fn remember_tracks(state: &mut RendererState) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let Some(id) = player.meta.get("id").and_then(Value::as_str) else {
        return;
    };
    let selected = |subtitle: bool| {
        player
            .tracks
            .iter()
            .find(|track| track.subtitle == subtitle && track.selected)
            .and_then(language_of)
    };
    let entry = json!({
        "audio": selected(false),
        "subtitle": selected(true).unwrap_or_else(|| "off".to_owned()),
    });
    let mut memory = state
        .settings
        .values
        .get(MEMORY_KEY)
        .cloned()
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    if let Some(map) = memory.as_object_mut() {
        if map.len() >= MEMORY_LIMIT && !map.contains_key(id) {
            if let Some(oldest) = map.keys().next().cloned() {
                map.remove(&oldest);
            }
        }
        map.insert(id.to_owned(), entry);
    }
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(MEMORY_KEY.to_owned(), memory.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: MEMORY_KEY.to_owned(),
            value: memory,
        });
}

pub(super) fn restore_tracks(
    player: &mut PlayerSession,
    video: &mut dyn VideoBackend,
    settings: &fluxa_ui::SettingsModel,
) {
    let Some(entry) = player
        .meta
        .get("id")
        .and_then(Value::as_str)
        .and_then(|id| settings.values.get(MEMORY_KEY)?.get(id))
    else {
        return;
    };
    let tracks = video.tracks();
    let pick = |subtitle: bool, language: &str| {
        tracks
            .iter()
            .find(|track| {
                track.subtitle == subtitle && language_of(track).as_deref() == Some(language)
            })
            .map(|track| track.id.clone())
    };
    let mut selection = TrackSelection::default();
    if let Some(language) = entry["audio"].as_str() {
        selection.audio = pick(false, language);
    }
    match entry["subtitle"].as_str() {
        Some("off") => selection.subtitles_off = true,
        Some(language) => selection.subtitle = pick(true, language),
        None => {}
    }
    if selection.audio.is_some() || selection.subtitle.is_some() || selection.subtitles_off {
        video.command(VideoCommand::SelectTracks(selection));
        player.tracks = video.tracks();
    }
}

fn start_episode(state: &mut RendererState, id: &str) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let mut item = player.meta.clone();
    if let Some(fields) = item.as_object_mut() {
        fields.retain(|key, _| !key.starts_with("last") && key != "timeOffset");
        fields.insert("lastVideoId".to_owned(), json!(id));
    }
    close(state);
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
}

pub(super) fn previous(state: &mut RendererState) {
    let Some(id) = state
        .player
        .as_ref()
        .and_then(|player| adjacent(player, -1))
    else {
        return;
    };
    start_episode(state, &id);
}

pub(super) fn tick_sleep(state: &mut RendererState) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    if player.sleep_at.is_some_and(|at| Instant::now() >= at) {
        player.sleep_at = None;
        player.sleep = 0;
        if !player.status.paused {
            command(state, VideoCommand::TogglePause);
        }
    }
}
