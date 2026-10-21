use super::*;

impl HomeModel {
    pub fn loading_screen(&self) -> bool {
        self.is_loading && self.item_id.is_none() && self.cards.is_empty() && self.rows.is_empty()
    }

    pub fn content_rows(&self) -> Vec<(&str, &[HomeCard])> {
        self.content_rows_with_kind()
            .into_iter()
            .map(|(title, cards, _)| (title, cards))
            .collect()
    }

    pub fn content_rows_with_kind(
        &self,
    ) -> impl Iterator<Item = (&str, &[HomeCard], HomeRowKind)> + '_ {
        let continue_row = std::iter::once((
            "Continue Watching",
            self.cards.as_slice(),
            HomeRowKind::Continue,
        ))
        .filter(|(_, cards, _)| !cards.is_empty());
        let rows = self
            .rows
            .iter()
            .map(|row| (row.title.as_str(), row.cards.as_slice(), row.kind));
        let fallback = std::iter::once((
            "Continue Watching",
            self.cards.as_slice(),
            HomeRowKind::Continue,
        ))
        .filter(|_| self.cards.is_empty() && self.rows.is_empty());
        continue_row.chain(rows).chain(fallback)
    }

    pub fn apply_logos(&mut self, logos: &serde_json::Map<String, serde_json::Value>) {
        let cards = self
            .cards
            .iter_mut()
            .chain(self.rows.iter_mut().flat_map(|row| row.cards.iter_mut()));
        for card in cards {
            if card.logo_url.is_some() {
                continue;
            }
            card.logo_url = card
                .id
                .as_deref()
                .and_then(|id| logos.get(id))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);
        }
    }

    pub fn card_at(&self, index: usize) -> Option<&HomeCard> {
        if let Some(card) = self.cards.get(index) {
            return Some(card);
        }
        let row_index = index.saturating_sub(self.cards.len());
        self.rows
            .iter()
            .flat_map(|row| row.cards.iter())
            .nth(row_index)
    }
}

/// Projects the host-independent Core snapshot into the renderer's shared
/// Home model. Keeping this conversion here means Android, desktop and the
/// future WASM host all paint the same data shape without a Kotlin/TypeScript
/// mapper in front of the renderer.
pub fn home_model_from_core_snapshot(
    snapshot: &serde_json::Value,
    form_factor: UiFormFactorJson,
) -> HomeModel {
    home_model_with_hero(snapshot, form_factor, None, None)
}

