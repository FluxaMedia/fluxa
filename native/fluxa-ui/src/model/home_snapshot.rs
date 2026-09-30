use super::*;

impl HomeModel {
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
    let home = snapshot.get("home").unwrap_or(&serde_json::Value::Null);
    let billboard = home
        .get("billboard")
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
        accent: snapshot
            .pointer("/settings/values/accentColorArgb")
            .and_then(accent_from_value),
        form_factor,
        language: snapshot_language(snapshot),
        ..HomeModel::default_empty()
    };
    let mut slide_keys = Vec::new();
    let mut hero_slides = Vec::new();
    for item in std::iter::once(billboard).chain(
        home.get("heroSlides")
            .or_else(|| home.get("slides"))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten(),
    ) {
        let hero = core_home_hero(item, &snapshot_language(snapshot));
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
    model
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

pub(crate) fn home_hero_index(context: &egui::Context, home: &HomeModel) -> usize {
    let count = home.hero_slides.len();
    if count <= 1 {
        return 0;
    }
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
    let id = Id::new(("fluxa-home-hero-carousel", signature));
    let now = context.input(|input| input.time);
    let mut state = context
        .data_mut(|data| data.get_temp::<HeroCarouselState>(id))
        .unwrap_or_default();
    let holding = home.trailer.as_ref().is_some_and(|trailer| {
        home.hero_slides
            .get(state.index % count)
            .and_then(|slide| slide.item_id.as_deref())
            == Some(trailer.item_id.as_str())
    });
    if state.next_at <= 0.0 || holding {
        state.next_at = now + 6.5;
    }
    if now >= state.next_at {
        state.index = (state.index + 1) % count;
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
