use super::*;

pub(super) const ACCOUNT_PROFILE_CARD_HEIGHT: f32 = 92.0;
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
pub(super) const ACCOUNT_SOURCES: [(&str, &str); 2] = [
    (
        "settings.integration_library_source",
        "integrationLibrarySource",
    ),
    (
        "settings.continue_watching_source",
        "continueWatchingSource",
    ),
];

pub(super) fn account_height(metrics: UiMetrics) -> f32 {
    APPEARANCE_PAGE_HEADER_HEIGHT
        + (APPEARANCE_GROUP_HEADING_HEIGHT + APPEARANCE_GROUP_GAP) * 4.0
        + ACCOUNT_PROFILE_CARD_HEIGHT
        + settings_group_card_height(ACCOUNT_SERVICES.len(), metrics)
        + settings_group_card_height(ACCOUNT_SOURCES.len(), metrics)
        + FIELD_HEIGHT * 2.0
}

pub(super) fn account_group(
    painter: &egui::Painter,
    rect: Rect,
    top: f32,
    height: f32,
    title: &str,
    compact: bool,
    metrics: UiMetrics,
) -> Rect {
    if compact {
        painter.text(
            Pos2::new(rect.left() + 16.0, top),
            Align2::LEFT_TOP,
            title,
            crate::fonts::regular(15.0),
            metrics.text_muted,
        );
    } else {
        painter.text(
            Pos2::new(rect.left() + 8.0, top),
            Align2::LEFT_TOP,
            title.to_uppercase(),
            crate::fonts::regular(13.0),
            metrics.text_secondary,
        );
    }
    let card = Rect::from_min_size(
        Pos2::new(rect.left(), top + APPEARANCE_GROUP_HEADING_HEIGHT),
        Vec2::new(rect.width(), height),
    );
    painter.rect_filled(card, metrics.card_radius, metrics.surface);
    painter.rect_stroke(
        card,
        metrics.card_radius,
        egui::Stroke::new(1.0, metrics.border),
        egui::StrokeKind::Inside,
    );
    card
}

pub(super) fn account_row(card: Rect, index: usize, metrics: UiMetrics) -> Rect {
    Rect::from_min_size(
        card.left_top()
            + Vec2::new(
                metrics.settings_row_inset,
                6.0 + index as f32 * metrics.settings_row_spacing,
            ),
        Vec2::new(
            card.width() - metrics.settings_row_inset * 2.0,
            metrics.settings_row_height,
        ),
    )
}

pub(super) fn account_divider(painter: &egui::Painter, row: Rect, metrics: UiMetrics) {
    let y = row.top() - (metrics.settings_row_spacing - metrics.settings_row_height) * 0.5;
    painter.line_segment(
        [Pos2::new(row.left(), y), Pos2::new(row.right(), y)],
        egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
    );
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
    let options = SOURCE_TOKENS
        .into_iter()
        .filter(|(_, token_key)| account_connected(settings, token_key))
        .map(|(source, _)| (source.to_owned(), source_label(source, language)))
        .collect::<Vec<_>>();
    let current = settings
        .value(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let label = match options.iter().find(|(source, _)| *source == current) {
        Some((_, label)) => label.clone(),
        None if options.is_empty() => localized("settings.not_connected", language),
        None => localized("settings.option.none", language),
    };
    (options, current, label)
}

pub fn account_source(index: usize) -> Option<(&'static str, &'static str)> {
    ACCOUNT_SOURCES.get(index).copied()
}

pub(super) fn account_connected(settings: &SettingsModel, key: &str) -> bool {
    settings
        .profile
        .get(key)
        .or_else(|| settings.value(key))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|token| !token.is_empty())
}

