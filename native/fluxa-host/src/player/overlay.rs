use std::sync::mpsc::Receiver;
use std::time::Duration;

use fluxa_ui::{ChapterSpan, NextEpisodeCard, SegmentSpan, SkipCard, SkipKind};
use serde_json::{Map, Value, json};
use web_time::Instant;

use super::{PlayerSession, TrackSelection, VideoBackend, VideoCommand, VideoTrack};
use crate::{SessionHandle, core_value, host_log, profile_language};

const PLAN_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Default)]
pub(super) struct Overlay {
    plan: Value,
    planned_at: Option<Instant>,
    prefs: Option<Value>,
    segments: Vec<Value>,
    lookup: Option<Lookup>,
    next_episode: Option<Value>,
    current_id: Option<String>,
    next_resolved: bool,
    next_dismissed: bool,
    next_shown_at: Option<Instant>,
    next_prefetched: bool,
    addon_subtitles_request: Option<Receiver<Option<Value>>>,
    addon_subtitles_started: bool,
    pub(super) addon_subtitles: Vec<Value>,
    dismissed: Vec<String>,
    tracks_applied: bool,
    last_scrobble: Option<String>,
    last_saved_at_ms: i64,
}

struct Lookup {
    ctx: Value,
    providers: Value,
    responses: Map<String, Value>,
    inflight: Vec<(String, Receiver<Option<Value>>)>,
    done: bool,
}

impl Overlay {
    pub(super) fn scrobbling(&self) -> bool {
        self.last_scrobble.is_some()
    }

    pub(super) fn dismiss_next(&mut self) {
        self.next_dismissed = true;
    }

    pub(super) fn dismiss_skip(&mut self) {
        if let Some(kind) = self.plan.pointer("/skip/kind").and_then(Value::as_str) {
            self.dismissed.push(kind.to_owned());
        }
    }

    pub(super) fn skip(&self) -> Option<&Value> {
        self.plan.get("skip").filter(|skip| !skip.is_null())
    }

    pub(super) fn skip_target(&self) -> Option<f64> {
        let millis = self.plan.pointer("/skip/seekToMs")?.as_i64()?;
        Some(millis as f64 / 1000.0)
    }

    pub(super) fn current_id(&self) -> Option<&str> {
        self.current_id.as_deref()
    }

    pub(super) fn upcoming(&self) -> Option<&Value> {
        self.next_episode.as_ref()
    }

    pub(super) fn seek_seconds(&self) -> Option<f64> {
        self.prefs.as_ref()?["seekSeconds"].as_f64()
    }

    pub(super) fn next_video(&self) -> Option<&Value> {
        self.next_episode
            .as_ref()
            .filter(|_| self.card_visible_next())
    }

    fn card_visible_next(&self) -> bool {
        self.plan.get("next").is_some_and(|next| !next.is_null())
    }

    pub(super) fn card_node(&self) -> Option<u64> {
        if self.plan.get("skip").is_some_and(|skip| !skip.is_null()) {
            Some(fluxa_ui::NODE_PLAYER_SKIP)
        } else if self.card_visible_next() {
            Some(fluxa_ui::NODE_PLAYER_NEXT_PLAY)
        } else {
            None
        }
    }

    pub(super) fn chapter_spans(&self) -> Vec<ChapterSpan> {
        let spans = self.plan.get("chapters").and_then(Value::as_array);
        spans
            .into_iter()
            .flatten()
            .map(|span| ChapterSpan {
                start_fraction: span["startFraction"].as_f64().unwrap_or(0.0) as f32,
                end_fraction: span["endFraction"].as_f64().unwrap_or(1.0) as f32,
                title: span["title"].as_str().unwrap_or_default().to_owned(),
            })
            .collect()
    }

