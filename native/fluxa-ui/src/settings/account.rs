use super::*;
use crate::draw_text;

pub(super) const ACCOUNT_SERVICES: [(&str, &str); 4] = [
    ("Trakt", "traktAccessToken"),
    ("Simkl", "simklAccessToken"),
    ("AniList", "anilistAccessToken"),
    ("MDBList", "mdblistAccessToken"),
];
pub const SOURCE_TOKENS: [(&str, &str); 6] = [
    ("nuvio", "nuvioAccessToken"),
    ("trakt", "traktAccessToken"),
    ("simkl", "simklAccessToken"),
    ("anilist", "anilistAccessToken"),
    ("mdblist", "mdblistAccessToken"),
    ("stremio", "stremioAuthKey"),
];

pub const ACCOUNT_PROVIDERS: [&str; 4] = ["trakt", "simkl", "mdblist", "anilist"];
pub(super) const ACCOUNT_SOURCES: [(&str, &str); 3] = [
    (
        "settings.integration_library_source",
        "integrationLibrarySource",
    ),
    (
        "settings.continue_watching_source",
        "continueWatchingSource",
    ),
    ("settings.mdblist_library_list", "mdblistLibraryListId"),
];

const SYNC_SOURCES: usize = 2;
pub(super) const MDBLIST_LIST_SOURCE: usize = 2;

fn service_rows(mut metrics: UiMetrics) -> UiMetrics {
    let extra = metrics.settings_service_row_extra;
    metrics.settings_row_height += extra;
    metrics.settings_row_spacing += extra;
    metrics
}

pub(super) fn account_group(
    flow: &mut egui::Ui,
    title: &str,
    metrics: UiMetrics,
    rows: impl FnOnce(&mut egui::Ui),
) -> Rect {
    let width = flow.max_rect().width();
    let heading = flow
        .allocate_space(Vec2::new(width, metrics.settings_group_heading_height))
        .1;
    let painter = flow.painter().clone();
    group_heading(&painter, heading, heading.top(), title, metrics);
    crate::kit::view()
        .class("card")
        .show(flow, |card| {
            card.add_space(Space::S6.px(&metrics));
            rows(card);
            card.add_space(Space::S16.px(&metrics));
        })
        .response
        .rect
}

fn row_gap(metrics: UiMetrics) -> f32 {
    metrics.settings_row_spacing - metrics.settings_row_height
}

pub(super) fn chevron(
    painter: &egui::Painter,
    tip: Pos2,
    stroke: egui::Stroke,
    metrics: UiMetrics,
) {
    let depth = Space::S6.px(&metrics);
    painter.line_segment([tip + Vec2::new(-depth, -depth), tip], stroke);
    painter.line_segment([tip + Vec2::new(-depth, depth), tip], stroke);
}

pub(super) fn pill_text_width(
    painter: &egui::Painter,
    text: &str,
    size: f32,
    metrics: UiMetrics,
) -> f32 {
    crate::paint::text_width(painter, text, FontId::proportional(size))
        + Space::S16.px(&metrics) * 2.0
}

pub(super) fn account_row(
    card: &mut egui::Ui,
    painter: &egui::Painter,
    metrics: UiMetrics,
    divided: bool,
) -> Rect {
    let gap = row_gap(metrics);
    if divided {
        card.add_space(gap);
    }
    let width = card.max_rect().width();
    let slot = card
        .allocate_space(Vec2::new(width, metrics.settings_row_height))
        .1;
    let row = slot.shrink2(Vec2::new(metrics.settings_row_inset, 0.0));
    if divided {
        let y = row.top() - gap * 0.5;
        painter.line_segment(
            [Pos2::new(row.left(), y), Pos2::new(row.right(), y)],
            egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("outline")),
        );
    }
    row
}

pub(super) fn source_label(source: &str, language: &str) -> String {
    match source {
        "mdblist" => localized("settings.service.mdblist", language),
        _ => localized_or(&format!("settings.option.{source}"), source, language),
    }
}

