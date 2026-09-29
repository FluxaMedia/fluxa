use super::*;

const CARD_PADDING: f32 = 16.0;
const LOGO: f32 = 48.0;
const CHIP_HEIGHT: f32 = 26.0;
const ACTION: f32 = 40.0;
const CARD_GAP: f32 = 12.0;
const SCRAPER_ROW: f32 = 64.0;

pub fn addon_action_node(index: usize, action: u64) -> u64 {
    NODE_SETTINGS_ADDON_ACTION_BASE + index as u64 * 4 + action
}

pub fn addon_action(node: u64) -> Option<(usize, u64)> {
    let offset = node.checked_sub(NODE_SETTINGS_ADDON_ACTION_BASE)?;
    (offset < 4 * 200).then(|| ((offset / 4) as usize, offset % 4))
}

pub fn addon_transport_url(addon: &serde_json::Value) -> Option<&str> {
    addon
        .get("transportUrl")
        .or_else(|| addon.pointer("/manifest/transportUrl"))
        .and_then(serde_json::Value::as_str)
}

fn manifest(addon: &serde_json::Value) -> &serde_json::Value {
    addon
        .get("manifest")
        .filter(|m| m.is_object())
        .unwrap_or(addon)
}

fn text<'a>(addon: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    manifest(addon)
        .get(key)
        .or_else(|| addon.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn host(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or_default()
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn addon_chips(addon: &serde_json::Value, language: &str) -> Vec<String> {
    let manifest = manifest(addon);
    let list = |key: &str| {
        manifest
            .get(key)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|value| {
                value
                    .as_str()
                    .or_else(|| value.get("name").and_then(serde_json::Value::as_str))
            })
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    };
    let mut out: Vec<String> = Vec::new();
    for (prefix, values) in [("resource", list("resources")), ("type", list("types"))] {
        for value in values {
            let label = localized_or(
                &format!("settings.addon_{prefix}.{value}"),
                &capitalize(&value),
                language,
            );
            if !out.contains(&label) {
                out.push(label);
            }
        }
    }
    out
}

struct Card<'a> {
    title: &'a str,
    version: Option<&'a str>,
    subtitle: &'a str,
    description: Option<&'a str>,
    logo: Option<&'a str>,
    icon: Option<&'static str>,
    chips: Vec<String>,
    leading: Vec<(u64, &'static str, bool)>,
    trailing: Vec<(u64, &'static str, bool)>,
}

impl Card<'_> {
    fn height(&self) -> f32 {
        let description = if self.description.is_some() {
            44.0
        } else {
            0.0
        };
        let chips = if self.chips.is_empty() {
            0.0
        } else {
            CHIP_HEIGHT + 12.0
        };
        CARD_PADDING * 2.0 + LOGO + 12.0 + description + chips + ACTION
    }
}

fn addon_card<'a>(
    addon: &'a serde_json::Value,
    index: usize,
    count: usize,
    language: &str,
) -> Card<'a> {
    let url = addon_transport_url(addon);
    Card {
        title: text(addon, "name").unwrap_or("Add-on"),
        version: text(addon, "version"),
        subtitle: url.map(host).unwrap_or_default(),
        description: text(addon, "description"),
        logo: text(addon, "logo"),
        icon: None,
        chips: addon_chips(addon, language),
        leading: vec![
            (addon_action_node(index, 0), "ChevronUp", index > 0),
            (
                addon_action_node(index, 1),
                "ChevronDown",
                index + 1 < count,
            ),
        ],
        trailing: vec![
            (addon_action_node(index, 2), "Refresh", url.is_some()),
            (addon_action_node(index, 3), "Delete", url.is_some()),
        ],
    }
}

fn repository_card<'a>(repo: &'a serde_json::Value, index: usize, language: &str) -> Card<'a> {
    let url = text(repo, "manifestUrl").unwrap_or_default();
    let count = repo
        .get("scraperCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    Card {
        title: text(repo, "name").unwrap_or("Plugin repository"),
        version: text(repo, "version"),
        subtitle: host(url),
        description: text(repo, "description"),
        logo: text(repo, "logo").or_else(|| text(repo, "iconUrl")),
        icon: Some("Plugins"),
        chips: vec![
            localized("settings.plugin_scraper_count", language).replace("%s", &count.to_string()),
        ],
        leading: Vec::new(),
        trailing: vec![
            (
                NODE_SETTINGS_PLUGIN_REFRESH_BASE + index as u64,
                "Refresh",
                true,
            ),
            (
                NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + index as u64,
                "Delete",
                true,
            ),
        ],
    }
}