pub fn home_model_with_hero(
    snapshot: &serde_json::Value,
    form_factor: UiFormFactorJson,
    billboard_override: Option<&serde_json::Value>,
    slides_override: Option<&serde_json::Value>,
) -> HomeModel {
    let home = snapshot.get("home").unwrap_or(&serde_json::Value::Null);
    let billboard = billboard_override
        .or_else(|| home.get("billboard"))
        .filter(|value| !value.is_null())
        .or_else(|| snapshot.pointer("/billboard/movie"))
        .unwrap_or(&serde_json::Value::Null);
    let mut model = HomeModel {
        is_loading: home
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        title: first_value_string(billboard, &["name", "title"])
            .unwrap_or_else(|| "Recommended".to_owned()),
        eyebrow: hero_meta_line(billboard, &snapshot_language(snapshot)),
        description: first_value_string(billboard, &["description", "overview"])
            .unwrap_or_else(|| "Your connected catalogs will appear here.".to_owned()),
        background_url: first_value_string(
            billboard,
            &[
                "background",
                "backgroundUrl",
                "backdrop",
                "backdropUrl",
                "poster",
                "posterUrl",
            ],
        ),
        logo_url: first_value_string(billboard, &["logo", "logoUrl", "clearLogo"]),
        profile_avatar_url: first_value_string(
            home.get("activeProfile")
                .unwrap_or(&serde_json::Value::Null),
            &["avatarUrl", "avatar"],
        )
        .or_else(|| {
            first_value_string(
                snapshot
                    .pointer("/profile/active")
                    .unwrap_or(&serde_json::Value::Null),
                &["avatarUrl", "avatar"],
            )
        }),
        profile_name: first_value_string(
            home.get("activeProfile")
                .unwrap_or(&serde_json::Value::Null),
            &["displayName", "name"],
        )
        .or_else(|| {
            first_value_string(
                snapshot
                    .pointer("/profile/active")
                    .unwrap_or(&serde_json::Value::Null),
                &["displayName", "name"],
            )
        }),
        item_id: value_string(billboard, "id"),
        item_type: value_string(billboard, "type"),
        show_hero_section: home
            .get("activeProfile")
            .and_then(|profile| profile.get("showHeroSection"))
            .and_then(serde_json::Value::as_bool)
            .or_else(|| {
                snapshot
                    .pointer("/profile/active/showHeroSection")
                    .and_then(serde_json::Value::as_bool)
            })
            .unwrap_or(true),
        gif_autoplay_enabled: snapshot
            .pointer("/settings/values/gifAutoplayEnabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        follow_focus_hero: snapshot
            .pointer("/settings/values/followFocusHero")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        expanded_posters: snapshot
            .pointer("/settings/values/expandedPosters")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        expanded_posters_delay: snapshot
            .pointer("/settings/values/expandedPostersDelay")
            .and_then(|value| {
                value
                    .as_f64()
                    .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
            })
            .unwrap_or(1.0)
            .max(0.0),
        accent: snapshot
            .pointer("/settings/values/accentColorArgb")
            .and_then(accent_from_value),
        form_factor,
        language: snapshot_language(snapshot),
        ..HomeModel::default_empty()
    };
    let posters = snapshot
        .pointer("/settings/values/homeHeroArtwork")
        .and_then(serde_json::Value::as_str)
        == Some("poster");
    let season_posters = snapshot
        .pointer("/settings/values/homeSeasonPostersOnHero")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let poster = |item: &serde_json::Value| {
        season_posters
            .then(|| detail_season_posters(item))
            .and_then(|posters| posters.into_iter().max_by_key(|(season, _)| *season))
            .map(|(_, url)| url)
            .or_else(|| first_value_string(item, &["poster", "posterUrl"]))
    };
    model.hero_poster = posters;
    if posters && let Some(poster) = poster(billboard) {
        model.background_url = Some(poster);
    }
    let mut slide_keys = Vec::new();
    let mut hero_slides = Vec::new();
    for item in std::iter::once(billboard).chain(
        slides_override
            .or_else(|| home.get("heroSlides"))
            .or_else(|| home.get("slides"))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten(),
    ) {
        let mut hero = core_home_hero(item, &snapshot_language(snapshot));
        if posters && let Some(poster) = poster(item) {
            hero.background_url = Some(poster);
        }
        let key = hero
            .item_id
            .clone()
            .or_else(|| hero.background_url.clone())
            .unwrap_or_else(|| hero.title.clone());
        if !key.is_empty() && !slide_keys.contains(&key) {
            slide_keys.push(key);
            hero_slides.push(hero);
        }
    }
    model.hero_slides = hero_slides;
    if let Some(items) = [
        home.get("continueWatching"),
        home.get("currentWatchlist"),
        home.get("externalContinueWatching"),
    ]
    .into_iter()
    .flatten()
    .find_map(serde_json::Value::as_array)
    {
        model.cards = items
            .iter()
            .map(|item| core_home_card_for_kind(item, HomeRowKind::Continue))
            .collect();
    }
    if let Some(categories) = home
        .get("categories")
        .or_else(|| home.get("rows"))
        .and_then(serde_json::Value::as_array)
    {
        // `continueWatching` is rendered from its dedicated list above. Some
        // Core snapshots also include that same list as a category; keeping
        // both produces a duplicate shelf immediately before the catalogs.
        if model.cards.is_empty()
            && let Some(category) = categories.iter().find(|category| {
                value_string(category, "id").as_deref() == Some("continue_watching")
                    || category.get("type").and_then(serde_json::Value::as_str)
                        == Some("continue_watching")
            })
            && let Some(items) = category.get("items").and_then(serde_json::Value::as_array)
        {
            model.cards = items
                .iter()
                .map(|item| core_home_card_for_kind(item, HomeRowKind::Continue))
                .collect();
        }
        let language = model.language.clone();
        let show_catalog_type = snapshot
            .pointer("/settings/values/showCatalogType")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        model.rows = categories
            .iter()
            .enumerate()
            .filter_map(|(index, category)| {
                if value_string(category, "id").as_deref() == Some("continue_watching")
                    || category.get("type").and_then(serde_json::Value::as_str)
                        == Some("continue_watching")
                {
                    return None;
                }
                if matches!(
                    category.get("type").and_then(serde_json::Value::as_str),
                    Some("collection_folder_source")
                ) {
                    return None;
                }
                let cards = category
                    .get("items")
                    .and_then(serde_json::Value::as_array)?;
                if cards.is_empty() {
                    return None;
                }
                let kind = match (
                    value_string(category, "id").as_deref(),
                    category.get("type").and_then(serde_json::Value::as_str),
                ) {
                    (Some("continue_watching" | "upcoming"), _)
                    | (_, Some("continue_watching" | "upcoming")) => HomeRowKind::Continue,
                    (_, Some("collection" | "collection_folder")) => HomeRowKind::Collection,
                    _ => HomeRowKind::Poster,
                };
                Some(HomeRow {
                    id: value_string(category, "id"),
                    catalog_page: Some(category.clone()),
                    title: first_value_string(
                        category,
                        &["homeTitle", "name", "label", "title", "id"],
                    )
                    .filter(|title| !title.trim().is_empty())
                    .map(|title| normalize_home_row_title(&title))
                    .unwrap_or_else(|| format!("Catalog {}", index + 1)),
                    type_label: show_catalog_type
                        .then(|| category.get("type"))
                        .flatten()
                        .and_then(serde_json::Value::as_str)
                        .and_then(|kind| match kind {
                            "movie" => Some("settings.addon_type.movie"),
                            "series" => Some("settings.addon_type.series"),
                            _ => None,
                        })
                        .map(|key| crate::localized(key, &language)),
                    cards: cards
                        .iter()
                        .map(|item| core_home_card_for_kind(item, kind))
                        .collect(),
                    kind,
                    can_load_more: category
                        .get("canLoadMore")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                })
            })
            .collect();
    }
    finish_continue_cards(&mut model, snapshot);
    apply_row_layout(&mut model);
    model
}

fn row_matches(row: &HomeRow, key: &str) -> bool {
    row.id.as_deref().is_some_and(|id| id.to_lowercase() == key) || row.title.to_lowercase() == key
}

fn apply_row_layout(model: &mut HomeModel) {
    let hidden = fluxa_theme::css::hidden_rows();
    let order = fluxa_theme::css::row_order();
    if hidden.iter().any(|key| key == "continue-watching") {
        model.cards.clear();
    }
    model
        .rows
        .retain(|row| !hidden.iter().any(|key| row_matches(row, key)));
    if order.is_empty() {
        return;
    }
    let position = |row: &HomeRow| {
        order
            .iter()
            .position(|key| row_matches(row, key))
            .unwrap_or(usize::MAX)
    };
    model.rows.sort_by_key(position);
}

fn finish_continue_cards(model: &mut HomeModel, snapshot: &serde_json::Value) {
    let language = model.language.clone();
    let show_percent = snapshot
        .pointer("/settings/values/continueProgressLabel")
        .and_then(serde_json::Value::as_str)
        == Some("percent");
    for card in &mut model.cards {
        (card.detail, card.episodes) = continue_detail(&card.raw, &language, show_percent);
    }
    let separate = snapshot
        .pointer("/settings/values/showUpcomingRow")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !separate || !model.cards.iter().any(|card| card.upcoming) {
        return;
    }
    let (upcoming, watching): (Vec<_>, Vec<_>) = std::mem::take(&mut model.cards)
        .into_iter()
        .partition(|card| card.upcoming);
    model.cards = watching;
    model.rows.insert(
        0,
        HomeRow {
            id: Some("upcoming".to_owned()),
            catalog_page: None,
            title: localized("calendar.upcoming", &language),
            type_label: None,
            cards: upcoming,
            kind: HomeRowKind::Continue,
            can_load_more: false,
        },
    );
}

pub(crate) fn core_home_hero(item: &serde_json::Value, language: &str) -> HomeHero {
    HomeHero {
        title: first_value_string(item, &["name", "title"]).unwrap_or_default(),
        eyebrow: hero_meta_line(item, language),
        description: first_value_string(item, &["description", "overview"]).unwrap_or_default(),
        background_url: first_value_string(
            item,
            &[
                "background",
                "backgroundUrl",
                "backdrop",
                "backdropUrl",
                "poster",
                "posterUrl",
            ],
        ),
        logo_url: first_value_string(item, &["logo", "logoUrl", "clearLogo"]),
        item_id: value_string(item, "id"),
        item_type: value_string(item, "type"),
        trailers: trailer_urls(item),
        raw: item.clone(),
    }
}

pub(crate) fn trailer_urls(item: &serde_json::Value) -> Vec<String> {
    item.get("trailers")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| first_value_string(item, &["url", "link"]))
                .take(6)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HeroCarouselState {
    pub(super) index: usize,
    pub(super) generation: u64,
    pub(super) next_at: f64,
}

fn hero_carousel_id(home: &HomeModel) -> Id {
    let signature = home
        .hero_slides
        .iter()
        .map(|slide| {
            slide
                .item_id
                .as_deref()
                .or(slide.background_url.as_deref())
                .unwrap_or(slide.title.as_str())
        })
        .collect::<Vec<_>>();
    Id::new(("fluxa-home-hero-carousel", signature))
}

pub(crate) fn hero_shift(context: &egui::Context, home: &HomeModel, delta: isize) {
    let count = home.hero_slides.len();
    if count <= 1 {
        return;
    }
    let id = hero_carousel_id(home);
    let mut state = context
        .data_mut(|data| data.get_temp::<HeroCarouselState>(id))
        .unwrap_or_default();
    state.index = (state.index as isize + delta).rem_euclid(count as isize) as usize;
    state.generation = state.generation.wrapping_add(1);
    state.next_at = context.input(|input| input.time) + 6.5;
    context.data_mut(|data| data.insert_temp(id, state));
    context.request_repaint();
}

const HERO_WRAPPED: &str = "fluxa-hero-wrapped";

pub fn take_hero_wrap(context: &egui::Context) -> bool {
    context.data_mut(|data| data.remove_temp::<bool>(Id::new(HERO_WRAPPED)).is_some())
}

pub(crate) fn home_hero_index(context: &egui::Context, home: &HomeModel) -> usize {
    let count = home.hero_slides.len();
    if count <= 1 {
        return 0;
    }
    let id = hero_carousel_id(home);
    let now = context.input(|input| input.time);
    let mut state = context
        .data_mut(|data| data.get_temp::<HeroCarouselState>(id))
        .unwrap_or_default();
    let current_trailer = home.trailer.as_ref().filter(|trailer| {
        home.hero_slides
            .get(state.index % count)
            .and_then(|slide| slide.item_id.as_deref())
            == Some(trailer.item_id.as_str())
    });
    let holding = current_trailer.is_some_and(|trailer| !trailer.finished);
    if current_trailer.is_some_and(|trailer| trailer.finished) {
        state.next_at = now;
    } else if state.next_at <= 0.0 || holding {
        state.next_at = now + 6.5;
    }
    if now >= state.next_at {
        state.index = (state.index + 1) % count;
        if state.index == 0 {
            context.data_mut(|data| data.insert_temp(Id::new(HERO_WRAPPED), true));
        }
        state.generation = state.generation.wrapping_add(1);
        state.next_at = now + 6.5;
    }
    context.request_repaint_after(Duration::from_secs_f64((state.next_at - now).max(0.0)));
    state.index %= count;
    context.data_mut(|data| data.insert_temp(id, state));
    state.index
}

/// Returns the hero item currently displayed by the shared Home carousel.
/// Hosts use this to dispatch View Details for the selected slide instead of
/// accidentally dispatching the first Core billboard forever.
pub fn active_home_hero<'a>(context: &egui::Context, home: &'a HomeModel) -> Option<&'a HomeHero> {
    home.hero_slides.get(home_hero_index(context, home))
}

pub(crate) fn normalize_home_row_title(title: &str) -> String {
    let parts = title
        .split(" - ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() >= 2 {
        parts[1..].join(" ")
    } else {
        title.trim().to_owned()
    }
}