pub(super) fn draw_account(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let compact = viewport.is_compact();
    let sheet = compact || viewport.is_tv();
    let painter = context.layer_painter(egui::LayerId::background());
    let label_size = if compact {
        metrics.screen_body_size
    } else {
        metrics.settings_row_label_size_desktop
    };
    let connected = |key: &str| account_connected(settings, key);

    let mut top = rect.top()
        + if compact {
            8.0
        } else {
            APPEARANCE_PAGE_HEADER_HEIGHT
        };
    let card = account_group(
        &painter,
        rect,
        top,
        ACCOUNT_PROFILE_CARD_HEIGHT,
        &localized("settings.group.account_profile", language),
        compact,
        metrics,
    );
    let name = settings
        .profile
        .get("name")
        .or_else(|| settings.profile.get("displayName"))
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| localized("settings.account_empty", language));
    let avatar = Rect::from_center_size(
        Pos2::new(card.left() + 22.0 + 26.0, card.center().y),
        Vec2::splat(52.0),
    );
    let avatar_url = settings
        .profile
        .get("avatarUrl")
        .and_then(serde_json::Value::as_str)
        .filter(|url| !url.starts_with("data:"));
    match assets.cached_texture(avatar_url) {
        Some(texture) => {
            painter.add(
                egui::epaint::RectShape::filled(avatar, 26.0, Color32::WHITE)
                    .with_texture(texture, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))),
            );
        }
        None => {
            painter.circle_filled(avatar.center(), 26.0, Color32::from_white_alpha(22));
            let initials = name
                .split_whitespace()
                .take(2)
                .filter_map(|word| word.chars().next())
                .flat_map(char::to_uppercase)
                .collect::<String>();
            painter.text(
                avatar.center(),
                Align2::CENTER_CENTER,
                initials,
                FontId::proportional(20.0),
                Color32::WHITE,
            );
        }
    }
    let name_left = avatar.right() + 16.0;
    let mut name_right = card.right() - 20.0;
    let switch_label = localized("settings.switch_profiles", language);
    let switch_width = painter
        .layout_no_wrap(
            switch_label.clone(),
            FontId::proportional(label_size),
            Color32::WHITE,
        )
        .size()
        .x
        + 32.0;
    let switch_rect = Rect::from_center_size(
        Pos2::new(card.right() - 20.0 - switch_width * 0.5, card.center().y),
        Vec2::new(switch_width, 36.0),
    );
    if !compact {
        pill_button(
            context,
            layout,
            NODE_SETTINGS_SWITCH_PROFILE,
            switch_rect,
            &switch_label,
            label_size,
        );
        name_right = switch_rect.left() - 12.0;
    }
    let mut name_job = egui::text::LayoutJob::simple_singleline(
        name,
        crate::fonts::regular(label_size + 4.0),
        Color32::WHITE,
    );
    name_job.wrap.max_width = (name_right - name_left).max(1.0);
    name_job.wrap.max_rows = 1;
    name_job.wrap.break_anywhere = true;
    let name_galley = painter.layout_job(name_job);
    painter.galley(
        Pos2::new(name_left, card.center().y - name_galley.size().y * 0.5),
        name_galley,
        Color32::WHITE,
    );

    top = card.bottom() + APPEARANCE_GROUP_GAP;
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(ACCOUNT_SERVICES.len(), metrics),
        &localized("settings.group.account_services", language),
        compact,
        metrics,
    );
    for (index, (label, key)) in ACCOUNT_SERVICES.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
        let mut label_left = 4.0;
        if let Some((texture, aspect)) = assets.logo(&label.to_lowercase()) {
            let size = aspect * 22.0;
            painter.image(
                texture,
                Rect::from_min_size(row.left_center() + Vec2::new(4.0, -11.0), size),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            label_left += size.x + 12.0;
        }
        let on = connected(key);
        let provider = label.to_lowercase();
        let prompt = settings
            .account_auth
            .as_ref()
            .filter(|prompt| prompt.provider == provider);
        let status = match prompt {
            Some(prompt) if prompt.failed => localized("settings.account_sign_in_failed", language),
            Some(prompt) => format!(
                "{} {}  ·  {}",
                localized("trakt.device.enter_code", language),
                prompt.code,
                prompt.url
            ),
            None => localized(
                if on {
                    "settings.connected"
                } else {
                    "settings.not_connected"
                },
                language,
            ),
        };
        if sheet {
            let slot = ACCOUNT_PROVIDERS.iter().position(|name| *name == provider);
            if let Some(slot) = slot {
                layout
                    .focusable
                    .push((NODE_SETTINGS_ACCOUNT_BASE + slot as u64, row));
            }
            let status_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
            let tip = row.right_center() - Vec2::new(6.0, 0.0);
            let right = if slot.is_some() {
                tip.x - 16.0
            } else {
                row.right() - 4.0
            };
            if viewport.platform == UiPlatform::Android {
                let text_top = row.center().y - (label_size + 4.0 + status_font.size) * 0.5;
                painter.text(
                    Pos2::new(row.left() + label_left, text_top),
                    Align2::LEFT_TOP,
                    *label,
                    crate::fonts::regular(label_size),
                    metrics.text_primary,
                );
                painter.text(
                    Pos2::new(row.left() + label_left, text_top + label_size + 4.0),
                    Align2::LEFT_TOP,
                    truncate_to_width(
                        &painter,
                        &status,
                        &status_font,
                        right - row.left() - label_left,
                    ),
                    status_font,
                    Color32::from_white_alpha(if on { 200 } else { 160 }),
                );
            } else {
                painter.text(
                    row.left_center() + Vec2::new(label_left, 0.0),
                    Align2::LEFT_CENTER,
                    *label,
                    crate::fonts::regular(label_size),
                    metrics.text_primary,
                );
                painter.text(
                    Pos2::new(right, row.center().y),
                    Align2::RIGHT_CENTER,
                    status,
                    status_font,
                    metrics.text_secondary,
                );
            }
            if slot.is_some() && viewport.platform != UiPlatform::Android {
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(110));
                painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
            }
            continue;
        }
        painter.text(
            row.left_center() + Vec2::new(label_left, 0.0),
            Align2::LEFT_CENTER,
            *label,
            FontId::proportional(label_size),
            metrics.text_primary,
        );
        let mut status_right = 8.0;
        if let Some(slot) = ACCOUNT_PROVIDERS.iter().position(|name| *name == provider) {
            let action = localized(
                if on {
                    "settings.account_disconnect"
                } else {
                    "settings.account_connect"
                },
                language,
            );
            let width = painter
                .layout_no_wrap(
                    action.clone(),
                    FontId::proportional(label_size),
                    Color32::WHITE,
                )
                .size()
                .x
                + 32.0;
            let button = Rect::from_center_size(
                Pos2::new(row.right() - 4.0 - width * 0.5, row.center().y),
                Vec2::new(width, 32.0_f32.min(row.height() - 4.0)),
            );
            pill_button(
                context,
                layout,
                NODE_SETTINGS_ACCOUNT_BASE + slot as u64,
                button,
                &action,
                label_size,
            );
            status_right += width + 12.0;
        }
        painter.text(
            row.right_center() - Vec2::new(status_right, 0.0),
            Align2::RIGHT_CENTER,
            status,
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            Color32::from_white_alpha(if on { 200 } else { 110 }),
        );
    }

    top = card.bottom() + APPEARANCE_GROUP_GAP;
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(ACCOUNT_SOURCES.len(), metrics),
        &localized("settings.group.account_sync", language),
        compact,
        metrics,
    );
    for (index, (label_key, key)) in ACCOUNT_SOURCES.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
        let (options, current, current_label) = account_source_state(settings, key);
        let mut selected = current.clone();
        if sheet {
            let node = NODE_SETTINGS_ACCOUNT_SOURCE_BASE + index as u64;
            layout.focusable.push((node, row));
            let title = localized(label_key, language);
            let value_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
            let tip = row.right_center() - Vec2::new(6.0, 0.0);
            if viewport.platform == UiPlatform::Android {
                let text_top = row.center().y - (label_size + 4.0 + value_font.size) * 0.5;
                painter.text(
                    Pos2::new(row.left() + 4.0, text_top),
                    Align2::LEFT_TOP,
                    title,
                    crate::fonts::regular(label_size),
                    metrics.text_primary,
                );
                painter.text(
                    Pos2::new(row.left() + 4.0, text_top + label_size + 4.0),
                    Align2::LEFT_TOP,
                    current_label,
                    value_font,
                    metrics.text_secondary,
                );
            } else {
                painter.text(
                    row.left_center() + Vec2::new(4.0, 0.0),
                    Align2::LEFT_CENTER,
                    title,
                    crate::fonts::regular(label_size),
                    metrics.text_primary,
                );
                painter.text(
                    tip - Vec2::new(16.0, 0.0),
                    Align2::RIGHT_CENTER,
                    current_label,
                    value_font,
                    metrics.text_secondary,
                );
                let stroke = egui::Stroke::new(1.6, Color32::from_white_alpha(110));
                painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
            }
            continue;
        }
        painter.text(
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized(label_key, language),
            FontId::proportional(label_size),
            metrics.text_primary,
        );
        let value_width = if compact { 132.0 } else { 176.0 };
        let value_rect = Rect::from_min_size(
            Pos2::new(row.right() - 2.0 - value_width, row.center().y),
            Vec2::new(
                value_width,
                metrics.screen_control_height.min(row.height() - 4.0),
            ),
        )
        .translate(Vec2::new(
            0.0,
            -metrics.screen_control_height.min(row.height() - 4.0) * 0.5,
        ));
        egui::Area::new(Id::new(("fluxa-settings-account-choice", *key)))
            .constrain(false)
            .fixed_pos(value_rect.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                ui.set_min_size(value_rect.size());
                ui.set_max_size(value_rect.size());
                let response = components::choice_field(
                    ui,
                    Id::new(("settings-account", *key)),
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
                layout.focusable.push((
                    NODE_SETTINGS_ACCOUNT_SOURCE_BASE + index as u64,
                    response.rect,
                ));
            });
        if selected != current {
            layout.setting_change = Some(((*key).to_owned(), serde_json::Value::String(selected)));
        }
    }

    draw_field_group(
        context,
        settings,
        rect,
        card.bottom() + APPEARANCE_GROUP_GAP,
        &localized("settings.group.api_keys", language),
        &[1, 2],
        metrics,
        compact,
        layout,
    );
}