    pub(super) fn segment_spans(&self) -> Vec<SegmentSpan> {
        let spans = self.plan.get("segments").and_then(Value::as_array);
        spans
            .into_iter()
            .flatten()
            .filter_map(|span| {
                let kind = match span["kind"].as_str()? {
                    "intro" => SkipKind::Intro,
                    "recap" => SkipKind::Recap,
                    "outro" => SkipKind::Outro,
                    _ => return None,
                };
                Some(SegmentSpan {
                    start_fraction: span["startFraction"].as_f64()? as f32,
                    end_fraction: span["endFraction"].as_f64()? as f32,
                    kind,
                })
            })
            .collect()
    }

    pub(super) fn skip_card(&self, position: f64) -> Option<SkipCard> {
        let skip = self.plan.get("skip").filter(|skip| !skip.is_null())?;
        let kind = match skip.get("kind")?.as_str()? {
            "intro" => SkipKind::Intro,
            "recap" => SkipKind::Recap,
            "outro" => SkipKind::Outro,
            _ => return None,
        };
        Some(SkipCard {
            kind,
            seek_to: skip.get("seekToMs")?.as_i64()? as f64 / 1000.0,
            progress: {
                let start = skip["startMs"].as_f64().unwrap_or(0.0) / 1000.0;
                let end = skip["endMs"].as_f64().unwrap_or(0.0) / 1000.0;
                if end > start {
                    ((position - start) / (end - start)).clamp(0.0, 1.0) as f32
                } else {
                    0.0
                }
            },
        })
    }

    pub(super) fn next_card(&self) -> Option<NextEpisodeCard> {
        let next = self.plan.get("next").filter(|next| !next.is_null())?;
        let label = match (next["season"].as_i64(), next["episode"].as_i64()) {
            (Some(season), Some(episode)) => format!("S{season}:E{episode}"),
            (None, Some(episode)) => format!("E{episode}"),
            _ => String::new(),
        };
        let countdown = next["autoPlay"]
            .as_bool()
            .unwrap_or(false)
            .then(|| self.countdown_left(next))
            .flatten();
        Some(NextEpisodeCard {
            label,
            title: next["title"].as_str().unwrap_or_default().to_owned(),
            thumbnail: next["thumbnail"].as_str().map(ToOwned::to_owned),
            countdown,
        })
    }

    fn countdown_left(&self, next: &Value) -> Option<u32> {
        let total = next["countdownSecs"].as_u64()? as f32;
        let elapsed = self.next_shown_at?.elapsed().as_secs_f32();
        Some((total - elapsed).ceil().max(0.0) as u32)
    }

    fn countdown_done(&self) -> bool {
        self.plan
            .get("next")
            .filter(|next| next["autoPlay"].as_bool() == Some(true))
            .is_some_and(|next| self.countdown_left(next) == Some(0))
    }
}

fn now_ms() -> i64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

fn without_nulls(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, value)| !value.is_null())
                .collect::<Map<_, _>>(),
        ),
        other => other,
    }
}

fn current_video<'a>(meta: &'a Value, snapshot: &Value) -> Option<&'a Value> {
    let id = snapshot.pointer("/player/currentVideoId")?.as_str()?;
    meta.get("videos")?
        .as_array()?
        .iter()
        .find(|video| video.get("id").and_then(Value::as_str) == Some(id))
}

fn episode_numbers(video: &Value) -> (i64, i64) {
    let number = |key: &str| video.get(key).and_then(Value::as_i64);
    (
        number("season").unwrap_or(0),
        number("episode").or_else(|| number("number")).unwrap_or(0),
    )
}

fn resolve_next(meta: &Value, snapshot: &Value) -> Option<Value> {
    let videos = meta.get("videos")?;
    let (season, episode) = episode_numbers(current_video(meta, snapshot)?);
    core_value(
        "resolveNextEpisode",
        json!({
            "videos": videos,
            "currentSeason": season,
            "currentEpisode": episode,
            "nowMs": now_ms(),
            "releasedOnly": true,
        }),
    )
    .filter(|next| !next.is_null())
}

