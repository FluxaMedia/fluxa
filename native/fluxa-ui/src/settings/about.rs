use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

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

static MEASURED: AtomicU32 = AtomicU32::new(0);

pub(super) fn credits_card_height(width: f32, language: &str, metrics: UiMetrics) -> f32 {
    let measured = f32::from_bits(MEASURED.load(Ordering::Relaxed));
    if measured > 0.0 {
        return measured;
    }
    let size = metrics.text.meta;
    let per_line = ((width - metrics.settings_row_inset * 2.0) / (size * 0.5))
        .floor()
        .max(8.0);
    CREDITS
        .iter()
        .map(|credit| {
            let lines = (notice(credit, language).chars().count() as f32 / per_line).ceil();
            metrics.settings_row_padding * 2.0
                + metrics.settings_label_size * 1.3
                + metrics.settings_name_gap
                + lines * size * 1.35
        })
        .sum()
}

pub(super) fn page_flow<R>(
    context: &egui::Context,
    id: &str,
    rect: Rect,
    top: f32,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Area::new(Id::new(id))
        .constrain(false)
        .fixed_pos(Pos2::new(rect.left(), top))
        .interactable(false)
        .show(context, |ui| {
            let mut flow = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(
                Pos2::new(rect.left(), top),
                Vec2::new(rect.width(), f32::INFINITY),
            )));
            flow.spacing_mut().item_spacing = Vec2::ZERO;
            add(&mut flow)
        })
        .inner
}

