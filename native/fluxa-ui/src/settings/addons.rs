use super::*;
use crate::draw_text;

pub const ADDON_NAME_LIMIT: usize = 60;

pub fn addon_action_node(index: usize, action: u64) -> u64 {
    NODE_SETTINGS_ADDON_ACTION_BASE + index as u64 * 6 + action
}

pub fn addon_action(node: u64) -> Option<(usize, u64)> {
    let offset = node.checked_sub(NODE_SETTINGS_ADDON_ACTION_BASE)?;
    (offset < 6 * 200).then(|| ((offset / 6) as usize, offset % 6))
}

pub fn kit_section(addon: &serde_json::Value) -> Option<usize> {
    let title = match manifest(addon)
        .get("id")
        .and_then(serde_json::Value::as_str)?
    {
        "fluxa.streamkit" => "Streamkit",
        "fluxa.metadatakit" => "Metadatakit",
        _ => return None,
    };
    SETTINGS_SECTIONS
        .iter()
        .position(|section| section.title == title)
}

pub fn addon_enabled(addon: &serde_json::Value) -> bool {
    addon.get("enabled").and_then(serde_json::Value::as_bool) != Some(false)
}

pub fn addon_title(addon: &serde_json::Value) -> Option<&str> {
    addon
        .get("displayName")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .or_else(|| text(addon, "name"))
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
    let catalogs = manifest
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let mut out: Vec<String> = Vec::new();
    for (prefix, values) in [("resource", list("resources")), ("type", list("types"))] {
        for value in values {
            if prefix == "resource" && value == "catalog" {
                if catalogs == 1 {
                    out.push(localized("settings.addon_catalog_one", language));
                } else if catalogs > 1 {
                    out.push(
                        localized("settings.addon_catalog_count", language)
                            .replace("%s", &catalogs.to_string()),
                    );
                }
                continue;
            }
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
    toggle: Option<(u64, bool)>,
    group: Option<(u64, bool, String)>,
    leading: Vec<(u64, &'static str, bool)>,
    trailing: Vec<(u64, &'static str, bool)>,
    builtin: Option<(u64, String)>,
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
    let kit = kit_section(addon);
    let kit_title = kit.map(|section| SETTINGS_SECTIONS[section].title);
    Card {
        title: if resolved {
            addon_title(addon).unwrap_or("Add-on")
        } else {
            url.and_then(|url| url.rsplit('/').find(|part| !part.is_empty()))
                .unwrap_or("Add-on")
        },
        warning: failure,
        version: text(addon, "version").filter(|_| resolved),
        subtitle: url.map(host).filter(|_| kit.is_none()).unwrap_or_default(),
        description: text(addon, "description").filter(|_| resolved),
        logo: text(addon, "logo").filter(|_| resolved),
        icon: kit_title,
        chips: if resolved {
            addon_chips(addon, language)
        } else {
            Vec::new()
        },
        toggle: Some((addon_action_node(index, 4), addon_enabled(addon)))
            .filter(|_| url.is_some() && kit.is_none()),
        group: None,
        leading: if kit.is_some() {
            Vec::new()
        } else {
            vec![
                (addon_action_node(index, 0), "ChevronUp", index > 0),
                (
                    addon_action_node(index, 1),
                    "ChevronDown",
                    index + 1 < count,
                ),
            ]
        },
        trailing: if kit.is_some() {
            Vec::new()
        } else {
            vec![
                (addon_action_node(index, 2), "Refresh", url.is_some()),
                (addon_action_node(index, 5), "Edit", url.is_some()),
                (addon_action_node(index, 3), "Delete", url.is_some()),
            ]
        },
        builtin: kit_title.map(|title| {
            (
                addon_action_node(index, 3),
                localized("settings.addon_builtin", language)
                    .replace("%s", &section_label(title, language)),
            )
        }),
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
        subtitle: url,
        description: text(repo, "description"),
        logo: text(repo, "logo").or_else(|| text(repo, "iconUrl")),
        icon: Some("Plugins"),
        chips: vec![
            localized("settings.plugin_scraper_count", language).replace("%s", &count.to_string()),
        ],
        toggle: None,
        group: Some((
            NODE_SETTINGS_PLUGIN_GROUP_BASE + index as u64,
            repo.get("groupResults")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            localized("settings.plugin_group_results", language),
        )),
        leading: Vec::new(),
        builtin: None,
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

fn streamkit_card<'a>(addon: &'a serde_json::Value, index: usize, language: &str) -> Card<'a> {
    let url = addon_transport_url(addon);
    Card {
        title: text(addon, "name").unwrap_or("Add-on"),
        warning: None,
        version: text(addon, "version"),
        subtitle: url.map(host).unwrap_or_default(),
        description: text(addon, "description"),
        logo: text(addon, "logo"),
        icon: Some("Plugins"),
        chips: addon_chips(addon, language),
        toggle: None,
        group: None,
        leading: Vec::new(),
        builtin: None,
        trailing: vec![(
            NODE_SETTINGS_STREAMKIT_ADDON_BASE + index as u64,
            "Delete",
            url.is_some(),
        )],
    }
}

pub fn plugin_config_input(node: u64) -> Option<usize> {
    let index = node.checked_sub(NODE_SETTINGS_PLUGIN_CONFIG_INPUT_BASE)? as usize;
    (index < PLUGIN_CONFIG_FIELDS).then_some(index)
}

pub fn plugin_config_fields(scraper: &serde_json::Value) -> &[serde_json::Value] {
    scraper
        .get("configFields")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], |fields| {
            &fields[..fields.len().min(PLUGIN_CONFIG_FIELDS)]
        })
}

fn open_config(settings: &SettingsModel) -> Option<&serde_json::Value> {
    let scraper = scrapers(settings).get(settings.plugin_config?)?;
    (!plugin_config_fields(scraper).is_empty()).then_some(scraper)
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

fn streamkit_note(settings: &SettingsModel, language: &str) -> Option<String> {
    if settings.streamkit_addon_rejected() {
        return Some(localized("settings.streamkit_addon_not_stream", language));
    }
    settings.addon_error.clone()
}

type FormInput<'a> = (u64, &'a str, String);
type FormButton = (u64, String, components::ButtonKind);
type FormIcon = (u64, Option<crate::gfx::TextureId>);

fn draw_form(
    context: &crate::gfx::Context,
    card: &mut crate::gfx::Ui,
    inputs: &[FormInput],
    buttons: &[FormButton],
    icon: Option<FormIcon>,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let pad = metrics.settings_addon_card_padding;
    let gap = metrics.settings_extended_action_gap;
    let width = card.max_rect().width();
    let input_rects: Vec<std::cell::Cell<Rect>> = inputs
        .iter()
        .map(|_| std::cell::Cell::new(Rect::NOTHING))
        .collect();
    let action_rect = std::cell::Cell::new(Rect::NOTHING);
    let mut children = vec![crate::kit::layout_items::gap(pad)];
    for found in &input_rects {
        children.push(crate::kit::item(
            Vec2::new(width, metrics.settings_extended_input_height),
            |_, rect| found.set(rect),
        ));
        children.push(crate::kit::layout_items::gap(gap));
    }
    children.push(crate::kit::item(
        Vec2::new(width, metrics.settings_extended_action_height),
        |_, rect| action_rect.set(rect),
    ));
    children.push(crate::kit::layout_items::gap(pad));
    super::page::tight(card, |card| crate::kit::lay(card, children));
    let inset = crate::flex::Insets::xy(pad, 0.0);
    for ((node, value, hint), found) in inputs.iter().zip(&input_rects) {
        settings_panel_input(
            context,
            layout,
            *node,
            super::page::padded(found.get(), inset),
            value,
            hint,
            metrics,
        );
    }
    let row = super::page::padded(action_rect.get(), inset);
    let side = metrics.settings_extended_action_height;
    let icon_space = if icon.is_some() { side + gap } else { 0.0 };
    let count = buttons.len().max(1) as f32;
    let split = (row.width() - icon_space - gap * (count - 1.0)) / count;
    let width = if metrics.compact {
        split
    } else {
        metrics.settings_extended_addon_action_width.min(split)
    };
    for (index, (node, label, kind)) in buttons.iter().enumerate() {
        let button = super::page::at(
            row,
            index as f32 * (width + gap),
            0.0,
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
    if let Some((node, texture)) = icon {
        let button = super::page::at(
            row,
            buttons.len() as f32 * (width + gap),
            0.0,
            Vec2::splat(side),
        );
        layout.focusable.push((node, button));
        components::placed_front(
            context,
            Id::new(("fluxa-settings-form", node)),
            button.min,
            |ui| {
                let color = metrics.text_primary;
                if components::icon_button(ui, texture, side, color, true, false, true).clicked() {
                    layout.activated = Some(node);
                }
            },
        );
    }
}

fn draw_header(
    context: &crate::gfx::Context,
    flow: &mut crate::gfx::Ui,
    title: &str,
    input: FormInput,
    buttons: &[FormButton],
    icon: Option<FormIcon>,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    super::page::padded_body(flow, 0.0, metrics.settings_group_gap, |flow| {
        account_group(flow, title, metrics, |card| {
            draw_form(context, card, &[input], buttons, icon, metrics, layout);
        });
    });
}

fn heading(
    flow: &mut crate::gfx::Ui,
    painter: &crate::gfx::Painter,
    rect: Rect,
    label: String,
    lead: f32,
    metrics: UiMetrics,
) {
    let block = super::page::slot(
        flow,
        Vec2::new(rect.width(), metrics.settings_group_heading_height),
        crate::flex::Insets {
            top: lead,
            ..crate::flex::Insets::default()
        },
    );
    group_heading(painter, rect, block.top(), &label, metrics);
}

fn note(
    painter: &crate::gfx::Painter,
    rect: Rect,
    y: f32,
    text: &str,
    color: Color32,
    rows: usize,
    metrics: UiMetrics,
) {
    let inset = metrics.settings_text_inset;
    let mut job = crate::gfx::text::LayoutJob::simple(
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
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    flow_area(context, "fluxa-addons-page", rect, top, 0.0, |flow| {
        draw_header(
            context,
            flow,
            &localized("settings.addon_url", language),
            (
                NODE_SETTINGS_ADDON_URL,
                &settings.addon_url,
                "https://".to_owned(),
            ),
            &[(
                NODE_SETTINGS_ADDON_INSTALL,
                localized("settings.addon_install", language),
                components::ButtonKind::Primary,
            )],
            Some((NODE_SETTINGS_ADDON_REFRESH, assets.icon("Refresh"))),
            metrics,
            layout,
        );
        let count = settings.addons.len();
        heading(
            flow,
            &painter,
            rect,
            format!(
                "{}  ·  {count}",
                localized("settings.addon_installed", language)
            ),
            0.0,
            metrics,
        );
        if let Some(error) = settings.addon_error.as_deref() {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_note_height))
                .1;
            note(
                &painter,
                rect,
                block.top(),
                error,
                danger_color(),
                2,
                metrics,
            );
        }
        if count == 0 {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_empty_height))
                .1;
            note(
                &painter,
                rect,
                block.top() + Space::S4.px(&metrics),
                &localized("settings.addon_empty", language),
                crate::stylesheet::color("glyph"),
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
            super::page::padded_body(flow, 0.0, metrics.settings_card_gap, |flow| {
                draw_card(context, flow, &card, assets, layout, metrics);
            });
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_streamkit_addons(
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    flow_area(context, "fluxa-streamkit-page", rect, top, 0.0, |flow| {
        draw_header(
            context,
            flow,
            &localized("settings.streamkit_addon_url", language),
            (
                NODE_SETTINGS_STREAMKIT_ADDON_URL,
                &settings.streamkit_addon_url,
                "https://".to_owned(),
            ),
            &[(
                NODE_SETTINGS_STREAMKIT_ADDON_ADD,
                localized("settings.streamkit_addon_add", language),
                components::ButtonKind::Primary,
            )],
            None,
            metrics,
            layout,
        );
        let addons = settings.streamkit_addons();
        heading(
            flow,
            &painter,
            rect,
            format!(
                "{}  ·  {}",
                localized("settings.streamkit_addons", language),
                addons.len()
            ),
            0.0,
            metrics,
        );
        if let Some(error) = streamkit_note(settings, language) {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_note_height))
                .1;
            note(
                &painter,
                rect,
                block.top(),
                &error,
                danger_color(),
                2,
                metrics,
            );
        }
        if addons.is_empty() {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_empty_height))
                .1;
            note(
                &painter,
                rect,
                block.top() + Space::S4.px(&metrics),
                &localized("settings.streamkit_addons_empty", language),
                crate::stylesheet::color("glyph"),
                3,
                metrics,
            );
        }
        for (index, addon) in addons.iter().enumerate() {
            let card = streamkit_card(addon, index, language);
            super::page::padded_body(flow, 0.0, metrics.settings_card_gap, |flow| {
                draw_card(context, flow, &card, assets, layout, metrics);
            });
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_plugins(
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    flow_area(context, "fluxa-plugins-page", rect, top, 0.0, |flow| {
        draw_header(
            context,
            flow,
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
            None,
            metrics,
            layout,
        );
        let risk = flow
            .allocate_space(Vec2::new(rect.width(), metrics.settings_plugin_risk_height))
            .1;
        note(
            &painter,
            rect,
            risk.top() - metrics.settings_group_gap + 10.0,
            &localized("settings.plugin_risk", language),
            crate::stylesheet::color("glyph"),
            4,
            metrics,
        );
        let repos = repositories(settings);
        heading(
            flow,
            &painter,
            rect,
            format!(
                "{}  ·  {}",
                localized("settings.plugin_repositories", language),
                repos.len()
            ),
            0.0,
            metrics,
        );
        if let Some(error) = plugin_error(settings) {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_note_height))
                .1;
            note(
                &painter,
                rect,
                block.top(),
                error,
                danger_color(),
                2,
                metrics,
            );
        }
        if repos.is_empty() {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_empty_height))
                .1;
            note(
                &painter,
                rect,
                block.top() + Space::S4.px(&metrics),
                &localized("settings.plugin_empty", language),
                crate::stylesheet::color("glyph"),
                3,
                metrics,
            );
        }
        for (index, repo) in repos.iter().enumerate() {
            let card = repository_card(repo, index, language);
            super::page::padded_body(flow, 0.0, metrics.settings_card_gap, |flow| {
                draw_card(context, flow, &card, assets, layout, metrics);
            });
        }
        let scrapers = scrapers(settings);
        if scrapers.is_empty() {
            return;
        }
        heading(
            flow,
            &painter,
            rect,
            format!(
                "{}  ·  {}",
                localized("settings.plugin_scrapers", language),
                scrapers.len()
            ),
            Space::S12.px(&metrics),
            metrics,
        );
        let inset = Space::S16.px(&metrics);
        crate::kit::view().class("card").show(flow, |list| {
            let painter = list.painter().clone();
            let width = list.max_rect().width();
            for (index, scraper) in scrapers.iter().enumerate() {
                let row = list
                    .allocate_space(Vec2::new(width, metrics.settings_scraper_row))
                    .1;
                if index > 0 {
                    painter.line_segment(
                        [
                            row.left_top() + Vec2::new(inset, 0.0),
                            row.right_top() - Vec2::new(inset, 0.0),
                        ],
                        crate::gfx::Stroke::new(
                            stroke::hairline(),
                            crate::stylesheet::color("outline"),
                        ),
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
                let detail = text(scraper, "description")
                    .or_else(|| text(scraper, "repositoryUrl").map(host));
                let track = super::page::aligned(
                    row,
                    Align2::RIGHT_CENTER,
                    Vec2::new(
                        metrics.settings_toggle_width,
                        metrics.settings_toggle_height,
                    ),
                    inset,
                );
                let configurable = !plugin_config_fields(scraper).is_empty();
                let gear_side = metrics.settings_extended_action_height;
                let gear = super::page::aligned(
                    Rect::from_min_max(row.min, Pos2::new(track.left(), row.bottom())),
                    Align2::RIGHT_CENTER,
                    Vec2::splat(gear_side),
                    inset,
                );
                let text_width = if configurable {
                    gear.left() - row.left() - Space::S32.px(&metrics)
                } else {
                    track.left() - row.left() - Space::S32.px(&metrics)
                };
                let title_font = crate::fonts::regular(metrics.text.subtitle);
                let body = super::page::padded(
                    row,
                    crate::flex::Insets {
                        left: inset,
                        right: (row.width() - inset - text_width).max(0.0),
                        ..crate::flex::Insets::default()
                    },
                );
                let (title_slot, detail_slot) = if detail.is_some() {
                    super::page::stacked_lines(
                        body,
                        Space::S12.px(&metrics),
                        Space::S12.px(&metrics),
                    )
                } else {
                    (body, body)
                };
                crate::flex::paint_text(
                    &painter,
                    title_slot,
                    name,
                    &title_font,
                    text_primary(),
                    Align2::LEFT_CENTER,
                );
                if let Some(detail) = detail {
                    crate::flex::paint_text(
                        &painter,
                        detail_slot,
                        detail,
                        &crate::fonts::regular(metrics.text.body),
                        metrics.text_muted,
                        Align2::LEFT_CENTER,
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
                if configurable {
                    let config_node = NODE_SETTINGS_PLUGIN_CONFIG_BASE + index as u64;
                    layout.focusable.push((config_node, gear));
                    let texture = assets.icon("Settings");
                    components::placed_front(
                        context,
                        Id::new(("fluxa-scraper-config", config_node)),
                        gear.min,
                        |ui| {
                            let color = metrics.text_primary;
                            if components::icon_button(
                                ui, texture, gear_side, color, true, false, true,
                            )
                            .clicked()
                            {
                                layout.activated = Some(config_node);
                            }
                        },
                    );
                }
            }
        });
        let Some(scraper) = open_config(settings) else {
            return;
        };
        let fields = plugin_config_fields(scraper);
        let title = format!(
            "{}  ·  {}",
            localized("settings.plugin_preferences", language),
            text(scraper, "name").unwrap_or("Scraper")
        );
        let inputs: Vec<FormInput> = fields
            .iter()
            .enumerate()
            .map(|(slot, field)| {
                let label = field
                    .get("label")
                    .or_else(|| field.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                (
                    NODE_SETTINGS_PLUGIN_CONFIG_INPUT_BASE + slot as u64,
                    settings.plugin_config_fields[slot].as_str(),
                    label.to_owned(),
                )
            })
            .collect();
        super::page::padded_body(flow, Space::S12.px(&metrics), 0.0, |flow| {
            account_group(flow, &title, metrics, |card| {
                draw_form(
                    context,
                    card,
                    &inputs,
                    &[(
                        NODE_SETTINGS_PLUGIN_CONFIG_SAVE,
                        localized("settings.plugin_preferences_save", language),
                        components::ButtonKind::Primary,
                    )],
                    None,
                    metrics,
                    layout,
                );
            });
        });
    });
}

fn draw_card(
    context: &crate::gfx::Context,
    flow: &mut crate::gfx::Ui,
    card: &Card,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
    metrics: UiMetrics,
) {
    let s4 = Space::S4.px(&metrics);
    let s8 = Space::S8.px(&metrics);
    let s12 = Space::S12.px(&metrics);
    let s14 = Space::S14.px(&metrics);
    let card_view = crate::kit::view()
        .class("card")
        .padding(Space::S16)
        .gap(Space::S12);
    card_view.show(flow, |body| {
        let painter = body.painter().clone();
        let width = body.max_rect().width();
        let head = body
            .allocate_space(Vec2::new(width, metrics.settings_logo))
            .1;
        let toggle_size = Vec2::new(
            metrics.settings_toggle_width,
            metrics.settings_toggle_height,
        );
        let head_row = crate::flex::row([
            crate::flex::slot(metrics.settings_logo)
                .id(0)
                .height(metrics.settings_logo),
            crate::flex::fill().id(1).margin(crate::flex::Insets {
                left: s14,
                right: if card.toggle.is_some() { s12 } else { 0.0 },
                ..crate::flex::Insets::default()
            }),
            crate::flex::leaf(toggle_size)
                .id(2)
                .width(if card.toggle.is_some() {
                    toggle_size.x
                } else {
                    0.0
                }),
        ])
        .align(crate::flex::Align::Start)
        .solve(head);
        let logo = head_row.find(0).unwrap_or(head);
        painter.rect_filled(logo, radius::control(), crate::stylesheet::color("well"));
        let texture = assets.texture_for(card.logo, [96, 96], ArtworkPriority::Visible);
        if texture.is_none()
            && crate::components::failed_dom_image(
                &painter,
                assets,
                logo,
                radius::control(),
                card.logo,
                [96, 96],
                crate::ImageFit::Contain,
                [0.5, 0.5],
            )
        {
        } else if let Some(texture) = texture {
            let size = context
                .tex_manager()
                .read()
                .meta(texture)
                .map_or([1, 1], |meta| meta.size);
            let aspect = size[0] as f32 / size[1].max(1) as f32;
            let fit = if aspect > 1.0 {
                Vec2::new(metrics.settings_logo, metrics.settings_logo / aspect)
            } else {
                Vec2::new(metrics.settings_logo * aspect, metrics.settings_logo)
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
                crate::stylesheet::color("text.body"),
            );
        }
        let text_slot = head_row.find(1).unwrap_or(head);
        let text_left = text_slot.left();
        let text_width = text_slot.width();
        let title_font = crate::fonts::regular(metrics.text.subtitle);
        let title = truncate_to_width(&painter, card.title, &title_font, text_width * 0.75);
        let title_galley = painter.layout_no_wrap(title, title_font, Color32::WHITE);
        let title_size = title_galley.size();
        let title_pos = Pos2::new(text_left, head.top());
        painter.galley(title_pos, title_galley, text_primary());
        if let Some(version) = card.version {
            draw_text(
                &painter,
                Pos2::new(title_pos.x + title_size.x + s8, title_pos.y + s4),
                Align2::LEFT_TOP,
                format!("v{}", version.trim_start_matches('v')),
                crate::fonts::regular(metrics.text.meta),
                metrics.text_muted,
                None,
            );
        }
        if let Some((node, enabled)) = card.toggle {
            let track = head_row.find(2).unwrap_or(head);
            let hit = track.expand(Space::S6.px(&metrics));
            layout.focusable.push((node, hit));
            components::toggle(
                context,
                &painter,
                Id::new(("fluxa-addon-toggle", node)),
                track,
                enabled,
                metrics,
            );
            components::placed_front(
                context,
                Id::new(("fluxa-addon-toggle-hit", node)),
                hit.min,
                |ui| {
                    if ui.allocate_rect(hit, Sense::click()).clicked() {
                        layout.activated = Some(node);
                    }
                },
            );
        }
        draw_text(
            &painter,
            Pos2::new(text_left, title_pos.y + title_size.y + s4),
            Align2::LEFT_TOP,
            card.subtitle,
            crate::fonts::regular(metrics.text.body),
            metrics.text_muted,
            Some(text_width),
        );
        if let Some(warning) = card.warning.as_deref() {
            let galley = painter.layout(
                warning.to_owned(),
                crate::fonts::regular(metrics.text.body),
                danger_color(),
                width,
            );
            let slot = body.allocate_space(Vec2::new(width, galley.size().y)).1;
            painter.galley(slot.min, galley, danger_color());
        }
        if let Some(description) = card.description {
            let mut job = crate::gfx::text::LayoutJob::simple(
                description.to_owned(),
                crate::fonts::regular(metrics.text.body),
                metrics.text_secondary,
                width,
            );
            job.wrap.max_rows = 2;
            job.wrap.overflow_character = Some('…');
            let galley = painter.layout_job(job);
            let slot = body.allocate_space(Vec2::new(width, galley.size().y)).1;
            painter.galley(slot.min, galley, text_primary());
        }
        if !card.chips.is_empty() {
            let row = body
                .allocate_space(Vec2::new(width, metrics.settings_chip_height))
                .1;
            let font = crate::fonts::regular(metrics.text.meta);
            let mut x = row.left();
            for chip in &card.chips {
                let galley = painter.layout_no_wrap(
                    chip.clone(),
                    font.clone(),
                    crate::stylesheet::color("text.body"),
                );
                let chip_width = galley.size().x + Space::S20.px(&metrics);
                if x + chip_width > row.right() {
                    break;
                }
                let chip_rect = Rect::from_min_size(
                    Pos2::new(x, row.top()),
                    Vec2::new(chip_width, metrics.settings_chip_height),
                );
                painter.rect_stroke(
                    chip_rect,
                    radius::control(),
                    crate::gfx::Stroke::new(stroke::hairline(), crate::stylesheet::color("raised")),
                    crate::gfx::StrokeKind::Inside,
                );
                painter.galley(
                    chip_rect.center() - galley.size() * 0.5,
                    galley,
                    text_primary(),
                );
                x += chip_width + s8;
            }
        }
        let row = body
            .allocate_space(Vec2::new(width, metrics.settings_action))
            .1;
        if let Some((node, enabled, label)) = &card.group {
            let track = Rect::from_min_size(
                Pos2::new(
                    row.left(),
                    row.top() + (metrics.settings_action - metrics.settings_toggle_height) * 0.5,
                ),
                Vec2::new(
                    metrics.settings_toggle_width,
                    metrics.settings_toggle_height,
                ),
            );
            let hit = Rect::from_min_size(row.min, Vec2::new(width * 0.6, metrics.settings_action));
            let node = *node;
            layout.focusable.push((node, hit));
            components::toggle(
                context,
                &painter,
                Id::new(("fluxa-plugin-group-toggle", node)),
                track,
                *enabled,
                metrics,
            );
            draw_text(
                &painter,
                Pos2::new(
                    track.right() + s12,
                    row.top() + metrics.settings_action * 0.5,
                ),
                Align2::LEFT_CENTER,
                label,
                crate::fonts::regular(metrics.text.body),
                metrics.text_secondary,
                Some(hit.right() - track.right() - s12),
            );
            components::placed_front(
                context,
                Id::new(("fluxa-plugin-group-hit", node)),
                hit.min,
                |ui| {
                    if ui.allocate_rect(hit, Sense::click()).clicked() {
                        layout.activated = Some(node);
                    }
                },
            );
        }
        if let Some((node, label)) = &card.builtin {
            layout.focusable.push((*node, row));
            components::placed_front(
                context,
                Id::new(("fluxa-addon-builtin", *node)),
                row.min,
                |ui| {
                    let response = ui.allocate_rect(row, Sense::click());
                    if response.is_pointer_button_down_on() {
                        ui.painter().rect_filled(
                            row,
                            radius::control(),
                            crate::stylesheet::color("well"),
                        );
                    }
                    if response.clicked() {
                        layout.activated = Some(*node);
                    }
                },
            );
            if let Some(chevron) = assets.icon("ChevronRight") {
                painter.image(
                    chevron,
                    Rect::from_center_size(
                        Pos2::new(row.right() - s14, row.center().y),
                        Vec2::splat(18.0),
                    ),
                    full_uv(),
                    crate::stylesheet::color("text.body"),
                );
            }
            draw_text(
                &painter,
                Pos2::new(row.left() + s4, row.center().y),
                Align2::LEFT_CENTER,
                label,
                crate::fonts::regular(metrics.text.body),
                metrics.text_secondary,
                Some(row.width() - metrics.settings_action),
            );
        }
        let step = metrics.settings_action + s4;
        let leading = card
            .leading
            .iter()
            .enumerate()
            .map(|(slot, action)| (row.left() - s4 + slot as f32 * step, action));
        let trailing_count = card.trailing.len();
        let trailing = card.trailing.iter().enumerate().map(|(slot, action)| {
            (
                row.right() + s4 - (trailing_count - slot) as f32 * step + s4,
                action,
            )
        });
        for (x, &(node, icon, enabled)) in leading.chain(trailing) {
            let button = Rect::from_min_size(
                Pos2::new(x, row.top()),
                Vec2::splat(metrics.settings_action),
            );
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
                        metrics.settings_action,
                        crate::stylesheet::color("text.body"),
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
    });
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
        toggle: None,
        group: None,
        leading: Vec::new(),
        builtin: None,
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
        crate::stylesheet::color("text.body"),
    ))
}

const SERVER_KINDS: [(u64, &str); 3] = [
    (NODE_SETTINGS_SERVER_JELLYFIN, "Jellyfin"),
    (NODE_SETTINGS_SERVER_EMBY, "Emby"),
    (NODE_SETTINGS_SERVER_PLEX, "Plex"),
];
const PLEX_KIND: usize = 2;

pub(super) fn draw_servers(
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    flow_area(context, "fluxa-servers-page", rect, top, 0.0, |flow| {
        let inset = metrics.settings_text_inset;
        let labels: Vec<String> = SERVER_KINDS
            .iter()
            .map(|(_, label)| (*label).to_owned())
            .collect();
        let tabs_block = flow
            .allocate_space(Vec2::new(rect.width(), metrics.settings_server_tabs_height))
            .1;
        components::placed_front(
            context,
            Id::new("fluxa-server-kinds"),
            Pos2::new(rect.left() + inset, tabs_block.top()),
            |ui| {
                let tabs = components::text_tabs(
                    ui,
                    &labels,
                    settings.server_kind,
                    metrics.settings_server_tabs_height - 12.0,
                    metrics,
                );
                for ((node, _), response) in SERVER_KINDS.iter().zip(tabs) {
                    components::focus_action(layout, *node, &response);
                }
            },
        );

        crate::kit::view().class("card").show(flow, |card| {
            if settings.server_kind == PLEX_KIND {
                let card_painter = card.painter().clone();
                let hint = card
                    .allocate_space(Vec2::new(
                        card.max_rect().width(),
                        metrics.settings_plex_hint_height,
                    ))
                    .1;
                note(
                    &card_painter,
                    hint.shrink2(Vec2::new(metrics.settings_addon_card_padding - inset, 0.0)),
                    hint.top() + metrics.settings_addon_card_padding,
                    &localized("settings.server_plex_hint", language),
                    metrics.text_secondary,
                    2,
                    metrics,
                );
                draw_form(
                    context,
                    card,
                    &[],
                    &[(
                        NODE_SETTINGS_SERVER_CONNECT,
                        localized("auth.sign_in", language),
                        components::ButtonKind::Primary,
                    )],
                    None,
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
                    None,
                    metrics,
                    layout,
                );
            }
        });

        let notice = server_notice(settings, language);
        let lead = metrics.settings_group_gap;
        if let Some((message, color)) = &notice {
            let block = super::page::slot(
                flow,
                Vec2::new(rect.width(), metrics.settings_note_height),
                crate::flex::Insets {
                    top: lead,
                    ..crate::flex::Insets::default()
                },
            );
            note(
                &painter,
                rect,
                block.top() - 8.0,
                message,
                *color,
                2,
                metrics,
            );
        }
        heading(
            flow,
            &painter,
            rect,
            format!(
                "{}  ·  {}",
                localized("settings.server_connected", language),
                settings.media_servers.len()
            ),
            if notice.is_some() { 0.0 } else { lead },
            metrics,
        );
        if settings.media_servers.is_empty() {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_empty_height))
                .1;
            note(
                &painter,
                rect,
                block.top() + Space::S4.px(&metrics),
                &localized("settings.server_empty", language),
                crate::stylesheet::color("glyph"),
                3,
                metrics,
            );
        }
        for (index, server) in settings.media_servers.iter().enumerate() {
            let card = server_card(server, index, language);
            super::page::padded_body(flow, 0.0, metrics.settings_card_gap, |flow| {
                draw_card(context, flow, &card, assets, layout, metrics);
            });
        }
    });
}

pub(super) fn draw_addon_edit_modal(
    context: &crate::gfx::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    layout: &mut HomeLayout,
) {
    let Some(addon) = settings
        .addon_edit
        .and_then(|index| settings.addons.get(index))
    else {
        return;
    };
    let language = settings
        .values
        .get("language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en");
    let metrics = UiMetrics::for_viewport(viewport);
    layout.focusable.clear();
    layout.activated = None;
    layout.text_input = None;
    layout.text_input_node = None;
    let mut name = settings.addon_edit_name.clone();
    let url = addon_transport_url(addon).unwrap_or_default();
    let original = text(addon, "name").unwrap_or_default();
    crate::kit::dialog_with(context, "fluxa-addon-edit", &["wide"], |ui| {
        let title = localized("settings.addon_edit", language);
        let url_label = localized("settings.addon_edit_url", language);
        let name_label = localized("settings.addon_edit_name", language);
        let gap = |space: Space| crate::kit::layout_items::gap(space.px(&metrics));
        crate::kit::lay(
            ui,
            vec![
                crate::kit::lazy(|ui| {
                    ui.label(
                        RichText::new(title.clone())
                            .size(metrics.text.subtitle)
                            .strong()
                            .color(text_primary()),
                    );
                }),
                gap(Space::S16),
                crate::kit::lazy(|ui| {
                    ui.label(
                        RichText::new(url_label.clone())
                            .size(metrics.text.meta)
                            .color(metrics.text_secondary),
                    );
                }),
                gap(Space::S4),
                crate::kit::lazy(|ui| {
                    ui.add(
                        crate::gfx::Label::new(
                            RichText::new(url.clone())
                                .size(metrics.text.body)
                                .color(metrics.text_muted),
                        )
                        .wrap(),
                    );
                }),
                gap(Space::S14),
                crate::kit::lazy(|ui| {
                    ui.label(
                        RichText::new(name_label.clone())
                            .size(metrics.text.meta)
                            .color(metrics.text_secondary),
                    );
                }),
                gap(Space::S4),
            ],
        );
        let field_width = ui.available_width();
        let field = components::text_input(
            ui,
            &mut name,
            original,
            field_width,
            components::InputKind::Text,
            metrics,
        );
        name = name.chars().take(ADDON_NAME_LIMIT).collect();
        layout.focusable.push((NODE_ADDON_EDIT_NAME, field.rect));
        if field.changed() {
            layout.text_input = Some(name.clone());
            layout.text_input_node = Some(NODE_ADDON_EDIT_NAME);
        }
        let hint = localized("settings.addon_edit_hint", language);
        let buttons_rect = crate::kit::lay(
            ui,
            vec![
                crate::kit::layout_items::gap(Space::S6.px(&metrics)),
                crate::kit::lazy(|ui| {
                    ui.label(
                        RichText::new(hint.clone())
                            .size(metrics.text.meta)
                            .color(metrics.text_muted),
                    );
                }),
                crate::kit::layout_items::gap(Space::S14.px(&metrics)),
                crate::kit::item(
                    crate::gfx::Vec2::new(field_width, crate::metrics::button_height()),
                    |_, _| {},
                ),
            ],
        );
        let buttons = crate::gfx::Rect::from_min_size(
            crate::gfx::Pos2::new(buttons_rect.left(), buttons_rect.bottom() - crate::metrics::button_height()),
            crate::gfx::Vec2::new(field_width, crate::metrics::button_height()),
        );
        let placed = crate::flex::row([
            crate::flex::fill().id(0).min_width(0.0),
            crate::flex::fill().id(1).min_width(0.0),
        ])
        .gap(Space::S8.px(&metrics))
        .solve(buttons);
        let cancel_rect = placed.find(0).unwrap_or(buttons);
        let save_rect = placed.find(1).unwrap_or(buttons);
        let cancel = ui
            .scope_builder(crate::gfx::UiBuilder::new().max_rect(cancel_rect), |ui| {
                components::button_with_text_size(
                    ui,
                    &localized("common.cancel", language),
                    cancel_rect.width(),
                    crate::metrics::button_height(),
                    components::ButtonKind::Secondary,
                    crate::style::size::BUTTON_TEXT,
                    metrics,
                )
            })
            .inner;
        components::focus_action(layout, NODE_ADDON_EDIT_CANCEL, &cancel);
        let save = ui
            .scope_builder(crate::gfx::UiBuilder::new().max_rect(save_rect), |ui| {
                components::primary_button(
                    ui,
                    &localized("settings.addon_edit_save", language),
                    save_rect.width(),
                    true,
                )
            })
            .inner;
        layout.focusable.push((NODE_ADDON_EDIT_SAVE, save.rect));
        if save.clicked() {
            layout.activated = Some(NODE_ADDON_EDIT_SAVE);
        }
        if ui.input(|input| input.key_pressed(crate::gfx::Key::Escape)) {
            layout.activated = Some(NODE_ADDON_EDIT_CANCEL);
        }
    });
}