pub(super) fn lookup_context(player: &PlayerSession, snapshot: &Value) -> Option<Value> {
    let meta = &player.meta;
    let video = current_video(meta, snapshot);
    let text = |key: &str| meta.get(key).and_then(Value::as_str);
    let imdb = video
        .and_then(|video| video.get("id"))
        .and_then(Value::as_str)
        .or_else(|| text("id"))
        .and_then(|id| id.split(':').next())
        .filter(|id| id.starts_with("tt"))
        .map(str::to_owned);
    let tmdb = ["moviedb_id", "tmdbId", "tmdb_id"]
        .iter()
        .find_map(|key| meta.get(*key).and_then(Value::as_i64));
    if imdb.is_none() && tmdb.is_none() {
        return None;
    }
    let series = text("type") == Some("series");
    let (season, episode) = video.map(episode_numbers).unzip();
    Some(json!({
        "imdbId": imdb,
        "tmdbId": tmdb,
        "mediaType": if series { "tv" } else { "movie" },
        "season": season.filter(|_| series),
        "episode": episode.filter(|_| series),
        "title": text("name"),
        "durationMs": (player.status.duration * 1000.0) as i64,
        "anime": core_value("shouldAttemptAnimeTracking", meta.clone())
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
    }))
}

fn advance_lookup(player: &mut PlayerSession, session: &SessionHandle, snapshot: &Value) {
    if player.overlay.lookup.is_none() {
        let Some(ctx) = lookup_context(player, snapshot) else {
            player.overlay.lookup = Some(Lookup {
                ctx: Value::Null,
                providers: Value::Null,
                responses: Map::new(),
                inflight: Vec::new(),
                done: true,
            });
            return;
        };
        let mut providers = snapshot
            .pointer("/profile/active/segmentProviders")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let prefs = player.overlay.prefs.as_ref();
        let client_id = prefs
            .and_then(|prefs| prefs["animeSkipClientId"].as_str())
            .filter(|id| !id.is_empty());
        if prefs.is_some_and(|prefs| prefs["useAnimeSkip"].as_bool() != Some(false))
            && let Some(client_id) = client_id
        {
            providers["animeSkipClientId"] = json!(client_id);
        }
        player.overlay.lookup = Some(Lookup {
            ctx,
            providers,
            responses: Map::new(),
            inflight: Vec::new(),
            done: false,
        });
        step_lookup(&mut player.overlay, session);
        return;
    }
    let overlay = &mut player.overlay;
    let Some(lookup) = overlay.lookup.as_mut().filter(|lookup| !lookup.done) else {
        return;
    };
    let mut arrived = false;
    lookup.inflight.retain(|(id, receiver)| {
        let response = match receiver.try_recv() {
            Ok(response) => response,
            Err(std::sync::mpsc::TryRecvError::Empty) => return true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
        };
        lookup
            .responses
            .insert(id.clone(), response.unwrap_or(Value::Null));
        arrived = true;
        false
    });
    if arrived {
        step_lookup(overlay, session);
    }
}

fn step_lookup(overlay: &mut Overlay, session: &SessionHandle) {
    let Some(lookup) = overlay.lookup.as_mut() else {
        return;
    };
    let step = core_value(
        "playerSegmentsStep",
        json!({
            "ctx": lookup.ctx,
            "providers": lookup.providers,
            "responses": lookup.responses,
        }),
    );
    let Some(step) = step else {
        lookup.done = true;
        return;
    };
    if let Some(Value::Array(segments)) = step.get("segments").cloned() {
        overlay.segments = segments;
    }
    let requests = step["requests"].as_array().cloned().unwrap_or_default();
    lookup.done = requests.is_empty() && lookup.inflight.is_empty();
    for request in requests {
        let Some(id) = request["id"].as_str().map(str::to_owned) else {
            continue;
        };
        if lookup.inflight.iter().any(|(pending, _)| *pending == id) {
            continue;
        }
        lookup.inflight.push((id, session.request_json(request)));
    }
}

