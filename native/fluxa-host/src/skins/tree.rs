use super::*;

pub(super) static DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn touch() {
    DIRTY.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn begin_frame(idle: bool) {
    use std::sync::atomic::Ordering::Relaxed;
    let idle = idle && !DIRTY.swap(false, Relaxed);
    if idle {
        let frames = IDLE_FRAMES.fetch_add(1, Relaxed) + 1;
        IDLE.store(frames % IDLE_REFRESH != 0, Relaxed);
    } else {
        IDLE_FRAMES.store(0, Relaxed);
        IDLE.store(false, Relaxed);
    }
}

pub(super) fn windows_stamp(windows: &fluxa_skin::Windows) -> u64 {
    let mut hasher = fluxa_ui::FastHasher::default();
    windows.hash(&mut hasher);
    hasher.finish()
}

pub(crate) static TREE_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(super) struct TreeTiming(web_time::Instant);

impl Drop for TreeTiming {
    fn drop(&mut self) {
        TREE_NANOS.fetch_add(
            self.0.elapsed().as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

pub(crate) fn tree(
    route: Route,
    home: &HomeModel,
    settings: &SettingsModel,
    library: &fluxa_ui::LibraryModel,
    library_tab: fluxa_ui::LibraryTab,
    discover: &fluxa_ui::DiscoverModel,
    calendar: &fluxa_ui::CalendarModel,
    detail: &fluxa_ui::DetailModel,
    extra: Extra<'_>,
) -> Option<Arc<Vec<Resolved>>> {
    let _timing = TreeTiming(web_time::Instant::now());
    let page = PAGE.lock().ok().and_then(|page| page.clone());
    let screen = match extra {
        Extra::Profiles(_) => "profiles",
        Extra::Player(_) => "player",
        Extra::Folder(..) => "folder",
        Extra::Guide(_) => "live",
        Extra::Welcome(_) => "welcome",
        Extra::Menu(_) => "menu",
        Extra::Toast(_) | Extra::PlayerToast(_) => "toast",
        Extra::Tip(_) => "tip",
        Extra::Update(_) => "update",
        Extra::Window(_) => "window",
        Extra::Mini(_) => "mini",
        Extra::Watch(_) => "watch",
        Extra::Modal => "detail_modal",
        Extra::Dialog(_) => "dialog",
        Extra::Sources(_) => "sources",
        Extra::Recommendations(_) => "recommendations",
        Extra::None => match &page {
            Some(page) if route == Route::Home => page,
            _ => route.as_str(),
        },
    };
    let skin = skin_for(screen)?;
    let windows = fluxa_ui::grid_windows();
    let stamp = windows_stamp(&windows);
    if IDLE.load(std::sync::atomic::Ordering::Relaxed)
        && let Ok(cache) = CACHE.lock()
        && let Some((_, tree)) = cache.get(screen)
        && STAMPS
            .lock()
            .is_ok_and(|stamps| stamps.get(screen) == Some(&stamp))
    {
        return Some(tree.clone());
    }
    let fields = FIELDS
        .lock()
        .ok()
        .and_then(|fields| fields.clone())
        .unwrap_or_default();
    let mut nav = fluxa_ui::nav_data(if route == Route::Calendar {
        "library"
    } else {
        screen
    });

    let behavior = settings
        .values
        .get("navSidebarMode")
        .and_then(Value::as_str)
        .unwrap_or("hover");
    let current = page.as_deref().and_then(|page| page.strip_prefix("page."));
    nav["pages"] = json!(
        ACTIVE
            .read()
            .ok()
            .and_then(|skin| skin.clone())
            .map(|skin| skin
                .manifest
                .nav
                .iter()
                .map(|entry| json!({
                    "page": entry.page,
                    "label": entry.label,
                    "icon": entry.icon.as_deref().unwrap_or("NavLibrary"),
                    "class": if current == Some(entry.page.as_str()) { "navbar-item-active" } else { "navbar-item" },
                    "active": current == Some(entry.page.as_str()),
                }))
                .collect::<Vec<_>>())
            .unwrap_or_default()
    );
    nav["expanded"] = json!(behavior == "expanded");
    nav["hover"] = json!(
        home.form_factor == UiFormFactorJson::Desktop
            && behavior != "expanded"
            && behavior != "collapsed"
    );
    let slides = fluxa_ui::hero_slides_data(home);
    let screen_data = || match route {
        Route::Library => fluxa_ui::library_data(library, library_tab),
        Route::Discover => fluxa_ui::discover_data(discover),
        Route::Calendar => fluxa_ui::calendar_data(calendar),
        Route::Detail => fluxa_ui::detail_data(detail),
        _ => Value::Null,
    };
    let screen_key = if route == Route::Discover {
        let mut hasher = fluxa_ui::FastHasher::default();
        (
            discover.results.len(),
            discover.results.last().and_then(|card| card.id.as_deref()),
            discover.next_page.as_ref().map(json_key),
            discover.generation,
            &discover.query,
            &discover.content_type,
            &discover.selected_catalog_key,
            &discover.selected_extra_value,
            discover.is_loading || discover.catalogs_loading,
            discover.error.is_some(),
        )
            .hash(&mut hasher);
        hasher.finish()
    } else {
        json_key(&screen_data())
    };
    let extra_data = match extra {
        Extra::Profiles(model) => fluxa_ui::profiles_data(model, home.form_factor.into()),
        Extra::Player(model) => {
            fluxa_ui::player_data_for(model, matches!(home.form_factor, UiFormFactorJson::Mobile))
        }
        Extra::Folder(model, tab) => fluxa_ui::folder_data(model, tab),
        Extra::Guide(model) => fluxa_ui::guide_data(model),
        Extra::Welcome(model) => fluxa_ui::welcome_data(model),
        Extra::Menu(view) => menu_data(view),
        Extra::Toast(text) => json!({"text": text}),
        Extra::PlayerToast(data) => (*data).clone(),
        Extra::Modal => fluxa_ui::detail_data(detail),
        Extra::Dialog(data) => data.clone(),
        Extra::Sources(player) => fluxa_ui::sources_data(player),
        Extra::Recommendations(player) => fluxa_ui::recommendations_data(player),
        Extra::Mini(player) => json!({
            "playIcon": if player.paused { "PlayFilled" } else { "PauseFilled" },
            "ratio": if player.duration > 0.0 { (player.position / player.duration).clamp(0.0, 1.0) } else { 0.0 },
        }),
        Extra::Watch(model) => fluxa_ui::watch_data(model),
        Extra::Update(version) => json!({"version": version}),
        Extra::Window(full) => json!({
            "icon": if full { "CornersIn" } else { "CornersOut" },
        }),
        Extra::Tip(tip) => json!({
            "text": tip.text,
            "x": tip.rect.center().x,
            "y": tip.rect.bottom() + 8.0,
        }),
        Extra::None => Value::Null,
    };
    ext::start(&skin);
    let base = fingerprint(
        screen,
        &skin,
        home,
        settings,
        &fields,
        &nav,
        &(json_key(&slides), screen_key, json_key(&extra_data)),
        ext::version(),
    );
    let key = {
        let mut hasher = fluxa_ui::FastHasher::default();
        (base, &windows).hash(&mut hasher);
        hasher.finish()
    };
    if let Ok(cache) = CACHE.lock()
        && let Some((cached, tree)) = cache.get(screen)
        && *cached == key
    {
        if let Ok(mut stamps) = STAMPS.lock() {
            stamps.insert(screen.to_owned(), stamp);
        }
        return Some(tree.clone());
    }
    let language = settings
        .values
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or("en")
        .to_owned();
    let items_base = {
        let mut hasher = fluxa_ui::FastHasher::default();
        (screen, &skin.manifest.id, &skin.manifest.version, &language).hash(&mut hasher);
        std::mem::discriminant(&home.form_factor).hash(&mut hasher);
        hasher.finish()
    };
    let cached = DATA.lock().ok().and_then(|data| {
        data.get(screen)
            .filter(|(cached, _)| *cached == base)
            .map(|(_, data)| data.clone())
    });
    let data = match cached {
        Some(data) => data,
        None => {
            let active = match route {
                Route::Calendar => 1,
                Route::Library if library.downloads_open => 2,
                _ => 0,
            };
            let sections: Vec<_> = [
                ("library", "nav.library"),
                ("calendar", "nav.calendar"),
                ("downloads", "library.downloads"),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (id, label))| {
                json!({
                    "id": id,
                    "key": "section",
                    "label": fluxa_ui::localized(label, &language),
                    "selected": index == active,
                })
            })
            .collect();
            let mut home_data = serde_json::to_value(home).ok()?;
            home_data["heroSlides"] = slides;
            home_data["addonsPage"] = json!(fluxa_ui::addons_page());
            fluxa_ui::set_top_numbers(settings.top_numbers());
            let landscape = settings.poster_landscape();
            for (row, data) in home.rows.iter().zip(home_data["rows"].as_array_mut()?) {
                for (index, card) in data["cards"].as_array_mut()?.iter_mut().enumerate() {
                    if landscape && row.kind != fluxa_ui::HomeRowKind::Collection {
                        card["shape"] = json!("landscape");
                    }
                    if let Some(rank) = fluxa_ui::poster_rank(&row.title, row.kind, index) {
                        card["rank"] = json!(rank);
                    }
                    decorate(card, &row.cards[index], library);
                }
            }
            for (source, card) in home.cards.iter().zip(home_data["cards"].as_array_mut()?) {
                decorate(card, source, library);
            }
            fluxa_ui::register_cards(
                home.cards
                    .iter()
                    .chain(home.rows.iter().flat_map(|row| row.cards.iter())),
            );
            home_data["categories"] = json!(
                home.categories
                    .iter()
                    .map(|category| json!({
                        "node": category.node(),
                        "label": category.label_key(),
                        "active": *category == home.active_category,
                    }))
                    .collect::<Vec<_>>()
            );
            home_data["lift"] = json!(if home.poster_lift { "true" } else { "" });
            home_data["hoverDelay"] = if home.poster_hover_preview && !home.expanded_posters {
                json!(home.hover_preview_delay.to_string())
            } else {
                json!("")
            };
            home_data["expandDelay"] = if home.expanded_posters {
                json!(home.expanded_posters_delay.to_string())
            } else {
                json!("")
            };
            let mut screen_data = Some(screen_data());
            let mut data = json!({
                "home": home_data,
                "library": Value::Null,
                "discover": Value::Null,
                "calendar": Value::Null,
                "detail": if matches!(extra, Extra::Modal) { extra_data.clone() } else { Value::Null },
                "profiles": if matches!(extra, Extra::Profiles(_)) { extra_data.clone() } else { Value::Null },
                "player": if matches!(extra, Extra::Player(_)) { extra_data.clone() } else { Value::Null },
                "folder": if matches!(extra, Extra::Folder(..)) { extra_data.clone() } else { Value::Null },
                "guide": if matches!(extra, Extra::Guide(_)) { extra_data.clone() } else { Value::Null },
                "welcome": if matches!(extra, Extra::Welcome(_)) { extra_data.clone() } else { Value::Null },
                "menu": if matches!(extra, Extra::Menu(_)) { extra_data.clone() } else { Value::Null },
                "toast": if matches!(extra, Extra::Toast(_) | Extra::PlayerToast(_)) { extra_data.clone() } else { Value::Null },
                "tip": if matches!(extra, Extra::Tip(_)) { extra_data.clone() } else { Value::Null },
                "update": if matches!(extra, Extra::Update(_)) { extra_data.clone() } else { Value::Null },
                "window": if matches!(extra, Extra::Window(_)) { extra_data.clone() } else { Value::Null },
                "mini": if matches!(extra, Extra::Mini(_)) { extra_data.clone() } else { Value::Null },
                "sources": if matches!(extra, Extra::Sources(_)) { extra_data.clone() } else { Value::Null },
                "recommendations": if matches!(extra, Extra::Recommendations(_)) { extra_data.clone() } else { Value::Null },
                "dialog": if matches!(extra, Extra::Dialog(_)) { extra_data.clone() } else { Value::Null },
                "watch": if matches!(extra, Extra::Watch(_)) { extra_data } else { Value::Null },
                "const": skin.constants(),
                "sections": sections,
                "settings": settings.values,
                "prefs": if route == Route::Settings { prefs_data(settings, home.form_factor.into(), &fields) } else { Value::Null },
                "fields": fields,
                "nav": nav,
                "ext": ext::snapshot(),
            });
            let slot = match route {
                Route::Library => Some("library"),
                Route::Discover => Some("discover"),
                Route::Calendar => Some("calendar"),
                Route::Detail if !matches!(extra, Extra::Modal) => Some("detail"),
                _ => None,
            };
            if let (Some(slot), Some(value)) = (slot, screen_data.take()) {
                data[slot] = value;
            }
            let data = Arc::new(data);
            if let Ok(mut cache) = DATA.lock() {
                cache.insert(screen.to_owned(), (base, data.clone()));
            }
            data
        }
    };
    let resolved = {
        let mut caches = RESOLVE.lock().ok()?;
        let cache = caches
            .entry(screen.to_owned())
            .or_insert_with(|| (fluxa_skin::ResolveCache::new(), 0));
        if cache.1 != items_base {
            cache.0.clear_items();
            cache.1 = items_base;
        }
        cache.0.windows = windows;
        cache.0.data_stamp = base;
        cache.0.language = language.clone();
        cache.0.translate = Some(fluxa_ui::localized);
        skin.resolve(screen, &data, &mut cache.0)?
    };
    match resolved {
        Ok(tree) => {
            let tree = Arc::new(tree);
            if let Ok(mut cache) = CACHE.lock() {
                cache.insert(screen.to_owned(), (key, tree.clone()));
            }
            if let Ok(mut stamps) = STAMPS.lock() {
                stamps.insert(screen.to_owned(), stamp);
            }
            Some(tree)
        }
        Err(error) => {
            let message = format!("skin {}: {screen}: {error}", skin.manifest.id);
            if let Ok(mut last) = FAILED.lock()
                && last.as_deref() != Some(&message)
            {
                crate::host_log(&message);
                *last = Some(message);
            }
            None
        }
    }
}

pub(super) fn prefs_data(
    settings: &SettingsModel,
    form: fluxa_ui::UiFormFactor,
    fields: &HashMap<String, String>,
) -> Value {
    let mut data = fluxa_ui::settings_data(settings, form);
    if fluxa_ui::SETTINGS_SECTIONS
        .get(settings.active_section)
        .is_some_and(|section| section.title == "Theme")
    {
        data["skins"] = manager_data(settings.language());
    }
    if let Some(inputs) = data["inputs"].as_array_mut() {
        for input in inputs {
            let typed = input["key"].as_str().and_then(|key| fields.get(key));
            if let Some(typed) = typed {
                input["value"] = json!(typed);
            }
        }
    }
    if let Some(blocks) = data["blocks"].as_array_mut() {
        for row in blocks
            .iter_mut()
            .filter_map(|group| group["rows"].as_array_mut())
            .flatten()
        {
            let typed = row["key"]
                .as_str()
                .filter(|_| matches!(row["kind"].as_str(), Some("form" | "field")))
                .and_then(|key| fields.get(key));
            if let Some(typed) = typed {
                row["value"] = json!(typed);
            }
        }
    }
    data
}