fn repositories(settings: &SettingsModel) -> &[serde_json::Value] {
    settings
        .plugins
        .get("repositories")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], |items| &items[..items.len().min(20)])
}

fn scrapers(settings: &SettingsModel) -> &[serde_json::Value] {
    settings
        .plugins
        .get("scrapers")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], |items| &items[..items.len().min(30)])
}

fn plugin_error(settings: &SettingsModel) -> Option<&str> {
    settings
        .plugins
        .get("error")
        .and_then(|v| v.get("message"))
        .and_then(serde_json::Value::as_str)
}

fn header_height(metrics: UiMetrics) -> f32 {
    metrics.settings_extended_input_height
        + metrics.settings_extended_action_gap
        + metrics.settings_extended_action_height
        + 24.0
}

pub(super) fn addons_height(settings: &SettingsModel, metrics: UiMetrics, language: &str) -> f32 {
    let count = settings.addons.len();
    let cards: f32 = settings
        .addons
        .iter()
        .enumerate()
        .map(|(index, addon)| addon_card(addon, index, count, language).height() + CARD_GAP)
        .sum();
    header_height(metrics)
        + APPEARANCE_GROUP_HEADING_HEIGHT
        + if settings.addon_error.is_some() {
            32.0
        } else {
            0.0
        }
        + if count == 0 { 56.0 } else { 0.0 }
        + cards
}

pub(super) fn plugins_height(settings: &SettingsModel, metrics: UiMetrics, language: &str) -> f32 {
    let repos = repositories(settings);
    let cards: f32 = repos
        .iter()
        .enumerate()
        .map(|(index, repo)| repository_card(repo, index, language).height() + CARD_GAP)
        .sum();
    let scrapers = scrapers(settings).len();
    header_height(metrics)
        + APPEARANCE_GROUP_HEADING_HEIGHT
        + if plugin_error(settings).is_some() {
            32.0
        } else {
            0.0
        }
        + if repos.is_empty() { 56.0 } else { 0.0 }
        + cards
        + if scrapers > 0 {
            12.0 + APPEARANCE_GROUP_HEADING_HEIGHT + scrapers as f32 * SCRAPER_ROW
        } else {
            0.0
        }
}

#[allow(clippy::too_many_arguments)]
fn draw_header(
    context: &egui::Context,
    rect: Rect,
    top: f32,
    input: (u64, &str, String),
    buttons: &[(u64, String, components::ButtonKind)],
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) -> f32 {
    let mut y = top;
    settings_panel_input(
        context,
        layout,
        input.0,
        Rect::from_min_size(
            Pos2::new(rect.left(), y),
            Vec2::new(rect.width(), metrics.settings_extended_input_height),
        ),
        input.1,
        input.2,
        metrics,
    );
    y += metrics.settings_extended_input_height + metrics.settings_extended_action_gap;
    let width = metrics
        .settings_extended_addon_action_width
        .min((rect.width() - metrics.settings_extended_action_gap) / 2.0);
    for (index, (node, label, kind)) in buttons.iter().enumerate() {
        let button = Rect::from_min_size(
            Pos2::new(
                rect.left() + index as f32 * (width + metrics.settings_extended_action_gap),
                y,
            ),
            Vec2::new(width, metrics.settings_extended_action_height),
        );
        layout.focusable.push((*node, button));
        egui::Area::new(Id::new(("fluxa-addon-top", *node)))
            .constrain(false)
            .fixed_pos(button.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                if components::button(ui, label, width, button.height(), *kind, metrics).clicked() {
                    layout.activated = Some(*node);
                }
            });
    }
    y + metrics.settings_extended_action_height + 24.0
}

fn heading(painter: &egui::Painter, rect: Rect, y: f32, label: String) -> f32 {
    painter.text(
        Pos2::new(rect.left() + 4.0, y),
        Align2::LEFT_TOP,
        label,
        crate::fonts::regular(15.0),
        Color32::from_white_alpha(120),
    );
    y + APPEARANCE_GROUP_HEADING_HEIGHT
}

