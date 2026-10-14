use std::sync::mpsc::Receiver;

use fluxa_core::ambient::{self, Slide};
use fluxa_ui::{
    AMBIENT_FADE_IN, AMBIENT_FADE_OUT, AMBIENT_MOTIONS, AMBIENT_SLIDE_SECONDS, AmbientView,
    NODE_SETTINGS_AMBIENT_CATALOG_BASE, NODE_SETTINGS_AMBIENT_SAME_AS_HERO, next_ambient_motion,
};
use serde_json::{Value, json};

use super::*;

const EMPTY_RECHECK: Duration = Duration::from_millis(500);
const ALL_FAILED_RETRY: Duration = Duration::from_secs(15);
const IMAGE_TIMEOUT: Duration = Duration::from_secs(20);

pub(crate) struct State {
    last_input: Instant,
    show: Option<Show>,
    swallowing: bool,
    motion: Option<usize>,
    clock: u64,
}

struct Show {
    started: Instant,
    dismissed: Option<Instant>,
    slides: Vec<Slide>,
    index: usize,
    slide_started: Option<Instant>,
    waiting_since: Instant,
    motion: usize,
    failures: usize,
    check_at: Instant,
    retry_at: Option<Instant>,
    loading: Option<Receiver<Vec<Value>>>,
    filled: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_input: Instant::now(),
            show: None,
            swallowing: false,
            motion: None,
            clock: 0x9E37_79B9_7F4A_7C15,
        }
    }
}

impl State {
    fn roll(&mut self) -> u64 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as u64);
        self.clock = self.clock.rotate_left(17) ^ nanos.wrapping_mul(0x2545_F491_4F6C_DD1D);
        self.clock
    }

    pub(crate) fn visible(&self) -> bool {
        self.show.is_some()
    }
}

fn timeout(state: &RendererState) -> Duration {
    let minutes = ambient::timeout_minutes(state.settings.str_value("ambientModeTimeoutMinutes"));
    Duration::from_secs(u64::from(minutes) * 60)
}

fn playing(state: &RendererState) -> bool {
    state.player.is_some() || state.trailers.playing()
}

fn eligible(state: &RendererState) -> bool {
    state.settings.bool_value("ambientModeEnabled")
        && state.session.is_some()
        && state.gpu.is_some()
        && state.window_focused
        && !playing(state)
        && state.profiles.is_none()
        && state.welcome.is_none()
}

fn live(show: &Show) -> bool {
    show.dismissed.is_none()
}

pub(crate) fn input(state: &mut RendererState) -> bool {
    let now = Instant::now();
    state.ambient.last_input = now;
    match state.ambient.show.as_mut() {
        Some(show) if live(show) => {
            show.dismissed = Some(now);
            true
        }
        _ => false,
    }
}

pub(crate) fn pointer(state: &mut RendererState, phase: PointerPhase) -> bool {
    if input(state) {
        state.ambient.swallowing = true;
    }
    if state.ambient.swallowing {
        if phase == PointerPhase::Up {
            state.ambient.swallowing = false;
        }
        return true;
    }
    false
}

pub(crate) fn next_redraw(state: &RendererState) -> Option<Instant> {
    if state.ambient.show.is_some() {
        return Some(Instant::now());
    }
    eligible(state).then(|| state.ambient.last_input + timeout(state))
}

fn hero_items(state: &RendererState) -> Vec<Value> {
    state
        .home
        .hero_slides
        .iter()
        .map(|hero| hero.raw.clone())
        .collect()
}

