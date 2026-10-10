use super::*;
use crate::draw_text;

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
    warning: Option<String>,
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
        } + if self.warning.is_some() { 24.0 } else { 0.0 };
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
    unreachable: &[(String, String)],
) -> Card<'a> {
    let url = addon_transport_url(addon);
    let failure = url.and_then(|url| {
        unreachable
            .iter()
            .find(|(failing, _)| failing == host(url))
            .map(|(_, detail)| detail.clone())
    });
    let resolved = failure.is_none();
    Card {
        title: if resolved {
            text(addon, "name").unwrap_or("Add-on")
        } else {
            url.and_then(|url| url.rsplit('/').find(|part| !part.is_empty()))
                .unwrap_or("Add-on")
        },
        warning: failure,
        version: text(addon, "version").filter(|_| resolved),
        subtitle: url.map(host).unwrap_or_default(),
        description: text(addon, "description").filter(|_| resolved),
        logo: text(addon, "logo").filter(|_| resolved),
        icon: None,
        chips: if resolved {
            addon_chips(addon, language)
        } else {
            Vec::new()
        },
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
        warning: None,
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

const NOTE_HEIGHT: f32 = 48.0;
const EMPTY_HEIGHT: f32 = 64.0;
const PLUGIN_RISK_HEIGHT: f32 = 76.0;

fn form_height(inputs: usize, metrics: UiMetrics) -> f32 {
    inputs as f32 * (metrics.settings_extended_input_height + metrics.settings_extended_action_gap)
        + metrics.settings_extended_action_height
        + CARD_PADDING * 2.0
}

fn header_height(metrics: UiMetrics) -> f32 {
    APPEARANCE_GROUP_HEADING_HEIGHT + form_height(1, metrics) + APPEARANCE_GROUP_GAP
}

pub(super) fn addons_height(settings: &SettingsModel, metrics: UiMetrics, language: &str) -> f32 {
    let count = settings.addons.len();
    let cards: f32 = settings
        .addons
        .iter()
        .enumerate()
        .map(|(index, addon)| {
            addon_card(
                addon,
                index,
                count,
                language,
                &settings.unreachable_addon_hosts,
            )
            .height()
                + CARD_GAP
        })
        .sum();
    header_height(metrics)
        + APPEARANCE_GROUP_HEADING_HEIGHT
        + if settings.addon_error.is_some() {
            NOTE_HEIGHT
        } else {
            0.0
        }
        + if count == 0 { EMPTY_HEIGHT } else { 0.0 }
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
        + PLUGIN_RISK_HEIGHT
        + APPEARANCE_GROUP_HEADING_HEIGHT
        + if plugin_error(settings).is_some() {
            NOTE_HEIGHT
        } else {
            0.0
        }
        + if repos.is_empty() { EMPTY_HEIGHT } else { 0.0 }
        + cards
        + if scrapers > 0 {
            12.0 + APPEARANCE_GROUP_HEADING_HEIGHT + scrapers as f32 * SCRAPER_ROW
        } else {
            0.0
        }
}

type FormInput<'a> = (u64, &'a str, String);
type FormButton = (u64, String, components::ButtonKind);