fn notice(painter: &egui::Painter, rect: Rect, y: f32, text: &str, color: Color32) {
    let font = crate::fonts::regular(15.0);
    painter.text(
        Pos2::new(rect.left() + 4.0, y),
        Align2::LEFT_TOP,
        truncate_to_width(painter, text, &font, rect.width()),
        font,
        color,
    );
}

const ERROR: Color32 = Color32::from_rgb(255, 125, 110);

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_addons(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let mut y = draw_header(
        context,
        rect,
        top,
        (
            NODE_SETTINGS_ADDON_URL,
            &settings.addon_url,
            localized("settings.addon_url", language),
        ),
        &[
            (
                NODE_SETTINGS_ADDON_INSTALL,
                localized("settings.addon_install", language),
                components::ButtonKind::Primary,
            ),
            (
                NODE_SETTINGS_ADDON_REFRESH,
                localized("settings.addon_refresh", language),
                components::ButtonKind::Secondary,
            ),
        ],
        metrics,
        layout,
    );
    let count = settings.addons.len();
    y = heading(
        &painter,
        rect,
        y,
        format!(
            "{}  ·  {count}",
            localized("settings.addon_installed", language)
        ),
    );
    if let Some(error) = settings.addon_error.as_deref() {
        notice(&painter, rect, y, error, ERROR);
        y += 32.0;
    }
    if count == 0 {
        notice(
            &painter,
            rect,
            y + 8.0,
            &localized("settings.addon_empty", language),
            Color32::from_white_alpha(140),
        );
    }
    for (index, addon) in settings.addons.iter().enumerate() {
        let card = addon_card(addon, index, count, language);
        y = draw_card(context, &painter, &card, assets, rect, y, layout) + CARD_GAP;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_plugins(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let mut y = draw_header(
        context,
        rect,
        top,
        (
            NODE_SETTINGS_PLUGIN_URL,
            &settings.plugin_url,
            localized("settings.plugin_url", language),
        ),
        &[(
            NODE_SETTINGS_PLUGIN_INSTALL,
            localized("settings.plugin_add", language),
            components::ButtonKind::Primary,
        )],
        metrics,
        layout,
    );
    let repos = repositories(settings);
    y = heading(
        &painter,
        rect,
        y,
        format!(
            "{}  ·  {}",
            localized("settings.plugin_repositories", language),
            repos.len()
        ),
    );
    if let Some(error) = plugin_error(settings) {
        notice(&painter, rect, y, error, ERROR);
        y += 32.0;
    }
    if repos.is_empty() {
        notice(
            &painter,
            rect,
            y + 8.0,
            &localized("settings.plugin_empty", language),
            Color32::from_white_alpha(140),
        );
        y += 56.0;
    }
    for (index, repo) in repos.iter().enumerate() {
        let card = repository_card(repo, index, language);
        y = draw_card(context, &painter, &card, assets, rect, y, layout) + CARD_GAP;
    }
    let scrapers = scrapers(settings);
    if scrapers.is_empty() {
        return;
    }
    y = heading(
        &painter,
        rect,
        y + 12.0,
        format!(
            "{}  ·  {}",
            localized("settings.plugin_scrapers", language),
            scrapers.len()
        ),
    );
    let group = Rect::from_min_size(
        Pos2::new(rect.left(), y),
        Vec2::new(rect.width(), scrapers.len() as f32 * SCRAPER_ROW),
    );
    painter.rect_filled(group, 16.0, Color32::from_rgb(19, 19, 19));
    painter.rect_stroke(
        group,
        16.0,
        egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
        egui::StrokeKind::Inside,
    );
    for (index, scraper) in scrapers.iter().enumerate() {
        let row = Rect::from_min_size(
            Pos2::new(group.left(), y + index as f32 * SCRAPER_ROW),
            Vec2::new(group.width(), SCRAPER_ROW),
        );
        if index > 0 {
            painter.line_segment(
                [
                    row.left_top() + Vec2::new(16.0, 0.0),
                    row.right_top() - Vec2::new(16.0, 0.0),
                ],
                egui::Stroke::new(1.0, Color32::from_white_alpha(14)),
            );
        }
        let node = NODE_SETTINGS_PLUGIN_SCRAPER_BASE + index as u64;
        layout.focusable.push((node, row));
        let enabled = scraper
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let name = text(scraper, "name")
            .or_else(|| text(scraper, "id"))
            .unwrap_or("Scraper");
        let detail =
            text(scraper, "description").or_else(|| text(scraper, "repositoryUrl").map(host));
        let track = Rect::from_center_size(
            row.right_center() - Vec2::new(metrics.settings_toggle_width * 0.5 + 16.0, 0.0),
            Vec2::new(
                metrics.settings_toggle_width,
                metrics.settings_toggle_height,
            ),
        );
        let text_width = track.left() - row.left() - 32.0;
        let title_font = crate::fonts::regular(17.0);
        let title_y = if detail.is_some() {
            row.center().y - 11.0
        } else {
            row.center().y
        };
        painter.text(
            Pos2::new(row.left() + 16.0, title_y),
            Align2::LEFT_CENTER,
            truncate_to_width(&painter, name, &title_font, text_width),
            title_font,
            Color32::WHITE,
        );
        if let Some(detail) = detail {
            let font = crate::fonts::regular(14.0);
            painter.text(
                Pos2::new(row.left() + 16.0, row.center().y + 12.0),
                Align2::LEFT_CENTER,
                truncate_to_width(&painter, detail, &font, text_width),
                font,
                Color32::from_white_alpha(130),
            );
        }
        components::toggle(
            context,
            &painter,
            Id::new(("fluxa-scraper-toggle", node)),
            track,
            enabled,
            metrics,
        );
        egui::Area::new(Id::new(("fluxa-scraper-row", node)))
            .constrain(false)
            .fixed_pos(row.min)
            .order(egui::Order::Middle)
            .show(context, |ui| {
                if ui.allocate_rect(row, Sense::click()).clicked() {
                    layout.activated = Some(node);
                }
            });
    }
}

fn draw_card(
    context: &egui::Context,
    painter: &egui::Painter,
    card: &Card,
    assets: &mut impl HomeAssets,
    rect: Rect,
    top: f32,
    layout: &mut HomeLayout,
) -> f32 {
    let frame = Rect::from_min_size(
        Pos2::new(rect.left(), top),
        Vec2::new(rect.width(), card.height()),
    );
    painter.rect_filled(frame, 16.0, Color32::from_rgb(19, 19, 19));
    painter.rect_stroke(
        frame,
        16.0,
        egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
        egui::StrokeKind::Inside,
    );
    let inner = frame.shrink(CARD_PADDING);
    let logo = Rect::from_min_size(inner.min, Vec2::splat(LOGO));
    painter.rect_filled(logo, 12.0, Color32::from_white_alpha(14));
    let texture = assets.texture_for(card.logo, [96, 96], ArtworkPriority::Visible);
    if let Some(texture) = texture {
        let size = context
            .tex_manager()
            .read()
            .meta(texture)
            .map_or([1, 1], |meta| meta.size);
        let aspect = size[0] as f32 / size[1].max(1) as f32;
        let fit = if aspect > 1.0 {
            Vec2::new(LOGO, LOGO / aspect)
        } else {
            Vec2::new(LOGO * aspect, LOGO)
        };
        painter.image(
            texture,
            Rect::from_center_size(logo.center(), fit),
            full_uv(),
            Color32::WHITE,
        );
    } else if let Some(icon) = card.icon.and_then(|name| assets.icon(name)) {
        painter.image(
            icon,
            Rect::from_center_size(logo.center(), Vec2::splat(24.0)),
            full_uv(),
            Color32::from_white_alpha(200),
        );
    } else {
        painter.text(
            logo.center(),
            Align2::CENTER_CENTER,
            capitalize(&card.title.chars().take(1).collect::<String>()),
            crate::fonts::regular(20.0),
            Color32::from_white_alpha(200),
        );
    }
    let text_left = logo.right() + 14.0;
    let text_width = inner.right() - text_left;
    let title_font = crate::fonts::regular(18.0);
    let title = truncate_to_width(painter, card.title, &title_font, text_width * 0.75);
    let galley = painter.layout_no_wrap(title, title_font, Color32::WHITE);
    let title_pos = Pos2::new(text_left, logo.top() + 2.0);
    let title_width = galley.size().x;
    painter.galley(title_pos, galley, Color32::WHITE);
    if let Some(version) = card.version {
        painter.text(
            Pos2::new(title_pos.x + title_width + 8.0, title_pos.y + 4.0),
            Align2::LEFT_TOP,
            format!("v{}", version.trim_start_matches('v')),
            crate::fonts::regular(13.0),
            Color32::from_white_alpha(120),
        );
    }
    let subtitle_font = crate::fonts::regular(14.0);
    painter.text(
        Pos2::new(text_left, logo.top() + 28.0),
        Align2::LEFT_TOP,
        truncate_to_width(painter, card.subtitle, &subtitle_font, text_width),
        subtitle_font,
        Color32::from_white_alpha(130),
    );
    let mut y = logo.bottom() + 12.0;
    if let Some(description) = card.description {
        let mut job = egui::text::LayoutJob::simple(
            description.to_owned(),
            crate::fonts::regular(15.0),
            Color32::from_white_alpha(185),
            inner.width(),
        );
        job.wrap.max_rows = 2;
        job.wrap.overflow_character = Some('…');
        painter.galley(
            Pos2::new(inner.left(), y),
            painter.layout_job(job),
            Color32::WHITE,
        );
        y += 44.0;
    }
    if !card.chips.is_empty() {
        let font = crate::fonts::regular(13.0);
        let mut x = inner.left();
        for chip in &card.chips {
            let galley =
                painter.layout_no_wrap(chip.clone(), font.clone(), Color32::from_white_alpha(210));
            let width = galley.size().x + 20.0;
            if x + width > inner.right() {
                break;
            }
            let chip_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(width, CHIP_HEIGHT));
            painter.rect_stroke(
                chip_rect,
                8.0,
                egui::Stroke::new(1.0, Color32::from_white_alpha(40)),
                egui::StrokeKind::Inside,
            );
            painter.galley(
                chip_rect.center() - galley.size() * 0.5,
                galley,
                Color32::WHITE,
            );
            x += width + 8.0;
        }
        y += CHIP_HEIGHT + 12.0;
    }
    let step = ACTION + 4.0;
    let leading = card
        .leading
        .iter()
        .enumerate()
        .map(|(slot, action)| (inner.left() - 4.0 + slot as f32 * step, action));
    let trailing_count = card.trailing.len();
    let trailing = card.trailing.iter().enumerate().map(|(slot, action)| {
        (
            inner.right() + 4.0 - (trailing_count - slot) as f32 * step + 4.0,
            action,
        )
    });
    for (x, &(node, icon, enabled)) in leading.chain(trailing) {
        let button = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(ACTION));
        if enabled {
            layout.focusable.push((node, button));
        }
        let texture = assets.icon(icon);
        egui::Area::new(Id::new(("fluxa-addon-action", node)))
            .constrain(false)
            .fixed_pos(button.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                let response = components::icon_button(
                    ui,
                    texture,
                    ACTION,
                    Color32::from_white_alpha(220),
                    false,
                    false,
                    enabled,
                );
                if enabled && response.clicked() {
                    layout.activated = Some(node);
                }
            });
    }
    frame.bottom()
}