fn apply_tracks(player: &mut PlayerSession, video: &mut dyn VideoBackend, profile: &Value) {
    let tracks = video.tracks();
    if !tracks.iter().any(|track| !track.subtitle) {
        return;
    }
    player.overlay.tracks_applied = true;
    let audio: Vec<Value> = tracks
        .iter()
        .filter(|track| !track.subtitle)
        .map(|track| {
            json!({"id": track.id, "language": track.language, "isSelected": track.selected})
        })
        .collect();
    let subtitles: Vec<Value> = tracks
        .iter()
        .filter(|track| track.subtitle)
        .map(|track| {
            json!({
                "id": track.id,
                "label": track.title.clone().unwrap_or_default(),
                "language": track.language,
            })
        })
        .collect();
    let field = |key: &str| profile.get(key).cloned().unwrap_or(Value::Null);
    let meta = &player.meta;
    let Some(plan) = core_value(
        "playerTrackPlan",
        json!({
            "audioTracks": audio,
            "subtitleTracks": subtitles,
            "prefs": {
                "preferredAudioLanguage": field("preferredAudioLanguage"),
                "secondaryAudioLanguage": field("secondaryAudioLanguage"),
                "preferredSubtitleLanguage": field("preferredSubtitleLanguage"),
                "secondarySubtitleLanguage": field("secondarySubtitleLanguage"),
                "autoEnableSubtitles": field("autoEnableSubtitles"),
            },
            "lastAudioLanguage": meta.get("lastAudioLanguage"),
            "lastSubtitleLanguage": meta.get("lastSubtitleLanguage"),
            "originalLanguage": meta.get("originalLanguage"),
        }),
    ) else {
        return;
    };
    let id = |key: &str| plan.get(key).and_then(Value::as_str).map(ToOwned::to_owned);
    video.command(VideoCommand::SelectTracks(TrackSelection {
        audio: id("audioId"),
        subtitle: id("subtitleId"),
        secondary_subtitle: id("secondarySubtitleId"),
        subtitles_off: plan["disableSubtitles"].as_bool().unwrap_or(false),
    }));
}

fn refresh_plan(player: &mut PlayerSession) {
    let overlay = &player.overlay;
    if overlay
        .planned_at
        .is_some_and(|at| at.elapsed() < PLAN_INTERVAL)
    {
        return;
    }
    let chapters: Vec<Value> = player
        .status
        .chapters
        .iter()
        .map(|(start, title)| json!({"startMs": (start * 1000.0) as i64, "title": title}))
        .collect();
    let next = overlay.next_episode.as_ref().map(|video| {
        json!({
            "title": video.get("name").or_else(|| video.get("title")),
            "season": video.get("season"),
            "episode": video.get("episode").or_else(|| video.get("number")),
            "thumbnail": video.get("thumbnail"),
        })
    });
    let plan = core_value(
        "playerOverlayPlan",
        json!({
            "positionMs": (player.status.position * 1000.0) as i64,
            "durationMs": (player.status.duration * 1000.0) as i64,
            "chapters": chapters,
            "segments": overlay.segments,
            "nextEpisode": next,
            "prefs": overlay.prefs,
            "dismissed": overlay.dismissed,
            "nextDismissed": overlay.next_dismissed,
        }),
    );
    let overlay = &mut player.overlay;
    overlay.planned_at = Some(Instant::now());
    overlay.plan = plan.unwrap_or(Value::Null);
    let showing = overlay.card_visible_next();
    match (showing, overlay.next_shown_at) {
        (true, None) => overlay.next_shown_at = Some(Instant::now()),
        (false, Some(_)) => overlay.next_shown_at = None,
        _ => {}
    }
}

fn tick_scrobble(player: &mut PlayerSession, session: &SessionHandle, snapshot: &Value) {
    let now = now_ms();
    let plan = core_value(
        "playerTickPlan",
        json!({
            "paused": player.status.paused,
            "hasFrame": player.status.has_frame,
            "positionMs": (player.status.position * 1000.0) as i64,
            "durationMs": (player.status.duration * 1000.0) as i64,
            "nowMs": now,
            "lastSavedAtMs": player.overlay.last_saved_at_ms,
            "lastScrobble": player.overlay.last_scrobble,
        }),
    );
    let Some(plan) = plan else {
        return;
    };
    if let Some(action) = plan["scrobble"].as_str() {
        player.overlay.last_scrobble = Some(action.to_owned());
        super::scrobble(session, player, snapshot, action);
    }
    if plan["saveProgress"].as_bool() == Some(true) {
        player.overlay.last_saved_at_ms = now;
        super::save_progress(session, player, snapshot);
    }
}