fn credit_row_height(name: &egui::Galley, text: &egui::Galley, metrics: UiMetrics) -> f32 {
    metrics.settings_row_padding * 2.0 + name.size().y + metrics.settings_name_gap + text.size().y
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_credits(
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
) {
    let width = rect.width() - metrics.settings_row_inset * 2.0 - 8.0;
    let name_font = crate::fonts::medium(metrics.settings_label_size);
    let rows: Vec<_> = CREDITS
        .iter()
        .map(|credit| {
            let name = painter.layout_no_wrap(
                credit.name.to_owned(),
                name_font.clone(),
                metrics.text_primary,
            );
            let text = painter.layout_job(egui::text::LayoutJob::simple(
                notice(credit, language),
                crate::fonts::regular(metrics.text.meta),
                metrics.text_secondary,
                width,
            ));
            (credit, name, text)
        })
        .collect();
    let height = rows
        .iter()
        .map(|(_, name, text)| credit_row_height(name, text, metrics))
        .sum::<f32>();
    MEASURED.store(height.to_bits(), Ordering::Relaxed);
    page_flow(painter.ctx(), "fluxa-licenses-page", rect, top, |flow| {
        let block = flow
            .allocate_space(Vec2::new(
                rect.width(),
                metrics.settings_group_heading_height + height,
            ))
            .1;
        let card = account_group(
            painter,
            rect,
            block.top(),
            height,
            &localized("settings.about.credits", language),
            metrics,
        );
        let mut body = flow.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(
            card.left_top(),
            Vec2::new(card.width(), f32::INFINITY),
        )));
        body.spacing_mut().item_spacing = Vec2::ZERO;
        let left = card.left() + metrics.settings_row_inset + 4.0;
        let right = card.right() - metrics.settings_row_inset - 4.0;
        for (index, (credit, name, text)) in rows.into_iter().enumerate() {
            let row = body
                .allocate_space(Vec2::new(
                    card.width(),
                    credit_row_height(&name, &text, metrics),
                ))
                .1;
            if index > 0 {
                painter.line_segment(
                    [Pos2::new(left, row.top()), Pos2::new(right, row.top())],
                    egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("outline")),
                );
            }
            let name_rect = Rect::from_min_size(
                Pos2::new(left, row.top() + metrics.settings_row_padding),
                name.size(),
            );
            painter.galley(name_rect.min, name, metrics.text_primary);
            if let Some((texture, size)) = credit.logo.and_then(|logo| assets.logo(logo)) {
                let logo = Rect::from_min_size(
                    Pos2::new(
                        right - metrics.settings_logo_height * size.x,
                        name_rect.center().y - metrics.settings_logo_height * 0.5,
                    ),
                    Vec2::new(
                        metrics.settings_logo_height * size.x,
                        metrics.settings_logo_height,
                    ),
                );
                painter.image(
                    texture,
                    logo,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            let text_top = name_rect.bottom() + metrics.settings_name_gap;
            painter.galley(Pos2::new(left, text_top), text, metrics.text_secondary);
        }
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

pub(super) fn about_hero_height(settings: &SettingsModel, metrics: UiMetrics) -> f32 {
    let compact = metrics.compact;
    let action = if compact && about_has_action(settings) {
        20.0 + metrics.settings_about_action_height
    } else {
        0.0
    };
    let tiles = if compact {
        metrics.settings_about_tile_height * ABOUT_LINKS.len() as f32
    } else {
        metrics.settings_about_tile_height
    };
    metrics.settings_about_hero_height
        + action
        + metrics.settings_group_gap
        + metrics.settings_group_heading_height
        + tiles
        + metrics.settings_group_gap
}

pub(super) fn draw_about(
    context: &egui::Context,
    painter: &egui::Painter,
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
            20.0 + metrics.settings_about_action_height
        } else {
            0.0
        };
    page_flow(context, "fluxa-about-page", rect, top, |flow| {
        let hero = flow.allocate_space(Vec2::new(rect.width(), hero_height)).1;
        painter.rect_filled(hero, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            hero,
            metrics.card_radius,
            egui::Stroke::new(1.0, crate::stylesheet::color("divider")),
            egui::StrokeKind::Inside,
        );
        let left = hero.left() + metrics.settings_about_inset;
        if let Some((texture, aspect)) = assets.logo("wordmark") {
            let height = 36.0;
            painter.image(
                texture,
                Rect::from_min_size(
                    Pos2::new(left - 4.0, hero.top() + 24.0),
                    Vec2::new(height * aspect.x, height),
                ),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        let small = metrics.screen_card_subtitle_size + 2.0;
        let version = update.current.clone();
        let version_galley =
            painter.layout_no_wrap(version, crate::fonts::bold(small), metrics.text_primary);
        let chip = Rect::from_min_size(
            Pos2::new(left, hero.top() + 66.0),
            Vec2::new(version_galley.size().x + 20.0, 24.0),
        );
        painter.rect_filled(chip, 12.0, Color32::from_white_alpha(14));
        painter.galley(
            chip.center() - version_galley.size() * 0.5,
            version_galley,
            metrics.text_primary,
        );
        let ok = matches!(update.state, UpdateState::UpToDate);
        if ok {
            painter.circle_filled(
                Pos2::new(chip.right() + 14.0, chip.center().y),
                3.5,
                Color32::from_rgb(52, 199, 89),
            );
        }
        draw_text(
            painter,
            Pos2::new(chip.right() + if ok { 26.0 } else { 12.0 }, chip.center().y),
            Align2::LEFT_CENTER,
            status,
            crate::fonts::regular(small),
            metrics.text_secondary,
            Some((hero.right() - chip.right() - 200.0).max(80.0)),
        );
        draw_text(
            painter,
            Pos2::new(left, hero.top() + 106.0),
            Align2::LEFT_CENTER,
            format!(
                "{} {}",
                localized("settings.about.created_by", language),
                localized("settings.about.owner_name", language)
            ),
            crate::fonts::regular(small - 1.0),
            ink(110),
            None,
        );
        if let Some(key) = action {
            let label = localized(key, language);
            let size = metrics.screen_card_subtitle_size + 2.0;
            let button = if compact {
                Rect::from_min_size(
                    Pos2::new(left, hero.top() + metrics.settings_about_hero_height - 4.0),
                    Vec2::new(
                        hero.width() - (left - hero.left()) * 2.0,
                        metrics.settings_about_action_height,
                    ),
                )
            } else {
                let width = label.chars().count() as f32 * size * 0.58 + 44.0;
                Rect::from_center_size(
                    Pos2::new(hero.right() - 28.0 - width * 0.5, hero.center().y),
                    Vec2::new(width, 40.0),
                )
            };
            pill_button_styled(
                context,
                layout,
                NODE_SETTINGS_UPDATE_ACTION,
                button,
                &label,
                size,
                true,
            );
        }

        flow.add_space(metrics.settings_group_gap);
        let heading = flow
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
            let list = flow
                .allocate_space(Vec2::new(
                    rect.width(),
                    metrics.settings_about_tile_height * ABOUT_LINKS.len() as f32,
                ))
                .1;
            painter.rect_filled(list, metrics.card_radius, metrics.surface_raised);
            painter.rect_stroke(
                list,
                metrics.card_radius,
                egui::Stroke::new(1.0, crate::stylesheet::color("divider")),
                egui::StrokeKind::Inside,
            );
            for (index, link) in ABOUT_LINKS.iter().enumerate() {
                let y = list.top() + index as f32 * metrics.settings_about_tile_height;
                if index > 0 {
                    painter.hline(
                        list.left() + 18.0..=list.right() - 18.0,
                        y,
                        egui::Stroke::new(1.0, ink(24)),
                    );
                }
                about_tile(
                    context,
                    assets,
                    language,
                    link,
                    arrow,
                    Rect::from_min_size(
                        Pos2::new(list.left(), y),
                        Vec2::new(list.width(), metrics.settings_about_tile_height),
                    ),
                    false,
                    metrics,
                    layout,
                );
            }
        } else {
            let tiles = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_about_tile_height))
                .1;
            let count = ABOUT_LINKS.len() as f32;
            let width = (rect.width() - metrics.settings_about_tile_gap * (count - 1.0)) / count;
            for (index, link) in ABOUT_LINKS.iter().enumerate() {
                about_tile(
                    context,
                    assets,
                    language,
                    link,
                    arrow,
                    Rect::from_min_size(
                        Pos2::new(
                            rect.left() + index as f32 * (width + metrics.settings_about_tile_gap),
                            tiles.top(),
                        ),
                        Vec2::new(width, metrics.settings_about_tile_height),
                    ),
                    true,
                    metrics,
                    layout,
                );
            }
        }
    });
}

pub(super) fn about_tile(
    context: &egui::Context,
    assets: &impl HomeAssets,
    language: &str,
    link: &AboutLink,
    arrow: Option<egui::TextureId>,
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