pub const SERVER_INPUTS: [u64; 3] = [
    NODE_SETTINGS_SERVER_ADDRESS,
    NODE_SETTINGS_SERVER_USERNAME,
    NODE_SETTINGS_SERVER_PASSWORD,
];

pub fn server_input(node: u64) -> Option<usize> {
    SERVER_INPUTS.iter().position(|input| *input == node)
}

pub fn server_index(node: u64) -> Option<usize> {
    let offset = node.checked_sub(NODE_SETTINGS_SERVER_BASE)?;
    (offset < 50).then_some(offset as usize)
}

fn server_card<'a>(server: &'a serde_json::Value, index: usize, language: &str) -> Card<'a> {
    let kind = text(server, "kind").unwrap_or_default();
    let libraries = server
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    Card {
        title: text(server, "name").unwrap_or("Server"),
        version: None,
        subtitle: text(server, "baseUrl").map(host).unwrap_or_default(),
        description: None,
        logo: None,
        icon: Some("Server"),
        chips: vec![
            capitalize(kind),
            localized("settings.server_libraries", language).replace("%s", &libraries.to_string()),
        ],
        leading: Vec::new(),
        trailing: vec![(NODE_SETTINGS_SERVER_BASE + index as u64, "Delete", true)],
    }
}

fn server_notice(settings: &SettingsModel, language: &str) -> Option<(String, Color32)> {
    let prompt = settings
        .account_auth
        .as_ref()
        .filter(|prompt| matches!(prompt.provider.as_str(), "plex" | "jellyfin" | "emby"))?;
    if prompt.failed {
        return Some((localized("settings.server_failed", language), ERROR));
    }
    Some((
        format!(
            "{} {}  ·  {}",
            localized("trakt.device.enter_code", language),
            prompt.code,
            prompt.url
        ),
        Color32::from_white_alpha(200),
    ))
}

