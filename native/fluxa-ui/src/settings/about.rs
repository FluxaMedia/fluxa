use super::page::padded;
use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

const NAME: usize = 1;
const TEXT: usize = 2;
const WORDMARK: usize = 3;
const CHIP: usize = 4;
const STATUS: usize = 5;
const CREATED: usize = 6;
const ACTION: usize = 7;
const BESIDE: usize = 8;
const LOGO: usize = 9;

enum Notice {
    Key(&'static str),
    Verbatim(&'static str),
}

pub(super) struct Credit {
    logo: Option<&'static str>,
    name: &'static str,
    notice: Notice,
}

const CREDITS: [Credit; 8] = [
    Credit {
        logo: None,
        name: "Fluxa",
        notice: Notice::Key("settings.about.credit.fluxa"),
    },
    Credit {
        logo: Some("tmdb"),
        name: "TMDB",
        notice: Notice::Verbatim(
            "This product uses the TMDB API but is not endorsed or certified by TMDB.",
        ),
    },
    Credit {
        logo: None,
        name: "mpv / FFmpeg / libass",
        notice: Notice::Key("settings.about.credit.playback"),
    },
    Credit {
        logo: None,
        name: "Phosphor Icons",
        notice: Notice::Key("settings.about.credit.phosphor"),
    },
    Credit {
        logo: None,
        name: "Archivo / Noto Sans",
        notice: Notice::Key("settings.about.credit.fonts"),
    },
    Credit {
        logo: None,
        name: "Anime4K",
        notice: Notice::Key("settings.about.credit.anime4k"),
    },
    Credit {
        logo: None,
        name: "epaint",
        notice: Notice::Key("settings.about.credit.epaint"),
    },
    Credit {
        logo: None,
        name: "librqbit-dualstack-sockets",
        notice: Notice::Key("settings.about.credit.librqbit"),
    },
];

fn notice(credit: &Credit, language: &str) -> String {
    match credit.notice {
        Notice::Key(key) => localized(key, language),
        Notice::Verbatim(text) => text.to_string(),
    }
}

fn credit_row_height(
    name: &crate::gfx::Galley,
    text: &crate::gfx::Galley,
    metrics: UiMetrics,
) -> f32 {
    metrics.settings_row_padding * 2.0 + name.size().y + metrics.settings_name_gap + text.size().y
}

pub(super) fn draw_credits(
    context: &crate::gfx::Context,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
) {
    let measure = context.layer_painter(crate::gfx::LayerId::background());
    let width = rect.width() - metrics.settings_row_inset * 2.0 - Space::S8.px(&metrics);
    let name_font = crate::fonts::medium(metrics.settings_label_size);
    let rows: Vec<_> = CREDITS
        .iter()
        .map(|credit| {
            let name = measure.layout_no_wrap(
                credit.name.to_owned(),
                name_font.clone(),
                metrics.text_primary,
            );
            let text = measure.layout_job(crate::gfx::text::LayoutJob::simple(
                notice(credit, language),
                crate::fonts::regular(metrics.text.meta),
                metrics.text_secondary,
                width,
            ));
            (credit, name, text)
        })
        .collect();
    flow_area(context, "fluxa-licenses-page", rect, top, metrics.settings_group_gap, |flow| {
        account_group(
            flow,
            &localized("settings.about.credits", language),
            metrics,
            |card| {
                let painter = card.painter().clone();
                let card_width = card.max_rect().width();
                let edge = metrics.settings_row_inset + Space::S4.px(&metrics);
                for (index, (credit, name, text)) in rows.into_iter().enumerate() {
                    let row = card
                        .allocate_space(Vec2::new(
                            card_width,
                            credit_row_height(&name, &text, metrics),
                        ))
                        .1;
                    if index > 0 {
                        let rule = padded(row, Insets::xy(edge, 0.0));
                        painter.line_segment(
                            [rule.left_top(), rule.right_top()],
                            crate::gfx::Stroke::new(
                                stroke::hairline(),
                                crate::stylesheet::color("outline"),
                            ),
                        );
                    }
                    let logo = credit.logo.and_then(|logo| assets.logo(logo));
                    let mut line = vec![flex::leaf(name.size()).id(NAME), flex::fill()];
                    if let Some((_, size)) = logo {
                        line.push(
                            flex::leaf(Vec2::new(
                                metrics.settings_logo_height * size.x,
                                metrics.settings_logo_height,
                            ))
                            .id(LOGO),
                        );
                    }
                    let lines = flex::column([
                        flex::row(line).align(Align::Center).height(name.size().y),
                        flex::leaf(text.size()).id(TEXT),
                    ])
                    .align(Align::Stretch)
                    .gap(metrics.settings_name_gap)
                    .padding(Insets {
                        left: edge,
                        right: edge,
                        top: metrics.settings_row_padding,
                        bottom: 0.0,
                    })
                    .solve(row);
                    let name_rect = lines.find(NAME).unwrap_or(row);
                    let text_rect = lines.find(TEXT).unwrap_or(row);
                    painter.galley(name_rect.min, name, metrics.text_primary);
                    if let (Some((texture, _)), Some(slot)) = (logo, lines.find(LOGO)) {
                        painter.image(texture, slot, full_uv(), Color32::WHITE);
                    }
                    painter.galley(text_rect.min, text, metrics.text_secondary);
                }
            },
        );
    });
}

pub fn settings_link_url(node: u64) -> Option<&'static str> {
    ABOUT_LINKS
        .iter()
        .find(|link| link.node == node)
        .map(|link| link.url)
        .filter(|url| !url.is_empty())
}

pub(super) struct AboutLink {
    node: u64,
    logo: &'static str,
    title: &'static str,
    subtitle: &'static str,
    url: &'static str,
}

pub(super) const ABOUT_LINKS: [AboutLink; 3] = [
    AboutLink {
        node: NODE_SETTINGS_LINK_GITHUB,
        logo: "github",
        title: "settings.about.github",
        subtitle: "settings.about.github_value",
        url: "https://github.com/FluxaMedia/fluxa",
    },
    AboutLink {
        node: NODE_SETTINGS_LINK_DISCORD,
        logo: "discord",
        title: "settings.about.discord",
        subtitle: "settings.about.discord_value",
        url: "https://discord.gg/wan9FeDEfe",
    },
    AboutLink {
        node: NODE_SETTINGS_REPORT_PROBLEM,
        logo: "report",
        title: "settings.about.report",
        subtitle: "settings.about.report_value",
        url: "",
    },
];

pub(super) fn about_update(update: &UpdateView, language: &str) -> (String, Option<&'static str>) {
    match &update.state {
        _ if !update.supported => (localized("settings.updates.unsupported", language), None),
        UpdateState::Idle => (
            localized("settings.updates.idle", language),
            Some("settings.updates.check"),
        ),
        UpdateState::Checking => (localized("settings.updates.checking", language), None),
        UpdateState::UpToDate => (
            localized("settings.updates.up_to_date", language),
            Some("settings.updates.check"),
        ),
        UpdateState::Available(version) => (
            format!(
                "{} {version}",
                localized("settings.updates.available", language)
            ),
            Some("settings.updates.install"),
        ),
        UpdateState::Downloading(percent) => (
            format!(
                "{} {percent}%",
                localized("settings.updates.downloading", language)
            ),
            None,
        ),
        UpdateState::Ready(version) => (
            format!(
                "{} {version}",
                localized("settings.updates.ready", language)
            ),
            Some("settings.updates.restart"),
        ),
        UpdateState::Failed(reason) => (
            format!(
                "{}: {reason}",
                localized("settings.updates.failed", language)
            ),
            Some("settings.updates.check"),
        ),
    }
}

pub(super) fn about_has_action(settings: &SettingsModel) -> bool {
    about_update(&settings.update, settings.language())
        .1
        .is_some()
}

pub(super) fn draw_about(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    assets: &impl HomeAssets,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    compact: bool,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let update = &settings.update;
    let top = page_top(rect, metrics);
    let (status, action) = about_update(update, language);
    let action_below = compact && action.is_some();
    let hero_height = metrics.settings_about_hero_height
        + if action_below {
            Space::S20.px(&metrics) + metrics.settings_about_action_height
        } else {
            0.0
        };
    flow_area(context, "fluxa-about-page", rect, top, metrics.settings_group_gap, |flow| {
        let hero = flow.allocate_space(Vec2::new(rect.width(), hero_height)).1;
        painter.rect_filled(hero, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            hero,
            metrics.card_radius,
            crate::gfx::Stroke::new(1.0, crate::stylesheet::color("divider")),
            crate::gfx::StrokeKind::Inside,
        );
        let wordmark_height = IconSize::Xl.px(&metrics);
        let logo = assets.logo("wordmark");
        let small = metrics.screen_card_subtitle_size + Dp::D2.px();
        let version_galley = painter.layout_no_wrap(
            update.current.clone(),
            crate::fonts::bold(small),
            metrics.text_primary,
        );
        let ok = matches!(update.state, UpdateState::UpToDate);
        let action_label = action.map(|key| localized(key, language));
        let beside_width = action_label.as_ref().filter(|_| !compact).map(|label| {
            label.chars().count() as f32 * small * metrics.settings_about_char_share
                + Space::S24.px(&metrics) * 2.0
        });
        let created_text = format!(
            "{} {}",
            localized("settings.about.created_by", language),
            localized("settings.about.owner_name", language)
        );
        let created_font = crate::fonts::regular(small - 1.0);
        let spacer = |space: Space| flex::leaf(Vec2::ZERO).height(space.px(&metrics));
        let mut lines = vec![
            flex::row([flex::leaf(Vec2::new(
                wordmark_height * logo.map_or(0.0, |(_, aspect)| aspect.x),
                wordmark_height,
            ))
            .id(WORDMARK)]),
            spacer(Space::S6),
            flex::row([
                flex::leaf(Vec2::new(
                    version_galley.size().x + Space::S20.px(&metrics),
                    metrics.settings_chip_height,
                ))
                .id(CHIP),
                flex::leaf(Vec2::ZERO)
                    .min_width(Dp::D80.px())
                    .grow(1.0)
                    .id(STATUS),
            ])
            .align(Align::Center)
            .gap(if ok {
                Space::S24.px(&metrics)
            } else {
                Space::S12.px(&metrics)
            })
            .height(metrics.settings_chip_height),
            spacer(Space::S10),
            flex::leaf(flex::text_size(painter, &created_text, &created_font)).id(CREATED),
        ];
        if action_label.is_some() && beside_width.is_none() {
            lines.push(spacer(Space::S10));
            lines.push(
                flex::column([flex::leaf(Vec2::ZERO)
                    .height(metrics.settings_about_action_height)
                    .id(ACTION)])
                .align(Align::Stretch)
                .padding(Insets {
                    right: metrics.settings_about_inset,
                    ..Insets::default()
                }),
            );
        }
        let mut columns = vec![
            flex::column(lines)
                .align(Align::Stretch)
                .padding(Insets {
                    left: metrics.settings_about_inset,
                    top: Space::S20.px(&metrics),
                    ..Insets::default()
                })
                .grow(1.0),
        ];
        if let Some(width) = beside_width {
            columns.push(
                flex::column([flex::leaf(Vec2::new(width, metrics.button_height)).id(BESIDE)])
                    .justify(flex::Justify::Center)
                    .padding(Insets {
                        right: metrics.settings_about_inset,
                        ..Insets::default()
                    }),
            );
        }
        let placed = flex::row(columns)
            .align(Align::Stretch)
            .gap(Space::S24.px(&metrics))
            .solve(hero);
        let chip = placed.find(CHIP).unwrap_or(hero);
        if let (Some((texture, _)), Some(slot)) = (logo, placed.find(WORDMARK)) {
            painter.image(
                texture,
                slot.translate(Vec2::new(-Space::S4.px(&metrics), 0.0)),
                full_uv(),
                Color32::WHITE,
            );
        }
        painter.rect_filled(chip, chip.height() * 0.5, Color32::from_white_alpha(14));
        painter.galley(
            flex::anchored(chip, Align2::CENTER_CENTER, version_galley.size(), 0.0).min,
            version_galley,
            metrics.text_primary,
        );
        if ok {
            painter.circle_filled(
                Pos2::new(chip.right() + Space::S14.px(&metrics), chip.center().y),
                metrics.settings_about_dot,
                Color32::from_rgb(52, 199, 89),
            );
        }
        if let Some(slot) = placed.find(STATUS) {
            flex::paint_text(
                painter,
                slot,
                &status,
                &crate::fonts::regular(small),
                metrics.text_secondary,
                Align2::LEFT_CENTER,
            );
        }
        if let Some(slot) = placed.find(CREATED) {
            draw_text(
                painter,
                slot.min,
                Align2::LEFT_TOP,
                created_text,
                created_font,
                ink(110),
                None,
            );
        }
        if let (Some(label), Some(button)) = (
            action_label,
            placed.find(BESIDE).or_else(|| placed.find(ACTION)),
        ) {
            pill_button_styled(
                context,
                layout,
                NODE_SETTINGS_UPDATE_ACTION,
                button,
                &label,
                small,
                true,
            );
        }

        super::page::tight(flow, |group| {
            let heading = group
                .allocate_space(Vec2::new(
                    rect.width(),
                    metrics.settings_group_heading_height,
                ))
                .1;
            group_heading(
                painter,
                rect,
                heading.top(),
                &localized("settings.about.project", language),
                metrics,
            );
            let arrow = assets.icon("ArrowUpRight");
            if compact {
                let list = group
                    .allocate_space(Vec2::new(
                        rect.width(),
                        metrics.settings_about_tile_height * ABOUT_LINKS.len() as f32,
                    ))
                    .1;
                painter.rect_filled(list, metrics.card_radius, metrics.surface_raised);
                painter.rect_stroke(
                    list,
                    metrics.card_radius,
                    crate::gfx::Stroke::new(1.0, crate::stylesheet::color("divider")),
                    crate::gfx::StrokeKind::Inside,
                );
                let tiles = flex::column((0..ABOUT_LINKS.len()).map(|index| {
                    flex::leaf(Vec2::ZERO)
                        .width(list.width())
                        .height(metrics.settings_about_tile_height)
                        .id(index)
                }))
                .solve(list);
                for (index, link) in ABOUT_LINKS.iter().enumerate() {
                    let tile = tiles.find(index).unwrap_or(list);
                    if index > 0 {
                        let rule = padded(tile, Insets::xy(Space::S16.px(&metrics), 0.0));
                        painter.hline(
                            rule.left()..=rule.right(),
                            tile.top(),
                            crate::gfx::Stroke::new(1.0, ink(24)),
                        );
                    }
                    about_tile(
                        context, assets, language, link, arrow, tile, false, metrics, layout,
                    );
                }
            } else {
                let tiles = group
                    .allocate_space(Vec2::new(rect.width(), metrics.settings_about_tile_height))
                    .1;
                let placed = flex::row((0..ABOUT_LINKS.len()).map(|index| flex::fill().id(index)))
                    .align(Align::Stretch)
                    .gap(metrics.settings_about_tile_gap)
                    .solve(tiles);
                for (index, link) in ABOUT_LINKS.iter().enumerate() {
                    about_tile(
                        context,
                        assets,
                        language,
                        link,
                        arrow,
                        placed.find(index).unwrap_or(tiles),
                        true,
                        metrics,
                        layout,
                    );
                }
            }
        });
    });
}

pub(super) fn about_tile(
    context: &crate::gfx::Context,
    assets: &impl HomeAssets,
    language: &str,
    link: &AboutLink,
    arrow: Option<crate::gfx::TextureId>,
    rect: Rect,
    framed: bool,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    components::placed_front(
        context,
        Id::new(("fluxa-settings-about-link", link.node)),
        rect.min,
        |ui| {
            let response = components::link_tile(
                ui,
                rect.size(),
                assets.logo(link.logo).map(|(texture, _)| texture),
                arrow,
                &localized(link.title, language),
                &localized(link.subtitle, language),
                framed,
                metrics,
            );
            components::focus_action(layout, link.node, &response);
        },
    );
}