fn selected_catalogs(state: &RendererState) -> Vec<String> {
    state
        .settings
        .value("ambientModeCatalogs")
        .and_then(Value::as_array)
        .map(|keys| {
            keys.iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn begin(state: &mut RendererState) {
    state.card_menu = None;
    let now = Instant::now();
    let selected = selected_catalogs(state);
    let loading = if selected.is_empty() {
        None
    } else {
        let catalogs: Vec<Value> = ambient::catalog_options(&state.settings.addons)
            .into_iter()
            .filter(|option| selected.contains(&option.key))
            .map(|option| {
                json!({
                    "transportUrl": option.transport_url,
                    "contentType": option.content_type,
                    "catalogId": option.catalog_id,
                })
            })
            .collect();
        state
            .session
            .as_ref()
            .map(|session| session.executor().fetch_ambient_items(catalogs))
    };
    let motion = next_ambient_motion(state.ambient.motion, state.ambient.roll());
    state.ambient.motion = Some(motion);
    state.ambient.show = Some(Show {
        started: now,
        dismissed: None,
        slides: Vec::new(),
        index: 0,
        slide_started: None,
        waiting_since: now,
        motion,
        failures: 0,
        check_at: now,
        retry_at: None,
        loading,
        filled: false,
    });
}

fn fill(state: &mut RendererState, now: Instant) {
    let seed = state.ambient.roll();
    let hero = hero_items(state);
    let Some(show) = state.ambient.show.as_mut() else {
        return;
    };
    if show.filled || now < show.check_at {
        return;
    }
    show.check_at = now + EMPTY_RECHECK;
    let items = match show.loading.as_ref() {
        Some(receiver) => match receiver.try_recv() {
            Ok(items) => {
                show.loading = None;
                items
            }
            Err(_) => return,
        },
        None => hero,
    };
    show.slides = ambient::slides(&items, seed);
    if !show.slides.is_empty() {
        show.filled = true;
        show.waiting_since = now;
    }
}

fn advance(state: &mut RendererState, now: Instant) {
    let roll = state.ambient.roll();
    let previous = state.ambient.motion;
    let motion = next_ambient_motion(previous, roll);
    let Some(show) = state.ambient.show.as_mut() else {
        return;
    };
    show.index = (show.index + 1) % show.slides.len().max(1);
    show.slide_started = None;
    show.waiting_since = now;
    show.motion = motion;
    state.ambient.motion = Some(motion);
}

fn drive_image(state: &mut RendererState, now: Instant) {
    let Some(gpu) = state.gpu.as_mut() else {
        return;
    };
    let target = [gpu.config.width, gpu.config.height];
    let Some(show) = state.ambient.show.as_mut() else {
        return;
    };
    if !show.filled || show.slides.is_empty() {
        return;
    }
    if let Some(at) = show.retry_at {
        if now < at {
            return;
        }
        show.retry_at = None;
        show.failures = 0;
    }
    let slide = show.slides[show.index].backdrop.clone();
    let next = show.slides[(show.index + 1) % show.slides.len()]
        .backdrop
        .clone();
    let ready = gpu
        .artwork
        .texture_for_priority(Some(&slide), target, ArtworkFetchPriority::Hero)
        .is_some();
    if show.slides.len() > 1 {
        let _ =
            gpu.artwork
                .texture_for_priority(Some(&next), target, ArtworkFetchPriority::Prefetch);
    }
    if ready {
        if show.slide_started.is_none() {
            show.slide_started = Some(now);
            show.failures = 0;
        }
        return;
    }
    let failed = gpu.artwork.failed(&slide, target)
        || now.duration_since(show.waiting_since) > IMAGE_TIMEOUT;
    if !failed {
        return;
    }
    show.failures += 1;
    if show.failures >= show.slides.len() {
        show.retry_at = Some(now + ALL_FAILED_RETRY);
        return;
    }
    advance(state, now);
}

pub(crate) fn tick(state: &mut RendererState) {
    let now = Instant::now();
    if playing(state) {
        state.ambient.last_input = now;
    }
    let eligible = eligible(state);
    match state.ambient.show.as_mut() {
        Some(show) => {
            if let Some(dismissed) = show.dismissed {
                if now.duration_since(dismissed).as_secs_f32() >= AMBIENT_FADE_OUT {
                    state.ambient.show = None;
                }
                return;
            }
            if !eligible {
                show.dismissed = Some(now);
                return;
            }
        }
        None => {
            if eligible && now.duration_since(state.ambient.last_input) >= timeout(state) {
                begin(state);
            } else {
                return;
            }
        }
    }
    fill(state, now);
    let finished = state
        .ambient
        .show
        .as_ref()
        .and_then(|show| show.slide_started)
        .is_some_and(|started| now.duration_since(started).as_secs_f32() >= AMBIENT_SLIDE_SECONDS);
    if finished {
        advance(state, now);
    }
    drive_image(state, now);
}

pub(crate) fn view(state: &RendererState) -> Option<AmbientView> {
    let show = state.ambient.show.as_ref()?;
    let now = Instant::now();
    let fade_in = (now.duration_since(show.started).as_secs_f32() / AMBIENT_FADE_IN).min(1.0);
    let overlay = match show.dismissed {
        Some(at) => {
            let out = 1.0 - now.duration_since(at).as_secs_f32() / AMBIENT_FADE_OUT;
            out.clamp(0.0, 1.0).min(fade_in)
        }
        None => fade_in,
    };
    let mut view = AmbientView {
        overlay,
        motion: show.motion % AMBIENT_MOTIONS,
        ..AmbientView::default()
    };
    if let (Some(slide), Some(started)) = (show.slides.get(show.index), show.slide_started) {
        view.backdrop = slide.backdrop.clone();
        view.title = slide.title.clone();
        view.logo = slide.logo.clone();
        view.genres = slide.genres.clone();
        view.elapsed = now.duration_since(started).as_secs_f32();
    }
    Some(view)
}

pub(crate) fn sync_settings(state: &mut RendererState) {
    if state.route != Route::Settings {
        return;
    }
    let selected = selected_catalogs(state);
    state.settings.ambient_catalogs = ambient::catalog_options(&state.settings.addons)
        .into_iter()
        .map(|option| fluxa_ui::AmbientCatalogRow {
            selected: selected.contains(&option.key),
            name: option.name,
        })
        .collect();
    state.settings.ambient_minutes =
        ambient::timeout_minutes(state.settings.str_value("ambientModeTimeoutMinutes"));
    state.settings.ambient_limit = ambient::MAX_CATALOGS;
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if node == NODE_SETTINGS_AMBIENT_SAME_AS_HERO {
        crate::stream_badges::set_setting(state, "ambientModeCatalogs", json!([]));
    } else if let Some(index) = node
        .checked_sub(NODE_SETTINGS_AMBIENT_CATALOG_BASE)
        .map(|offset| offset as usize)
        .filter(|index| *index < fluxa_ui::AMBIENT_CATALOG_MAX)
    {
        let Some(option) = ambient::catalog_options(&state.settings.addons)
            .into_iter()
            .nth(index)
        else {
            return true;
        };
        let next = ambient::toggle_catalog(&selected_catalogs(state), &option.key);
        crate::stream_badges::set_setting(state, "ambientModeCatalogs", json!(next));
    } else {
        return false;
    }
    sync_settings(state);
    true
}