pub(super) fn servers_height(settings: &SettingsModel, metrics: UiMetrics, language: &str) -> f32 {
    let cards: f32 = settings
        .media_servers
        .iter()
        .enumerate()
        .map(|(index, server)| server_card(server, index, language).height() + CARD_GAP)
        .sum();
    let input = metrics.settings_extended_input_height + metrics.settings_extended_action_gap;
    input * 3.0
        + metrics.settings_extended_action_height
        + 24.0
        + APPEARANCE_GROUP_HEADING_HEIGHT
        + 32.0
        + if settings.media_servers.is_empty() {
            56.0
        } else {
            0.0
        }
        + cards
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_servers(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let mut y = top;
    let hints = [
        "settings.server_address",
        "settings.server_username",
        "settings.server_password",
    ];
    for (index, node) in SERVER_INPUTS.iter().enumerate() {
        let field = Rect::from_min_size(
            Pos2::new(rect.left(), y),
            Vec2::new(rect.width(), metrics.settings_extended_input_height),
        );
        let mut value = settings.server_fields[index].clone();
        egui::Area::new(Id::new(("fluxa-server-input", *node)))
            .constrain(false)
            .fixed_pos(field.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                let response = components::text_input(
                    ui,
                    &mut value,
                    &localized(hints[index], language),
                    field.width(),
                    index == 2,
                );
                layout.focusable.push((*node, response.rect));
                if response.changed() {
                    layout.text_input = Some(value.clone());
                    layout.text_input_node = Some(*node);
                }
            });
        y += metrics.settings_extended_input_height + metrics.settings_extended_action_gap;
    }
    let gap = metrics.settings_extended_action_gap;
    let width = metrics
        .settings_extended_addon_action_width
        .min((rect.width() - gap * 2.0) / 3.0);
    let buttons = [
        (
            NODE_SETTINGS_SERVER_JELLYFIN,
            "Jellyfin",
            components::ButtonKind::Primary,
        ),
        (
            NODE_SETTINGS_SERVER_EMBY,
            "Emby",
            components::ButtonKind::Primary,
        ),
        (
            NODE_SETTINGS_SERVER_PLEX,
            "Plex",
            components::ButtonKind::Secondary,
        ),
    ];
    for (index, (node, label, kind)) in buttons.iter().enumerate() {
        let button = Rect::from_min_size(
            Pos2::new(rect.left() + index as f32 * (width + gap), y),
            Vec2::new(width, metrics.settings_extended_action_height),
        );
        layout.focusable.push((*node, button));
        egui::Area::new(Id::new(("fluxa-server-add", *node)))
            .constrain(false)
            .fixed_pos(button.min)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                if components::button(ui, label, width, button.height(), *kind, metrics).clicked() {
                    layout.activated = Some(*node);
                }
            });
    }
    y += metrics.settings_extended_action_height + 24.0;
    y = heading(
        &painter,
        rect,
        y,
        format!(
            "{}  ·  {}",
            localized("settings.server_connected", language),
            settings.media_servers.len()
        ),
    );
    if let Some((message, color)) = server_notice(settings, language) {
        notice(&painter, rect, y, &message, color);
    }
    y += 32.0;
    if settings.media_servers.is_empty() {
        notice(
            &painter,
            rect,
            y + 8.0,
            &localized("settings.server_empty", language),
            Color32::from_white_alpha(140),
        );
        y += 56.0;
    }
    for (index, server) in settings.media_servers.iter().enumerate() {
        let card = server_card(server, index, language);
        y = draw_card(context, &painter, &card, assets, rect, y, layout) + CARD_GAP;
    }
}