pub fn account_source_state(
    settings: &SettingsModel,
    key: &str,
) -> (Vec<(String, String)>, String, String) {
    let language = settings.language();
    let options = if key == "trickplaySource" {
        trickplay_sources(settings)
    } else if key == "heroBadgeSource" || key == "externalAudioSource" {
        addon_sources(settings)
    } else if key == "mdblistLibraryListId" {
        if account_connected(settings, "mdblistAccessToken")
            || account_connected(settings, "mdblistApiKey")
        {
            std::iter::once((
                String::new(),
                localized("settings.mdblist_watchlist", language),
            ))
            .chain(settings.mdblist_lists.iter().cloned())
            .collect()
        } else {
            Vec::new()
        }
    } else {
        SOURCE_TOKENS
            .into_iter()
            .filter(|(_, token_key)| account_connected(settings, token_key))
            .map(|(source, _)| (source.to_owned(), source_label(source, language)))
            .collect::<Vec<_>>()
    };
    let current = settings
        .value(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let label = match options.iter().find(|(source, _)| *source == current) {
        Some((_, label)) => label.clone(),
        None if matches!(
            key,
            "trickplaySource" | "heroBadgeSource" | "externalAudioSource"
        ) =>
        {
            localized("settings.option.none", language)
        }
        None if options.is_empty() => localized("settings.not_connected", language),
        None => localized("settings.option.none", language),
    };
    (options, current, label)
}

pub fn account_source(index: usize) -> Option<(&'static str, &'static str)> {
    ACCOUNT_SOURCES
        .get(index)
        .copied()
        .or((index == TRICKPLAY_SOURCE_INDEX).then_some(TRICKPLAY_SOURCE))
        .or((index == HERO_BADGE_SOURCE_INDEX).then_some(HERO_BADGE_SOURCE))
        .or((index == EXTERNAL_AUDIO_SOURCE_INDEX).then_some(EXTERNAL_AUDIO_SOURCE))
}

const TRICKPLAY_SOURCE_INDEX: usize = 3;
const TRICKPLAY_SOURCE: (&str, &str) = ("settings.trickplay_source", "trickplaySource");
pub const HERO_BADGE_SOURCE_INDEX: usize = 4;
const HERO_BADGE_SOURCE: (&str, &str) = ("settings.hero_badge_source", "heroBadgeSource");
pub const EXTERNAL_AUDIO_SOURCE_INDEX: usize = 5;
const EXTERNAL_AUDIO_SOURCE: (&str, &str) =
    ("settings.external_audio_source", "externalAudioSource");

pub(super) fn source_group(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    flow: &mut egui::Ui,
    (index, title): (usize, &str),
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let language = settings.language();
    account_group(flow, &localized(title, language), metrics, |card| {
        let painter = card.painter().clone();
        let row = account_row(card, &painter, metrics, false);
        if let Some(source) = account_source(index) {
            source_row(
                context,
                &painter,
                viewport,
                settings,
                row,
                NODE_SETTINGS_ACCOUNT_SOURCE_BASE + index as u64,
                source,
                metrics,
                layout,
            );
        }
    });
}

fn addon_sources(settings: &SettingsModel) -> Vec<(String, String)> {
    let text = |value: &serde_json::Value, key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned)
    };
    let addons = settings
        .addons
        .iter()
        .filter(|addon| addon_enabled(addon))
        .filter_map(|addon| {
            let url = addon_transport_url(addon)?;
            let manifest = addon.get("manifest").unwrap_or(addon);
            let name = text(manifest, "name").unwrap_or_else(|| url.to_owned());
            Some((format!("addon:{url}"), name))
        });
    let plugins = settings
        .plugins
        .get("scrapers")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|scraper| {
            let id = text(scraper, "id")?;
            let name = text(scraper, "name").unwrap_or_else(|| id.clone());
            Some((format!("plugin:{id}"), name))
        });
    addons.chain(plugins).collect()
}

fn trickplay_sources(settings: &SettingsModel) -> Vec<(String, String)> {
    let language = settings.language();
    std::iter::once((
        "stream".to_owned(),
        localized("settings.trickplay_source_stream", language),
    ))
    .chain(std::iter::once((
        "server".to_owned(),
        localized("settings.trickplay_source_server", language),
    )))
    .chain(addon_sources(settings))
    .collect()
}

pub(super) fn account_connected(settings: &SettingsModel, key: &str) -> bool {
    settings
        .profile
        .get(key)
        .or_else(|| settings.value(key))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|token| !token.is_empty())
}

