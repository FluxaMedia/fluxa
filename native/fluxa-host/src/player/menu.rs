use super::*;
use fluxa_ui::{PanelRow, PlayerPanel, localized};

const SPEEDS: [f64; 7] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const ASPECTS: [&str; 5] = ["-1", "16:9", "4:3", "21:9", "1:1"];
const SLEEP_MINUTES: [u32; 5] = [0, 15, 30, 45, 60];
const DELAY_STEP: f64 = 0.1;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Menu {
    Tracks,
    Speed,
    Episodes,
    Settings,
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
    Sleep,
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

fn track_label(track: &VideoTrack, index: usize, language: &str) -> (String, Option<String>) {
    let code = track.language.as_deref().filter(|code| !code.is_empty());
    match track.title.as_deref().filter(|title| !title.is_empty()) {
        Some(title) => (title.to_owned(), code.map(str::to_uppercase)),
        None => match code {
            Some(code) => (code.to_uppercase(), None),
            None => (
                format!(
                    "{} {}",
                    localized("player.audio_track", language),
                    index + 1
                ),
                None,
            ),
        },
    }
}

fn episodes(player: &PlayerSession) -> Vec<&Value> {
    let mut videos: Vec<&Value> = player
        .meta
        .get("videos")
        .and_then(Value::as_array)
        .map(|videos| videos.iter().collect())
        .unwrap_or_default();
    let key = |video: &Value| {
        let number = |key: &str| video.get(key).and_then(Value::as_i64).unwrap_or(0);
        (
            number("season"),
            video
                .get("episode")
                .and_then(Value::as_i64)
                .unwrap_or_else(|| number("number")),
        )
    };
    videos.retain(|video| key(video).0 > 0);
    videos.sort_by_key(|video| key(video));
    videos
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
        Menu::Tracks => {
            let mut rows = vec![heading(text("player.audio"))];
            for (index, track) in player.tracks.iter().filter(|t| !t.subtitle).enumerate() {
                let (label, value) = track_label(track, index, language);
                rows.push(row(
                    label,
                    value,
                    track.selected,
                    Item::Audio(track.id.clone()),
                ));
            }
            rows.push(heading(text("player.subtitles")));
            let subtitles: Vec<&VideoTrack> = player.tracks.iter().filter(|t| t.subtitle).collect();
            rows.push(row(
                text("player.off"),
                None,
                !subtitles.iter().any(|track| track.selected),
                Item::Subtitle(None),
            ));
            for (index, track) in subtitles.into_iter().enumerate() {
                let (label, value) = track_label(track, index, language);
                rows.push(row(
                    label,
                    value,
                    track.selected,
                    Item::Subtitle(Some(track.id.clone())),
                ));
            }
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
                    Some(row(
                        format!("S{}·E{} {}", number("season"), episode, name),
                        None,
                        player.overlay.current_id() == Some(id),
                        Item::Episode(id.to_owned()),
                    ))
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
            let rows = vec![
                row(
                    text("player.aspect_ratio"),
                    Some(aspect),
                    false,
                    Item::Aspect,
                ),
                row(
                    text("player.audio_delay_earlier"),
                    Some(delay(player.audio_delay)),
                    false,
                    Item::AudioDelay(-DELAY_STEP),
                ),
                row(
                    text("player.audio_delay_later"),
                    Some(delay(player.audio_delay)),
                    false,
                    Item::AudioDelay(DELAY_STEP),
                ),
                row(
                    text("player.subtitle_delay_earlier"),
                    Some(delay(player.subtitle_delay)),
                    false,
                    Item::SubtitleDelay(-DELAY_STEP),
                ),
                row(
                    text("player.subtitle_delay_later"),
                    Some(delay(player.subtitle_delay)),
                    false,
                    Item::SubtitleDelay(DELAY_STEP),
                ),
                row(text("player.sleep_timer"), Some(sleep), false, Item::Sleep),
            ];
            (text("player.settings"), rows)
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
    }
}

pub(super) fn open(state: &mut RendererState, menu: Menu) {
    if matches!(menu, Menu::Tracks)
        && let Some(video) = state.video.as_mut()
        && let Some(player) = state.player.as_mut()
    {
        player.tracks = video.tracks();
    }
    if let Some(player) = state.player.as_mut() {
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
        Item::Sleep => {
            player.sleep = (player.sleep + 1) % SLEEP_MINUTES.len();
            player.sleep_at = (SLEEP_MINUTES[player.sleep] > 0).then(|| {
                Instant::now() + Duration::from_secs(u64::from(SLEEP_MINUTES[player.sleep]) * 60)
            });
        }
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
