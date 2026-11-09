use super::*;

pub(crate) struct PlayerSession {
    pub(crate) stale: Option<Value>,
    pub(super) stale_until: Instant,
    pub(super) snapshot_logo: Option<String>,
    pub(super) logo_requested: bool,
    pub(super) meta: Value,
    pub(super) local: bool,
    pub(super) started: bool,
    pub(super) loaded_url: Option<String>,
    pub(super) failed_url: Option<String>,
    pub(crate) cast_target: Option<crate::cast::Device>,
    pub(crate) texture: Option<egui::TextureId>,
    pub(crate) had_frame: bool,
    pub(super) last_activity: Instant,
    pub(crate) locked: bool,
    pub(super) style: Value,
    pub(super) style_hidden: bool,
    pub(super) sub_pos: Option<String>,
    pub(super) sync: subtitle_sync::SyncState,
    pub(super) external_audio: external_audio::ExternalAudio,
    pub(super) torrent_link: Option<String>,
    pub(super) torrent_since: Option<Instant>,
    pub(super) torrent_rx: Option<Receiver<Value>>,
    pub(super) torrent_status: Option<Value>,
    pub(super) error: Option<String>,
    pub(crate) status: VideoStatus,
    pub(super) language: String,
    pub(super) description: Option<String>,
    pub(super) episode_title: Option<String>,
    pub(super) episode_image: Option<String>,
    pub(super) season_episode: Option<(i64, i64)>,
    pub(super) synopsis: Option<String>,
    pub(super) stream_addons: Vec<String>,
    pub(super) pending_addons: Vec<String>,
    pub(super) warnings_rx: Option<Receiver<Option<Value>>>,
    pub(super) warnings_requested: bool,
    pub(super) warnings: Vec<(String, String)>,
    pub(super) warnings_clock: Option<f32>,
    pub(super) last_pump: Instant,
    pub(super) thumbnail: Option<(f64, egui::TextureHandle)>,
    pub(super) thumbnail_requested: Option<(f64, Instant)>,
    pub(super) filmstrip: filmstrip::Filmstrip,
    pub(super) trickplay: Option<trickplay::Trickplay>,
    pub(super) seek_previews: Option<seek_previews::SeekPreviews>,
    pub(super) outro_reached: bool,
    pub(super) recommendations_rx: Option<Receiver<Vec<Value>>>,
    pub(super) recommendations: Vec<Value>,
    pub(super) recommendation_index: usize,
    pub(super) load_progress: f32,
    pub(super) scrub: Option<(f64, Instant)>,
    pub(super) scrub_streak: u32,
    pub(super) passthrough: bool,
    pub(super) subtitle: Option<String>,
    pub(super) dispatched: Option<Value>,
    pub(super) overlay: Overlay,
    pub(super) panel: Option<Panel>,
    pub(crate) cast: crate::cast::Cast,
    pub(super) trailer: Option<trailer::TrailerPlay>,
    pub(super) sources: Option<Vec<Value>>,
    pub(super) episodes: std::sync::OnceLock<Vec<Value>>,
    pub(super) source_rows:
        std::sync::Mutex<Option<(SourceRowsKey, std::sync::Arc<Vec<fluxa_ui::PlayerSource>>)>>,
    pub(super) opened_ms: u128,
    pub(super) feed_version: u64,
    pub(super) stream_parts: std::collections::BTreeMap<usize, Vec<Value>>,
    pub(super) streams_requested: bool,
    pub(super) streams_done: bool,
    pub(super) chosen: Option<usize>,
    pub(super) source_filter: Option<String>,
    pub(crate) source_chips_offset: f32,
    pub(super) manual: bool,
    pub(super) toast: Option<(Value, Instant)>,
    pub(super) seek_run: Option<(f64, Instant)>,
    pub(super) media_sent: Option<(Instant, String, f64)>,
    pub(super) brightness: f32,
    pub(super) speed_held: bool,
    pub(crate) speed: f64,
    pub(super) tracks: Vec<VideoTrack>,
    pub(super) track_lang: Option<(bool, String)>,
    pub(super) aspect: usize,
    pub(super) audio_delay: f64,
    pub(super) subtitle_delay: f64,
    pub(super) sleep: usize,
    pub(super) sleep_at: Option<Instant>,
    pub(super) tracks_restored: bool,
    pub(super) recovery: recover::Recovery,
    pub(super) resume_at: Option<f64>,
    pub(super) start_paused: bool,
    pub(crate) anime4k: bool,
    pub(crate) upscaling: String,
    pub(crate) fit: &'static str,
    pub(crate) compact: bool,
    pub(super) volume_pin: Option<f64>,
    pub(crate) stats_visible: bool,
    pub(super) stats: Vec<fluxa_ui::StatsSection>,
    pub(super) stats_at: Option<Instant>,
    pub(super) last_stats: Option<PlaybackStats>,
    pub(super) playing_badges: Vec<fluxa_ui::SourceBadge>,
    pub(crate) minimized: bool,
}