pub(super) fn tick(
    player: &mut PlayerSession,
    session: &SessionHandle,
    video: &mut dyn VideoBackend,
    snapshot: &Value,
) -> bool {
    let profile = snapshot.pointer("/profile/active").unwrap_or(&Value::Null);
    if player.overlay.prefs.is_none() {
        player.overlay.prefs =
            core_value("playbackPreferencesPlan", profile.clone()).map(without_nulls);
    }
    if !player.status.has_frame {
        return false;
    }
    if player.overlay.current_id.is_none() {
        player.overlay.current_id = snapshot
            .pointer("/player/currentVideoId")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    if !player.overlay.next_resolved && player.status.duration > 0.0 {
        player.overlay.next_resolved = true;
        player.overlay.next_episode = resolve_next(&player.meta, snapshot);
    }
    let wants_segments = player
        .overlay
        .prefs
        .as_ref()
        .and_then(|prefs| prefs["useSkipSegments"].as_bool())
        .unwrap_or(true);
    if wants_segments {
        advance_lookup(player, session, snapshot);
    }
    if !player.overlay.tracks_applied {
        apply_tracks(player, video, profile);
    }
    load_addon_subtitles(player, session, video, snapshot, profile);
    tick_scrobble(player, session, snapshot);
    refresh_plan(player);
    prefetch_next_streams(player, session, snapshot);
    auto_skip(player, video);
    player.overlay.countdown_done()
}

fn load_addon_subtitles(
    player: &mut PlayerSession,
    session: &SessionHandle,
    video: &mut dyn VideoBackend,
    snapshot: &Value,
    profile: &Value,
) {
    let overlay = &mut player.overlay;
    if !overlay.addon_subtitles_started {
        overlay.addon_subtitles_started = true;
        let id = snapshot
            .pointer("/player/currentVideoId")
            .or_else(|| player.meta.get("id"))
            .and_then(Value::as_str);
        let content_type = player.meta.get("type").and_then(Value::as_str);
        if let (Some(id), Some(content_type)) = (id, content_type) {
            overlay.addon_subtitles_request =
                Some(session.fetch_addon_subtitles(content_type.to_owned(), id.to_owned()));
        }
        return;
    }
    let Some(receiver) = overlay.addon_subtitles_request.as_ref() else {
        return;
    };
    let tracks = match receiver.try_recv() {
        Ok(tracks) => tracks,
        Err(std::sync::mpsc::TryRecvError::Empty) => return,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
    };
    overlay.addon_subtitles_request = None;
    let Some(Value::Array(tracks)) = tracks else {
        return;
    };
    let candidates: Vec<Value> = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            json!({
                "id": index.to_string(),
                "label": track["label"],
                "language": track["lang"],
            })
        })
        .collect();
    overlay.addon_subtitles = tracks;
    let field = |key: &str| profile.get(key).cloned().unwrap_or(Value::Null);
    let Some(plan) = core_value(
        "playerTrackPlan",
        json!({
            "audioTracks": [],
            "subtitleTracks": candidates,
            "prefs": {
                "preferredSubtitleLanguage": field("preferredSubtitleLanguage"),
                "secondarySubtitleLanguage": field("secondarySubtitleLanguage"),
                "autoEnableSubtitles": field("autoEnableSubtitles"),
            },
            "lastSubtitleLanguage": player.meta.get("lastSubtitleLanguage"),
        }),
    ) else {
        return;
    };
    let embedded_selected = video
        .tracks()
        .iter()
        .any(|track| track.subtitle && track.selected);
    for (key, select) in [
        ("subtitleId", !embedded_selected),
        ("secondarySubtitleId", false),
    ] {
        let Some(track) = plan[key]
            .as_str()
            .and_then(|id| id.parse::<usize>().ok())
            .and_then(|index| player.overlay.addon_subtitles.get(index))
        else {
            continue;
        };
        let Some(url) = track["url"].as_str() else {
            continue;
        };
        let mut args = vec![
            "sub-add".to_owned(),
            url.to_owned(),
            if select { "select" } else { "auto" }.to_owned(),
            track["label"].as_str().unwrap_or("Subtitle").to_owned(),
        ];
        if let Some(lang) = track["lang"].as_str() {
            args.push(lang.to_owned());
        }
        video.command(VideoCommand::Mpv(args));
    }
}