fn draw_form(
    context: &egui::Context,
    card: Rect,
    inputs: &[FormInput],
    buttons: &[FormButton],
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let inner = card.shrink(CARD_PADDING);
    let gap = metrics.settings_extended_action_gap;
    let mut y = inner.top();
    for (node, value, hint) in inputs {
        settings_panel_input(
            context,
            layout,
            *node,
            Rect::from_min_size(
                Pos2::new(inner.left(), y),
                Vec2::new(inner.width(), metrics.settings_extended_input_height),
            ),
            value,
            hint,
            metrics,
        );
        y += metrics.settings_extended_input_height + gap;
    }
    let count = buttons.len().max(1) as f32;
    let split = (inner.width() - gap * (count - 1.0)) / count;
    let width = if metrics.compact {
        split
    } else {
        metrics.settings_extended_addon_action_width.min(split)
    };
    for (index, (node, label, kind)) in buttons.iter().enumerate() {
        let button = Rect::from_min_size(
            Pos2::new(inner.left() + index as f32 * (width + gap), y),
            Vec2::new(width, metrics.settings_extended_action_height),
        );
        layout.focusable.push((*node, button));
        components::placed_front(
            context,
            Id::new(("fluxa-settings-form", *node)),
            button.min,
            |ui| {
                if components::button(ui, label, width, button.height(), *kind, metrics).clicked() {
                    layout.activated = Some(*node);
                }
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_header(
    context: &egui::Context,
    rect: Rect,
    top: f32,
    title: &str,
    input: FormInput,
    buttons: &[FormButton],
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) -> f32 {
    let painter = context.layer_painter(egui::LayerId::background());
    let card = account_group(&painter, rect, top, form_height(1, metrics), title, metrics);
    draw_form(context, card, &[input], buttons, metrics, layout);
    card.bottom() + APPEARANCE_GROUP_GAP
}

fn heading(painter: &egui::Painter, rect: Rect, y: f32, label: String, metrics: UiMetrics) -> f32 {
    group_heading(painter, rect, y, &label, metrics);
    y + APPEARANCE_GROUP_HEADING_HEIGHT
}

fn note(
    painter: &egui::Painter,
    rect: Rect,
    y: f32,
    text: &str,
    color: Color32,
    rows: usize,
    metrics: UiMetrics,
) {
    let inset = if metrics.compact { 16.0 } else { 8.0 };
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        crate::fonts::regular(metrics.text.body),
        color,
        rect.width() - inset * 2.0,
    );
    job.wrap.max_rows = rows;
    painter.galley(
        Pos2::new(rect.left() + inset, y),
        painter.layout_job(job),
        color,
    );
}

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
        &localized("settings.addon_url", language),
        (
            NODE_SETTINGS_ADDON_URL,
            &settings.addon_url,
            "https://".to_owned(),
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
        metrics,
    );
    if let Some(error) = settings.addon_error.as_deref() {
        note(&painter, rect, y, error, danger_color(), 2, metrics);
        y += NOTE_HEIGHT;
    }
    if count == 0 {
        note(
            &painter,
            rect,
            y + 4.0,
            &localized("settings.addon_empty", language),
            ink(tone::MUTED),
            3,
            metrics,
        );
    }
    for (index, addon) in settings.addons.iter().enumerate() {
        let card = addon_card(
            addon,
            index,
            count,
            language,
            &settings.unreachable_addon_hosts,
        );
        y = draw_card(context, &painter, &card, assets, rect, y, layout, metrics) + CARD_GAP;
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
        &localized("settings.plugin_url", language),
        (
            NODE_SETTINGS_PLUGIN_URL,
            &settings.plugin_url,
            "https://".to_owned(),
        ),
        &[(
            NODE_SETTINGS_PLUGIN_INSTALL,
            localized("settings.plugin_add", language),
            components::ButtonKind::Primary,
        )],
        metrics,
        layout,
    );
    note(
        &painter,
        rect,
        y - APPEARANCE_GROUP_GAP + 10.0,
        &localized("settings.plugin_risk", language),
        ink(tone::MUTED),
        4,
        metrics,
    );
    y += PLUGIN_RISK_HEIGHT;
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
        metrics,
    );
    if let Some(error) = plugin_error(settings) {
        note(&painter, rect, y, error, danger_color(), 2, metrics);
        y += NOTE_HEIGHT;
    }
    if repos.is_empty() {
        note(
            &painter,
            rect,
            y + 4.0,
            &localized("settings.plugin_empty", language),
            ink(tone::MUTED),
            3,
            metrics,
        );
        y += EMPTY_HEIGHT;
    }
    for (index, repo) in repos.iter().enumerate() {
        let card = repository_card(repo, index, language);
        y = draw_card(context, &painter, &card, assets, rect, y, layout, metrics) + CARD_GAP;
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
        metrics,
    );
    let group = Rect::from_min_size(
        Pos2::new(rect.left(), y),
        Vec2::new(rect.width(), scrapers.len() as f32 * SCRAPER_ROW),
    );
    painter.rect_filled(group, metrics.card_radius, metrics.surface);
    painter.rect_stroke(
        group,
        metrics.card_radius,
        egui::Stroke::new(stroke::HAIRLINE, metrics.border),
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
                egui::Stroke::new(stroke::HAIRLINE, ink(tone::FAINT)),
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
        let title_font = crate::fonts::regular(metrics.text.subtitle);
        let title_y = if detail.is_some() {
            row.center().y - 11.0
        } else {
            row.center().y
        };
        draw_text(
            &painter,
            Pos2::new(row.left() + 16.0, title_y),
            Align2::LEFT_CENTER,
            name,
            title_font,
            text_primary(),
            Some(text_width),
        );
        if let Some(detail) = detail {
            let font = crate::fonts::regular(metrics.text.body);
            draw_text(
                &painter,
                Pos2::new(row.left() + 16.0, row.center().y + 12.0),
                Align2::LEFT_CENTER,
                detail,
                font,
                metrics.text_muted,
                Some(text_width),
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
        components::placed(
            context,
            Id::new(("fluxa-scraper-row", node)),
            row.min,
            |ui| {
                if ui.allocate_rect(row, Sense::click()).clicked() {
                    layout.activated = Some(node);
                }
            },
        );
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
    metrics: UiMetrics,
) -> f32 {
    let frame = Rect::from_min_size(
        Pos2::new(rect.left(), top),
        Vec2::new(rect.width(), card.height()),
    );
    painter.rect_filled(frame, metrics.card_radius, metrics.surface);
    painter.rect_stroke(
        frame,
        metrics.card_radius,
        egui::Stroke::new(stroke::HAIRLINE, metrics.border),
        egui::StrokeKind::Inside,
    );
    let inner = frame.shrink(CARD_PADDING);
    let logo = Rect::from_min_size(inner.min, Vec2::splat(LOGO));
    painter.rect_filled(logo, radius::control(), ink(tone::FAINT));
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
            ink(tone::BODY),
        );
    } else {
        draw_text(
            painter,
            logo.center(),
            Align2::CENTER_CENTER,
            capitalize(&card.title.chars().take(1).collect::<String>()),
            crate::fonts::regular(metrics.text.title),
            metrics.text_primary,
            None,
        );
    }
    let text_left = logo.right() + 14.0;
    let text_width = inner.right() - text_left;
    let title_font = crate::fonts::regular(metrics.text.subtitle);
    let title = truncate_to_width(painter, card.title, &title_font, text_width * 0.75);
    let galley = painter.layout_no_wrap(title, title_font, Color32::WHITE);
    let title_pos = Pos2::new(text_left, logo.top() + 2.0);
    let title_width = galley.size().x;
    painter.galley(title_pos, galley, text_primary());
    if let Some(version) = card.version {
        draw_text(
            painter,
            Pos2::new(title_pos.x + title_width + 8.0, title_pos.y + 4.0),
            Align2::LEFT_TOP,
            format!("v{}", version.trim_start_matches('v')),
            crate::fonts::regular(metrics.text.meta),
            metrics.text_muted,
            None,
        );
    }
    let subtitle_font = crate::fonts::regular(metrics.text.body);
    draw_text(
        painter,
        Pos2::new(text_left, logo.top() + 28.0),
        Align2::LEFT_TOP,
        card.subtitle,
        subtitle_font,
        metrics.text_muted,
        Some(text_width),
    );
    let mut y = logo.bottom() + 12.0;
    if let Some(warning) = card.warning.as_deref() {
        draw_text(
            painter,
            Pos2::new(inner.left(), y),
            Align2::LEFT_TOP,
            warning,
            crate::fonts::regular(metrics.text.body),
            danger_color(),
            Some(inner.width()),
        );
        y += 24.0;
    }
    if let Some(description) = card.description {
        let mut job = egui::text::LayoutJob::simple(
            description.to_owned(),
            crate::fonts::regular(metrics.text.body),
            metrics.text_secondary,
            inner.width(),
        );
        job.wrap.max_rows = 2;
        job.wrap.overflow_character = Some('…');
        painter.galley(
            Pos2::new(inner.left(), y),
            painter.layout_job(job),
            text_primary(),
        );
        y += 44.0;
    }
    if !card.chips.is_empty() {
        let font = crate::fonts::regular(metrics.text.meta);
        let mut x = inner.left();
        for chip in &card.chips {
            let galley = painter.layout_no_wrap(chip.clone(), font.clone(), ink(tone::BODY));
            let width = galley.size().x + 20.0;
            if x + width > inner.right() {
                break;
            }
            let chip_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(width, CHIP_HEIGHT));
            painter.rect_stroke(
                chip_rect,
                radius::control(),
                egui::Stroke::new(stroke::HAIRLINE, ink(tone::RAISED)),
                egui::StrokeKind::Inside,
            );
            painter.galley(
                chip_rect.center() - galley.size() * 0.5,
                galley,
                text_primary(),
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
        components::placed_front(
            context,
            Id::new(("fluxa-addon-action", node)),
            button.min,
            |ui| {
                let response = components::icon_button(
                    ui,
                    texture,
                    ACTION,
                    ink(tone::BODY),
                    false,
                    false,
                    enabled,
                );
                if enabled && response.clicked() {
                    layout.activated = Some(node);
                }
            },
        );
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
        warning: None,
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
        return Some((
            localized("settings.server_failed", language),
            danger_color(),
        ));
    }
    Some((
        format!(
            "{} {}  ·  {}",
            localized("trakt.device.enter_code", language),
            prompt.code,
            prompt.url
        ),
        ink(tone::BODY),
    ))
}

const SERVER_KINDS: [(u64, &str); 3] = [
    (NODE_SETTINGS_SERVER_JELLYFIN, "Jellyfin"),
    (NODE_SETTINGS_SERVER_EMBY, "Emby"),
    (NODE_SETTINGS_SERVER_PLEX, "Plex"),
];
const PLEX_KIND: usize = 2;
const SERVER_TABS_HEIGHT: f32 = 44.0;
const PLEX_HINT_HEIGHT: f32 = 48.0;

fn server_form_height(settings: &SettingsModel, metrics: UiMetrics) -> f32 {
    if settings.server_kind == PLEX_KIND {
        PLEX_HINT_HEIGHT + form_height(0, metrics)
    } else {
        form_height(SERVER_INPUTS.len(), metrics)
    }
}

pub(super) fn servers_height(settings: &SettingsModel, metrics: UiMetrics, language: &str) -> f32 {
    let cards: f32 = settings
        .media_servers
        .iter()
        .enumerate()
        .map(|(index, server)| server_card(server, index, language).height() + CARD_GAP)
        .sum();
    SERVER_TABS_HEIGHT
        + server_form_height(settings, metrics)
        + APPEARANCE_GROUP_GAP
        + if server_notice(settings, language).is_some() {
            NOTE_HEIGHT
        } else {
            0.0
        }
        + APPEARANCE_GROUP_HEADING_HEIGHT
        + if settings.media_servers.is_empty() {
            EMPTY_HEIGHT
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
    let inset = if metrics.compact { 16.0 } else { 8.0 };
    let labels: Vec<String> = SERVER_KINDS
        .iter()
        .map(|(_, label)| (*label).to_owned())
        .collect();
    components::placed_front(
        context,
        Id::new("fluxa-server-kinds"),
        Pos2::new(rect.left() + inset, top),
        |ui| {
            let tabs = components::text_tabs(
                ui,
                &labels,
                settings.server_kind,
                SERVER_TABS_HEIGHT - 12.0,
                metrics,
            );
            for ((node, _), response) in SERVER_KINDS.iter().zip(tabs) {
                components::focus_action(layout, *node, &response);
            }
        },
    );

    let card = Rect::from_min_size(
        Pos2::new(rect.left(), top + SERVER_TABS_HEIGHT),
        Vec2::new(rect.width(), server_form_height(settings, metrics)),
    );
    painter.rect_filled(card, metrics.card_radius, metrics.surface);
    painter.rect_stroke(
        card,
        metrics.card_radius,
        egui::Stroke::new(stroke::HAIRLINE, metrics.border),
        egui::StrokeKind::Inside,
    );
    if settings.server_kind == PLEX_KIND {
        note(
            &painter,
            card.shrink2(Vec2::new(CARD_PADDING - inset, 0.0)),
            card.top() + CARD_PADDING,
            &localized("settings.server_plex_hint", language),
            metrics.text_secondary,
            2,
            metrics,
        );
        draw_form(
            context,
            card.with_min_y(card.top() + PLEX_HINT_HEIGHT),
            &[],
            &[(
                NODE_SETTINGS_SERVER_CONNECT,
                localized("auth.sign_in", language),
                components::ButtonKind::Primary,
            )],
            metrics,
            layout,
        );
    } else {
        let hints = [
            "settings.server_address",
            "settings.server_username",
            "settings.server_password",
        ];
        let inputs: Vec<FormInput> = SERVER_INPUTS
            .iter()
            .enumerate()
            .map(|(index, node)| {
                (
                    *node,
                    settings.server_fields[index].as_str(),
                    localized(hints[index], language),
                )
            })
            .collect();
        draw_form(
            context,
            card,
            &inputs,
            &[(
                NODE_SETTINGS_SERVER_CONNECT,
                localized("settings.account_connect", language),
                components::ButtonKind::Primary,
            )],
            metrics,
            layout,
        );
    }

    let mut y = card.bottom() + APPEARANCE_GROUP_GAP;
    if let Some((message, color)) = server_notice(settings, language) {
        note(&painter, rect, y - 8.0, &message, color, 2, metrics);
        y += NOTE_HEIGHT;
    }
    y = heading(
        &painter,
        rect,
        y,
        format!(
            "{}  ·  {}",
            localized("settings.server_connected", language),
            settings.media_servers.len()
        ),
        metrics,
    );
    if settings.media_servers.is_empty() {
        note(
            &painter,
            rect,
            y + 4.0,
            &localized("settings.server_empty", language),
            ink(tone::MUTED),
            3,
            metrics,
        );
    }
    for (index, server) in settings.media_servers.iter().enumerate() {
        let card = server_card(server, index, language);
        y = draw_card(context, &painter, &card, assets, rect, y, layout, metrics) + CARD_GAP;
    }
}