pub(super) enum Panel {
    Submit(submit::Submit),
    Cast,
    Menu(menu::Menu),
}

impl PlayerSession {
    pub(crate) fn trailer(meta: Value, urls: &[String]) -> Self {
        let mut player = Self::new(meta);
        player.trailer = Some(trailer::TrailerPlay::new(urls));
        player.started = true;
        player
    }

    pub(super) fn model_logo_from_meta(&self) -> Option<&str> {
        ["logo", "logoUrl", "clearLogo", "titleLogo", "titleLogoUrl"]
            .iter()
            .find_map(|key| {
                self.meta
                    .get(*key)
                    .and_then(Value::as_str)
                    .filter(|url| !url.trim().is_empty())
            })
    }

    pub(crate) fn with_logo_hint(mut self, logo: Option<String>) -> Self {
        if logo.is_some() {
            self.snapshot_logo = logo;
        }
        self
    }

    pub(crate) fn new(meta: Value) -> Self {
        Self {
            stale: None,
            stale_until: Instant::now() + STALE_GRACE,
            snapshot_logo: None,
            logo_requested: false,
            meta,
            local: false,
            started: false,
            loaded_url: None,
            failed_url: None,
            cast_target: None,
            texture: None,
            had_frame: false,
            last_activity: Instant::now(),
            locked: false,
            style: Value::Null,
            style_hidden: false,
            sub_pos: None,
            sync: subtitle_sync::SyncState::default(),
            external_audio: external_audio::ExternalAudio::default(),
            torrent_link: None,
            torrent_since: None,
            torrent_rx: None,
            torrent_status: None,
            error: None,
            status: VideoStatus::default(),
            language: "en".to_owned(),
            description: None,
            episode_title: None,
            episode_image: None,
            season_episode: None,
            synopsis: None,
            stream_addons: Vec::new(),
            pending_addons: Vec::new(),
            warnings_rx: None,
            warnings_requested: false,
            warnings: Vec::new(),
            warnings_clock: None,
            last_pump: Instant::now(),
            thumbnail: None,
            thumbnail_requested: None,
            filmstrip: filmstrip::Filmstrip::default(),
            trickplay: None,
            seek_previews: None,
            outro_reached: false,
            recommendations_rx: None,
            recommendations: Vec::new(),
            passthrough: false,
            subtitle: None,
            dispatched: None,
            recommendation_index: 0,
            load_progress: 0.0,
            sources: None,
            episodes: std::sync::OnceLock::new(),
            source_rows: std::sync::Mutex::new(None),
            opened_ms: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_millis()),
            feed_version: 0,
            stream_parts: std::collections::BTreeMap::new(),
            streams_requested: false,
            streams_done: false,
            chosen: None,
            source_filter: None,
            source_chips_offset: 0.0,
            manual: false,
            scrub: None,
            scrub_streak: 0,
            overlay: Overlay::default(),
            panel: None,
            cast: crate::cast::Cast::default(),
            trailer: None,
            toast: None,
            seek_run: None,
            media_sent: None,
            brightness: 1.0,
            speed_held: false,
            speed: 1.0,
            tracks: Vec::new(),
            track_lang: None,
            aspect: 0,
            audio_delay: 0.0,
            subtitle_delay: 0.0,
            sleep: 0,
            sleep_at: None,
            tracks_restored: false,
            recovery: recover::Recovery::new(),
            resume_at: None,
            start_paused: false,
            anime4k: false,
            upscaling: String::new(),
            fit: "fit",
            compact: false,
            volume_pin: None,
            stats_visible: false,
            stats: Vec::new(),
            stats_at: None,
            last_stats: None,
            playing_badges: Vec::new(),
            minimized: false,
        }
    }

    pub(super) fn title(&self) -> String {
        self.meta
            .get("name")
            .or_else(|| self.meta.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("Fluxa")
            .to_owned()
    }

    pub(crate) fn start_offset_ms(&self) -> i64 {
        self.meta
            .get("timeOffset")
            .and_then(Value::as_f64)
            .filter(|offset| *offset > 5.0)
            .map_or(0, |offset| (offset * 1000.0) as i64)
    }

    pub(crate) fn is_playing(&self, item: &Value) -> bool {
        if self.local || self.trailer.is_some() {
            return false;
        }
        let id = |meta: &Value| meta.get("id").and_then(Value::as_str).map(str::to_owned);
        let video = |meta: &Value| {
            ["lastVideoId", "videoId", "video_id"]
                .iter()
                .find_map(|key| meta.get(*key).and_then(Value::as_str))
                .map(str::to_owned)
        };
        id(&self.meta).is_some() && id(&self.meta) == id(item) && video(&self.meta) == video(item)
    }

    pub(crate) fn watch_content(&self) -> Option<Value> {
        if self.local || self.trailer.is_some() {
            return None;
        }
        let id = self.meta.get("id").and_then(Value::as_str)?;
        let video = ["lastVideoId", "videoId", "video_id"]
            .iter()
            .find_map(|key| self.meta.get(*key).and_then(Value::as_str));
        let season_episode = self.season_episode.or_else(|| {
            let mut parts = video?.rsplit(':');
            let episode = parts.next()?.parse().ok()?;
            Some((parts.next()?.parse().ok()?, episode))
        });
        let artwork = json!({
            "backdrop": self.meta.get("background").and_then(Value::as_str),
            "logo": self.model_logo_from_meta().or(self.snapshot_logo.as_deref()),
            "episodeTitle": self.episode_title,
            "season": season_episode.map(|(season, _)| season),
            "episode": season_episode.map(|(_, episode)| episode),
        });
        Some(json!({
            "id": id.split(':').next().unwrap_or(id),
            "contentType": self.meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
            "videoId": video,
            "title": self.title(),
            "artwork": artwork,
        }))
    }

    pub(crate) fn controls_visible(&self) -> bool {
        !self.status.has_frame
            || self.panel.is_some()
            || self.last_activity.elapsed() < CONTROLS_TIMEOUT
    }

    pub(super) fn hide(&mut self) {
        self.scrub = None;
        if let Some(past) = Instant::now().checked_sub(CONTROLS_TIMEOUT) {
            self.last_activity = past;
        }
    }

    pub(crate) fn seek_step(&self) -> f64 {
        self.overlay.seek_seconds().unwrap_or(DEFAULT_SEEK_STEP)
    }

    pub(super) fn scrub(&mut self, direction: f64) {
        let (base, streak) = match self.scrub {
            Some((time, at)) if at.elapsed() < SCRUB_COMMIT => (time, self.scrub_streak + 1),
            _ => (self.status.position, 0),
        };
        self.scrub_streak = streak;
        let step = self.seek_step() * (1 + streak / 4).min(6) as f64;
        let target = (base + direction * step).clamp(0.0, self.status.duration.max(0.0));
        self.scrub = Some((target, Instant::now()));
        self.touch();
    }

    pub(crate) fn toast(&mut self, kind: &str, value: f64) {
        if let Some(plan) = core_value("playerToastPlan", json!({"kind": kind, "value": value})) {
            self.toast = Some((plan, Instant::now()));
        }
    }

    pub(crate) fn toast_seek(&mut self, delta: f64) {
        let total = match self.seek_run {
            Some((total, at)) if at.elapsed() < SEEK_RUN && total * delta > 0.0 => total + delta,
            _ => delta,
        };
        self.seek_run = Some((total, Instant::now()));
        self.toast("seek", total);
    }

    pub(crate) fn toast_skip(&mut self) {
        let Some(skip) = self.overlay.skip() else {
            return;
        };
        let input = json!({
            "kind": "skip",
            "segment": skip["kind"],
            "label": skip["provider"].as_str().unwrap_or("Chapters"),
        });
        if let Some(plan) = core_value("playerToastPlan", input) {
            self.toast = Some((plan, Instant::now()));
        }
    }

    pub(super) fn toast_model(&self) -> Option<fluxa_ui::PlayerToast> {
        let (plan, at) = self.toast.as_ref()?;
        let hold = Duration::from_millis(plan["holdMs"].as_u64().unwrap_or(1200));
        let elapsed = at.elapsed();
        if elapsed >= hold + TOAST_FADE {
            return None;
        }
        let opacity = match elapsed.checked_sub(hold) {
            Some(fade) => 1.0 - fade.as_secs_f32() / TOAST_FADE.as_secs_f32(),
            None => 1.0,
        };
        let text = fluxa_ui::localized(plan["key"].as_str()?, &self.language)
            .replace("%s", plan["value"].as_str().unwrap_or_default());
        Some(fluxa_ui::PlayerToast {
            text,
            level: plan["level"].as_f64().map(|level| level as f32),
            opacity,
            gauge: match plan["key"].as_str()? {
                "player.toast.brightness" => Some(fluxa_ui::ToastGauge::Brightness),
                "player.toast.volume" => Some(fluxa_ui::ToastGauge::Volume),
                _ => None,
            },
        })
    }

    pub(crate) fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    pub(super) fn episode_detail(&self) -> Option<String> {
        let numbers = self
            .meta
            .get("lastVideoId")
            .and_then(Value::as_str)
            .and_then(|video| {
                let mut parts = video.rsplit(':');
                let episode = parts.next()?.parse::<i64>().ok()?;
                let season = parts.next()?.parse::<i64>().ok()?;
                Some((season, episode))
            });
        match (numbers, self.episode_title.as_deref()) {
            (Some((season, episode)), Some(title)) => Some(format!("S{season}:E{episode} {title}")),
            (Some((season, episode)), None) => Some(format!("S{season}:E{episode}")),
            (None, title) => title.map(ToOwned::to_owned),
        }
    }

    pub(crate) fn presence(&self) -> crate::presence::Presence {
        if !self.wants_landscape() {
            return crate::presence::Presence::Viewing {
                title: self.title(),
                poster: self
                    .meta
                    .get("poster")
                    .and_then(Value::as_str)
                    .filter(|url| url.starts_with("http"))
                    .map(ToOwned::to_owned),
            };
        }
        crate::presence::Presence::Playing {
            title: self.title(),
            detail: self.episode_detail(),
            paused: self.status.paused,
            position: self.status.position,
            duration: self.status.duration,
            poster: self
                .meta
                .get("poster")
                .and_then(Value::as_str)
                .filter(|url| url.starts_with("http"))
                .map(ToOwned::to_owned),
            party: None,
        }
    }

    pub(crate) fn pick_manually(&mut self, snapshot: Option<&Value>) {
        let profile = snapshot.and_then(|snapshot| snapshot.pointer("/profile/active"));
        self.manual = self.meta.get("pickSource").is_some()
            || (profile
                .and_then(|profile| profile.get("streamSourceSelectionMode"))
                .and_then(Value::as_str)
                .unwrap_or("manual")
                == "manual"
                && !stream_policy::resumes_saved_stream(&self.meta, profile));
        self.resume_at = self
            .meta
            .get("timeOffset")
            .and_then(Value::as_f64)
            .filter(|offset| *offset > 5.0);
        self.start_paused = self.meta.get("startPaused").and_then(Value::as_bool) == Some(true);
    }

    pub(super) fn streams_listing(&self) -> bool {
        self.streams_requested && !self.streams_done
    }

    pub(crate) fn listed_stream(&self, index: usize) -> Option<Value> {
        self.listed_streams().into_iter().nth(index)
    }

    pub(super) fn listed_streams(&self) -> Vec<Value> {
        self.stream_parts.values().flatten().cloned().collect()
    }

    pub(super) fn poll_streams(&mut self, session: &fluxa_effects::SessionHandle) {
        if !self.manual || self.local || self.chosen.is_some() || self.streams_done {
            return;
        }
        self.streams_requested = true;
        let Some(feed) = session.stream_feed_since(self.opened_ms, self.feed_version) else {
            return;
        };
        self.feed_version = feed.version;
        self.stream_parts = feed.parts;
        self.streams_done = feed.done;
        let listed = self.listed_streams();
        self.pending_addons = self.pending_for(&listed);
        if !listed.is_empty() || self.streams_done {
            self.sources = Some(listed);
        }
    }

    pub(super) fn pending_for(&self, listed: &[Value]) -> Vec<String> {
        self.stream_addons
            .iter()
            .filter(|addon| {
                !listed.iter().any(|stream| {
                    stream.get("addonName").and_then(Value::as_str) == Some(addon.as_str())
                })
            })
            .cloned()
            .collect()
    }

    pub(crate) fn wants_landscape(&self) -> bool {
        !(self.manual && self.chosen.is_none())
    }

    pub(super) fn saved_stream_failed(&self) -> bool {
        let saved = ["lastStreamUrl", "lastStream"].iter().any(|key| {
            self.meta
                .get(*key)
                .is_some_and(|value| !value.is_null() && value.as_str() != Some(""))
        });
        saved
            && self.chosen.is_none()
            && !self.local
            && !self.status.has_frame
            && (self.error.is_some() || self.status.error.is_some())
    }

    pub(super) fn load_stage(&self) -> String {
        let text = |key: &str| fluxa_ui::localized(key, &self.language);
        if self.loaded_url.is_none() {
            let previews = self.seek_previews.as_ref().is_some_and(|p| p.preparing());
            return text(if previews {
                "player.status_seek_previews"
            } else {
                "player.status_preparing"
            });
        }
        if let Some(progress) = self.seek_previews.as_ref().and_then(|p| p.holding()) {
            let label = text("player.status_seek_previews");
            return match progress {
                Some(progress) => {
                    format!("{label} {}%", progress.done * 100 / progress.count.max(1))
                }
                None => label,
            };
        }
        match self.status.buffering {
            Some(fraction) => text("player.status_buffering_percent")
                .replace("%s", &((fraction * 100.0).round() as u32).to_string()),
            None => text("player.status_connecting_source"),
        }
    }

    pub(super) fn torrent_unresponsive(&self) -> bool {
        let Some(status) = self.torrent_status.as_ref() else {
            return false;
        };
        let unresolved = status
            .get("hash")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty);
        let peers = status
            .get("total_peers")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        unresolved
            && peers > 0
            && self
                .torrent_since
                .is_some_and(|since| since.elapsed() >= TORRENT_UNRESPONSIVE_AFTER)
    }

    pub(super) fn torrent_lines(&self) -> (String, String) {
        if self.torrent_unresponsive() {
            let text = |key: &str| fluxa_ui::localized(key, &self.language);
            return (
                text("player.torrent_unresponsive"),
                text("player.torrent_unresponsive_hint"),
            );
        }
        fluxa_ui::torrent_status_lines(self.torrent_status.as_ref())
    }

    pub(crate) fn model(&self) -> PlayerModel {
        let holding = self
            .seek_previews
            .as_ref()
            .is_some_and(|p| p.holding().is_some());
        let waiting = self.status.stalled && self.recovery.stalled_for() >= BUFFERING_SHOWN_AFTER;
        let showing = self.had_frame && (self.texture.is_some() || self.passthrough);
        let status = if showing && (holding || !waiting) {
            None
        } else if self.torrent_link.is_some() {
            Some(self.torrent_lines())
        } else {
            Some((self.load_stage(), String::new()))
        };
        PlayerModel {
            title: self.title(),
            video: self.texture.filter(|_| self.had_frame),
            video_hole: false,
            passthrough: self.passthrough && self.had_frame,
            subtitle: self.subtitle.clone(),
            watch_button: false,
            locked: self.locked,
            watch: None,
            status,
            error: self.error.clone().or_else(|| self.status.error.clone()),
            position: self.status.position,
            duration: self.status.duration,
            paused: self.status.paused,
            muted: self.status.muted,
            volume: self.status.volume,
            boost: false,
            stats: self.stats.clone(),
            controls_visible: self.controls_visible(),
            show_pause_info: self.status.paused
                && self.status.has_frame
                && !self.controls_visible(),
            logo: ["logo", "logoUrl", "clearLogo", "titleLogo", "titleLogoUrl"]
                .iter()
                .find_map(|key| {
                    self.meta
                        .get(*key)
                        .and_then(Value::as_str)
                        .filter(|url| !url.trim().is_empty())
                })
                .map(ToOwned::to_owned)
                .or_else(|| self.snapshot_logo.clone()),
            episode_title: self.episode_title.clone(),
            episode_image: self.episode_image.clone(),
            season_episode: self.season_episode,
            synopsis: self.synopsis.clone(),
            description: self.description.clone(),
            chapters: self.status.chapters.clone(),
            buffered: self.status.buffered,
            chapter_spans: self.overlay.chapter_spans(),
            segment_spans: self.overlay.segment_spans(),
            mark_preview: match &self.panel {
                Some(Panel::Submit(submit)) => {
                    submit.preview(self.status.position, self.status.duration)
                }
                _ => None,
            },
            skip: self.overlay.skip_card(self.status.position),
            next_episode: self.overlay.next_card(),
            thumbnail: self
                .thumbnail
                .as_ref()
                .map(|(time, texture)| (*time, texture.id())),
            filmstrip: self.filmstrip.textures(),
            filmstrip_step: self.filmstrip_step(),
            warnings: self.warnings.clone(),
            warnings_elapsed: self.warnings_clock,
            language: self.language.clone(),
            upscaling: String::new(),
            recommendations: self
                .recommendations
                .iter()
                .map(|item| fluxa_ui::hero_from_meta(item, &self.language))
                .collect(),
            recommendation_index: self.recommendation_index,
            background: self
                .meta
                .get("background")
                .and_then(Value::as_str)
                .filter(|url| !url.trim().is_empty())
                .map(ToOwned::to_owned),
            load_progress: (self.load_progress > 0.0).then_some(self.load_progress),
            scrub: self.scrub.map(|(time, _)| time),
            playing_badges: self.playing_badges.clone(),
            sources: match &self.sources {
                Some(streams) => Some(self.rows_for(streams)),
                None if self.manual && self.chosen.is_none() && self.error.is_none() => {
                    Some(Default::default())
                }
                None => None,
            },
            sources_loading: self.sources.is_none() || self.streams_listing(),
            sources_pending: if self.sources.is_none() || self.streams_listing() {
                self.pending_addons.clone()
            } else {
                Vec::new()
            },
            source_filter: self.source_filter.clone(),
            source_chips_offset: self.source_chips_offset,
            toast: self.toast_model(),
            panel: self.panel.as_ref().map(|panel| match panel {
                Panel::Submit(submit) => submit.model(&self.language),
                Panel::Cast => self.cast.model(&self.language),
                Panel::Menu(menu) => menu::model(self, *menu),
            }),
            has_next: self.overlay.upcoming().is_some(),
            has_previous: menu::has_previous(self),
            trailer: self.trailer.is_some(),
            minimized: self.minimized,
            minimizable: false,
            pip: false,
            has_episodes: menu::has_episodes(self),
            can_change_source: !self.local && self.trailer.is_none(),
            dim: 1.0 - self.brightness,
            options: fluxa_ui::PlayerOptions::default(),
        }
    }
}