fn prefetch_next_streams(player: &mut PlayerSession, session: &SessionHandle, snapshot: &Value) {
    let overlay = &mut player.overlay;
    if overlay.next_prefetched || overlay.next_shown_at.is_none() {
        return;
    }
    let Some(next) = overlay.next_episode.as_ref() else {
        return;
    };
    let Some(next_video_id) = next["id"].as_str() else {
        return;
    };
    overlay.next_prefetched = true;
    let meta = &player.meta;
    let profile = snapshot.pointer("/profile/active").unwrap_or(&Value::Null);
    let command = json!({
        "type": "playerNextEpisodeCardShown",
        "contentType": meta.get("type").and_then(Value::as_str).unwrap_or("series"),
        "seriesId": meta.get("id"),
        "nextVideoId": next_video_id,
        "title": meta.get("name").or_else(|| meta.get("title")),
        "originalName": meta.get("originalName"),
        "year": meta.get("year"),
        "language": profile_language(profile),
        "profile": profile,
    });
    if let Err(error) = session.dispatch(command) {
        host_log(format!("core dispatch failed: {error}"));
    }
}

fn auto_skip(player: &mut PlayerSession, video: &mut dyn VideoBackend) {
    let skip = player
        .overlay
        .plan
        .get("skip")
        .filter(|skip| !skip.is_null());
    let Some(skip) = skip.filter(|skip| skip["auto"].as_bool() == Some(true)) else {
        return;
    };
    let Some(target) = skip["seekToMs"].as_i64() else {
        return;
    };
    player.toast_skip();
    player.overlay.dismiss_skip();
    video.command(VideoCommand::SeekTo(target as f64 / 1000.0));
}

pub(super) fn finish(
    player: &PlayerSession,
    session: &SessionHandle,
    snapshot: &Value,
    tracks: &[VideoTrack],
) {
    if !player.status.has_frame {
        return;
    }
    let stream = player
        .chosen
        .and_then(|index| player.sources.as_ref()?.get(index));
    let plan = core_value(
        "playbackClosePlan",
        json!({
            "meta": player.meta,
            "episode": current_video(&player.meta, snapshot),
            "stream": stream,
            "streamIndex": player.chosen,
            "nextEpisode": player.overlay.next_episode,
            "timePos": player.status.position,
            "duration": player.status.duration,
            "playbackStarted": true,
            "prefs": snapshot.pointer("/profile/active"),
            "scrobbleTraktPause": true,
        }),
    );
    let Some(plan) = plan else {
        return;
    };
    let selected = |subtitle: bool| {
        tracks
            .iter()
            .find(|track| track.selected && track.subtitle == subtitle)
            .and_then(|track| track.language.clone())
    };
    for (key, remembers_tracks) in [
        ("progressAction", true),
        ("markWatchedAction", false),
        ("upNextAction", false),
    ] {
        let Some(mut action) = plan.get(key).filter(|action| !action.is_null()).cloned() else {
            continue;
        };
        action["profile"] = session.active_profile();
        if remembers_tracks {
            action["lastAudioLanguage"] = json!(selected(false));
            action["lastSubtitleLanguage"] =
                json!(selected(true).unwrap_or_else(|| "__off__".into()));
        }
        if let Err(error) = session.dispatch(action) {
            host_log(format!("core dispatch failed: {error}"));
        }
    }
}