fn source_row(
    context: &egui::Context,
    painter: &egui::Painter,
    viewport: Viewport,
    settings: &SettingsModel,
    row: Rect,
    node: u64,
    (label_key, key): (&'static str, &'static str),
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let language = settings.language();
    let compact = viewport.is_compact();
    let sheet = compact || viewport.is_tv();
    let label_size = metrics.settings_label_size;
    let (options, current, current_label) = account_source_state(settings, key);
    let mut selected = current.clone();
    if sheet {
        layout.focusable.push((node, row));
        let title = localized(label_key, language);
        let value_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
        let tip = row.right_center() - Vec2::new(Space::S6.px(&metrics), 0.0);
        if viewport.platform == UiPlatform::Android {
            let text_top =
                row.center().y - (label_size + Space::S4.px(&metrics) + value_font.size) * 0.5;
            draw_text(
                painter,
                Pos2::new(row.left() + Space::S4.px(&metrics), text_top),
                Align2::LEFT_TOP,
                title,
                crate::fonts::regular(label_size),
                metrics.text_primary,
                None,
            );
            draw_text(
                painter,
                Pos2::new(
                    row.left() + Space::S4.px(&metrics),
                    text_top + label_size + Space::S4.px(&metrics),
                ),
                Align2::LEFT_TOP,
                current_label,
                value_font,
                metrics.text_secondary,
                None,
            );
        } else {
            draw_text(
                painter,
                row.left_center() + Vec2::new(Space::S4.px(&metrics), 0.0),
                Align2::LEFT_CENTER,
                title,
                crate::fonts::regular(label_size),
                metrics.text_primary,
                None,
            );
            draw_text(
                painter,
                tip - Vec2::new(Space::S16.px(&metrics), 0.0),
                Align2::RIGHT_CENTER,
                current_label,
                value_font,
                metrics.text_secondary,
                None,
            );
            let stroke = egui::Stroke::new(stroke::icon(), crate::stylesheet::color("glyph"));
            chevron(painter, tip, stroke, metrics);
        }
        return;
    }
    draw_text(
        painter,
        row.left_center() + Vec2::new(Space::S4.px(&metrics), 0.0),
        Align2::LEFT_CENTER,
        localized(label_key, language),
        FontId::proportional(label_size),
        metrics.text_primary,
        None,
    );
    let value_width = metrics.settings_service_value_width;
    let value_rect = Rect::from_min_size(
        Pos2::new(
            row.right() - Space::S2.px(&metrics) - value_width,
            row.center().y,
        ),
        Vec2::new(
            value_width,
            metrics
                .screen_control_height
                .min(row.height() - Space::S4.px(&metrics)),
        ),
    )
    .translate(Vec2::new(
        0.0,
        -metrics
            .screen_control_height
            .min(row.height() - Space::S4.px(&metrics))
            * 0.5,
    ));
    components::placed_front(
        context,
        Id::new(("fluxa-settings-account-choice", key)),
        value_rect.min,
        |ui| {
            ui.set_min_size(value_rect.size());
            ui.set_max_size(value_rect.size());
            let response = components::choice_field(
                ui,
                Id::new(("settings-account", key)),
                value_rect.size(),
                metrics.settings_row_value_size_desktop,
                &current_label,
                &options,
                &mut selected,
                value_rect.width(),
                !options.is_empty(),
                false,
                metrics,
            );
            layout.focusable.push((node, response.rect));
        },
    );
    if selected != current {
        layout.setting_change = Some(((key).to_owned(), serde_json::Value::String(selected)));
    }
}

pub(super) fn draw_account(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let compact = viewport.is_compact();
    let sheet = compact || viewport.is_tv();
    let label_size = metrics.settings_label_size;
    let connected = |key: &str| account_connected(settings, key);
    let pill_size = (label_size - 3.0).max(13.0);

    flow_area(
        context,
        "fluxa-account-page",
        rect,
        rect.top() + metrics.settings_appearance_header_height,
        |flow| {
            account_group(
                flow,
                &localized("settings.group.account_profile", language),
                metrics,
                |card| {
                    let painter = card.painter().clone();
                    let header = card
                        .allocate_space(Vec2::new(
                            card.max_rect().width(),
                            metrics.settings_profile_header_height,
                        ))
                        .1;
                    let name = settings
                        .profile
                        .get("name")
                        .or_else(|| settings.profile.get("displayName"))
                        .and_then(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| localized("settings.account_empty", language));
                    let avatar_radius = metrics.profile_avatar_tile * 0.5;
                    let avatar = Rect::from_center_size(
                        Pos2::new(
                            header.left() + Space::S20.px(&metrics) + avatar_radius,
                            header.center().y,
                        ),
                        Vec2::splat(metrics.profile_avatar_tile),
                    );
                    let avatar_url = settings
                        .profile
                        .get("avatarUrl")
                        .and_then(serde_json::Value::as_str)
                        .filter(|url| !url.starts_with("data:"));
                    let dom_avatar = components::dom_avatar(
                        &painter,
                        assets,
                        avatar.center(),
                        avatar_radius,
                        avatar_url,
                    );
                    match (!dom_avatar)
                        .then(|| {
                            assets.texture_for(avatar_url, [128, 128], ArtworkPriority::Visible)
                        })
                        .flatten()
                    {
                        _ if dom_avatar => {}
                        Some(texture) => {
                            components::rounded_image(
                                &painter,
                                texture,
                                avatar,
                                avatar_radius,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            components::animated_avatar(
                                &painter,
                                assets,
                                avatar.center(),
                                avatar_radius,
                                avatar_radius,
                                avatar_url,
                            );
                        }
                        None if avatar_url.is_some() => {
                            components::loading_circle(&painter, avatar.center(), avatar_radius);
                        }
                        None => {
                            painter.circle_filled(
                                avatar.center(),
                                avatar_radius,
                                crate::stylesheet::color("track"),
                            );
                        }
                    }
                    painter.circle_stroke(
                        avatar.center(),
                        avatar_radius,
                        egui::Stroke::new(stroke::emphasis(), crate::stylesheet::color("dim")),
                    );
                    let name_left = avatar.right() + Space::S16.px(&metrics);
                    let mut name_job = egui::text::LayoutJob::simple_singleline(
                        name,
                        crate::fonts::regular(label_size + 8.0),
                        text_primary(),
                    );
                    name_job.wrap.max_width =
                        (header.right() - Space::S20.px(&metrics) - name_left).max(1.0);
                    name_job.wrap.max_rows = 1;
                    name_job.wrap.break_anywhere = true;
                    let name_galley = painter.layout_job(name_job);
                    painter.galley(
                        Pos2::new(name_left, avatar.center().y - name_galley.size().y * 0.5),
                        name_galley,
                        text_primary(),
                    );

                    let profile_width = card.max_rect().width();
                    if !compact {
                        let row = card
                            .allocate_space(Vec2::new(
                                profile_width,
                                metrics.settings_profile_row_height,
                            ))
                            .1;
                        profile_row(
                            context,
                            layout,
                            NODE_SETTINGS_SWITCH_PROFILE,
                            row,
                            &localized("settings.switch_profiles", language),
                            Some(&localized("settings.switch_profiles_desc", language)),
                            label_size,
                            metrics,
                        );
                    }
                    let row = card
                        .allocate_space(Vec2::new(
                            profile_width,
                            metrics.settings_profile_row_height,
                        ))
                        .1;
                    profile_row(
                        context,
                        layout,
                        NODE_SETTINGS_LOG_OUT,
                        row,
                        &localized("auto.log_out", language),
                        None,
                        label_size,
                        metrics,
                    );
                },
            );

            flow.add_space(metrics.settings_group_gap);
            account_group(
                flow,
                &localized("settings.group.account_services", language),
                metrics,
                |card| {
                    let painter = card.painter().clone();
                    let logo_height = metrics.settings_service_logo_height;
                    for (index, (label, key)) in ACCOUNT_SERVICES.iter().enumerate() {
                        let row = account_row(card, &painter, service_rows(metrics), index > 0);
                        let mut label_left = Space::S4.px(&metrics);
                        if let Some((texture, aspect)) = assets.logo(&label.to_lowercase()) {
                            let size = aspect * logo_height;
                            painter.image(
                                texture,
                                Rect::from_min_size(
                                    row.left_center()
                                        + Vec2::new(Space::S4.px(&metrics), -logo_height * 0.5),
                                    size,
                                ),
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            label_left += size.x + metrics.settings_service_logo_gap;
                        }
                        let provider = label.to_lowercase();
                        let on = connected(key)
                            && !settings
                                .tracker_names
                                .iter()
                                .any(|(name, handle)| *name == provider && handle.is_empty());
                        let prompt = settings
                            .account_auth
                            .as_ref()
                            .filter(|prompt| prompt.provider == provider);
                        let status = match prompt {
                            Some(prompt) if prompt.failed => {
                                localized("settings.account_sign_in_failed", language)
                            }
                            Some(prompt) => format!(
                                "{} {}  ·  {}",
                                localized("trakt.device.enter_code", language),
                                prompt.code,
                                prompt.url
                            ),
                            None => match settings
                                .tracker_names
                                .iter()
                                .find(|(name, _)| *name == provider)
                                .filter(|_| on)
                            {
                                Some((_, name)) => {
                                    localized("settings.connected_as", language).replace("%s", name)
                                }
                                None => localized(
                                    if on {
                                        "settings.connected"
                                    } else {
                                        "settings.not_connected"
                                    },
                                    language,
                                ),
                            },
                        };
                        if sheet {
                            let slot = ACCOUNT_PROVIDERS.iter().position(|name| *name == provider);
                            if let Some(slot) = slot {
                                layout
                                    .focusable
                                    .push((NODE_SETTINGS_ACCOUNT_BASE + slot as u64, row));
                            }
                            let status_font =
                                crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
                            let tip = row.right_center() - Vec2::new(Space::S6.px(&metrics), 0.0);
                            let right = if slot.is_some() {
                                tip.x - Space::S16.px(&metrics)
                            } else {
                                row.right() - Space::S4.px(&metrics)
                            };
                            if viewport.platform == UiPlatform::Android {
                                let text_top = row.center().y
                                    - (label_size + Space::S4.px(&metrics) + status_font.size)
                                        * 0.5;
                                draw_text(
                                    &painter,
                                    Pos2::new(row.left() + label_left, text_top),
                                    Align2::LEFT_TOP,
                                    *label,
                                    crate::fonts::regular(label_size),
                                    metrics.text_primary,
                                    None,
                                );
                                draw_text(
                                    &painter,
                                    Pos2::new(
                                        row.left() + label_left,
                                        text_top + label_size + Space::S4.px(&metrics),
                                    ),
                                    Align2::LEFT_TOP,
                                    truncate_to_width(
                                        &painter,
                                        &status,
                                        &status_font,
                                        right - row.left() - label_left,
                                    ),
                                    status_font,
                                    ink(if on { 200 } else { 160 }),
                                    None,
                                );
                            } else {
                                draw_text(
                                    &painter,
                                    row.left_center() + Vec2::new(label_left, 0.0),
                                    Align2::LEFT_CENTER,
                                    *label,
                                    crate::fonts::regular(label_size),
                                    metrics.text_primary,
                                    None,
                                );
                                draw_text(
                                    &painter,
                                    Pos2::new(right, row.center().y),
                                    Align2::RIGHT_CENTER,
                                    status,
                                    status_font,
                                    metrics.text_secondary,
                                    None,
                                );
                            }
                            if slot.is_some() {
                                let stroke = egui::Stroke::new(
                                    stroke::icon(),
                                    crate::stylesheet::color("glyph"),
                                );
                                chevron(&painter, tip, stroke, metrics);
                            }
                            continue;
                        }
                        let mut text_right = row.right() - Space::S4.px(&metrics);
                        if let Some(slot) =
                            ACCOUNT_PROVIDERS.iter().position(|name| *name == provider)
                        {
                            let action = localized(
                                if on {
                                    "settings.account_disconnect"
                                } else {
                                    "settings.account_connect"
                                },
                                language,
                            );
                            let width = pill_text_width(&painter, &action, pill_size, metrics);
                            let button = Rect::from_center_size(
                                Pos2::new(
                                    row.right() - Space::S4.px(&metrics) - width * 0.5,
                                    row.center().y,
                                ),
                                Vec2::new(
                                    width,
                                    metrics
                                        .screen_control_height
                                        .min(row.height() - Space::S4.px(&metrics)),
                                ),
                            );
                            pill_button_styled(
                                context,
                                layout,
                                NODE_SETTINGS_ACCOUNT_BASE + slot as u64,
                                button,
                                &action,
                                pill_size,
                                !on,
                            );
                            text_right = button.left() - Space::S16.px(&metrics);
                        }
                        let name_font = crate::fonts::regular(label_size + 1.0);
                        let status_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
                        draw_text(
                            &painter,
                            row.left_center() + Vec2::new(label_left, -Space::S2.px(&metrics)),
                            Align2::LEFT_BOTTOM,
                            *label,
                            name_font,
                            metrics.text_primary,
                            None,
                        );
                        if prompt.is_some_and(|prompt| !prompt.failed)
                            && let Some(slot) =
                                ACCOUNT_PROVIDERS.iter().position(|name| *name == provider)
                        {
                            let link = Rect::from_min_max(
                                Pos2::new(
                                    row.left() + label_left,
                                    row.center().y + Space::S4.px(&metrics),
                                ),
                                Pos2::new(text_right, row.bottom()),
                            );
                            layout
                                .focusable
                                .push((NODE_SETTINGS_ACCOUNT_BASE + slot as u64, link));
                        }
                        draw_text(
                            &painter,
                            row.left_center() + Vec2::new(label_left, Space::S4.px(&metrics)),
                            Align2::LEFT_TOP,
                            truncate_to_width(
                                &painter,
                                &status,
                                &status_font,
                                text_right - row.left() - label_left,
                            ),
                            status_font,
                            metrics.text_secondary,
                            None,
                        );
                    }
                },
            );

            flow.add_space(metrics.settings_group_gap);
            account_group(
                flow,
                &localized("settings.group.account_sync", language),
                metrics,
                |card| {
                    let painter = card.painter().clone();
                    for (index, &(label_key, key)) in
                        ACCOUNT_SOURCES.iter().take(SYNC_SOURCES).enumerate()
                    {
                        let row = account_row(card, &painter, metrics, index > 0);
                        source_row(
                            context,
                            &painter,
                            viewport,
                            settings,
                            row,
                            NODE_SETTINGS_ACCOUNT_SOURCE_BASE + index as u64,
                            (label_key, key),
                            metrics,
                            layout,
                        );
                    }
                },
            );
        },
    );
}

fn profile_row(
    context: &egui::Context,
    layout: &mut HomeLayout,
    node: u64,
    rect: Rect,
    title: &str,
    description: Option<&str>,
    label_size: f32,
    metrics: UiMetrics,
) {
    let destructive = description.is_none();
    components::placed_front(
        context,
        Id::new(("fluxa-settings-profile-row", node)),
        rect.min,
        |ui| {
            let (_, response) = ui.allocate_exact_size(rect.size(), Sense::click());
            let painter = ui.painter();
            painter.line_segment(
                [
                    rect.left_top() + Vec2::new(Space::S20.px(&metrics), 0.0),
                    rect.right_top() - Vec2::new(Space::S20.px(&metrics), 0.0),
                ],
                egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("divider")),
            );
            if response.hovered() {
                painter.rect_filled(
                    rect.shrink2(Vec2::new(Space::S8.px(&metrics), Space::S4.px(&metrics))),
                    Radius::Md.px(&metrics),
                    crate::stylesheet::color("track"),
                );
            }
            let color = if destructive {
                danger_color()
            } else {
                text_primary()
            };
            let left = rect.left() + Space::S20.px(&metrics);
            let subtitle = metrics.screen_card_subtitle_size;
            let title_y = match description {
                Some(_) => rect.center().y - subtitle * 0.5 - 1.0,
                None => rect.center().y,
            };
            draw_text(
                painter,
                Pos2::new(left, title_y),
                if description.is_some() {
                    Align2::LEFT_BOTTOM
                } else {
                    Align2::LEFT_CENTER
                },
                title,
                crate::fonts::regular(label_size),
                color,
                None,
            );
            if let Some(description) = description {
                draw_text(
                    painter,
                    Pos2::new(left, rect.center().y + Space::S2.px(&metrics)),
                    Align2::LEFT_TOP,
                    truncate_to_width(
                        painter,
                        description,
                        &crate::fonts::regular(subtitle),
                        rect.width() - Space::S48.px(&metrics) - Space::S32.px(&metrics),
                    ),
                    crate::fonts::regular(subtitle),
                    metrics.text_secondary,
                    None,
                );
                let tip = rect.right_center() - Vec2::new(Space::S24.px(&metrics), 0.0);
                let stroke = egui::Stroke::new(stroke::icon(), crate::stylesheet::color("glyph"));
                chevron(painter, tip, stroke, metrics);
            }
            components::focus_action(layout, node, &response);
        },
    );
}
