use super::*;

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

fn credit_row_height(name: &egui::Galley, text: &egui::Galley, metrics: UiMetrics) -> f32 {
    metrics.settings_row_padding * 2.0 + name.size().y + metrics.settings_name_gap + text.size().y
}

pub(super) fn draw_credits(
    context: &egui::Context,
    assets: &impl HomeAssets,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
) {
    let measure = context.layer_painter(egui::LayerId::background());
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
            let text = measure.layout_job(egui::text::LayoutJob::simple(
                notice(credit, language),
                crate::fonts::regular(metrics.text.meta),
                metrics.text_secondary,
                width,
            ));
            (credit, name, text)
        })
        .collect();
    flow_area(context, "fluxa-licenses-page", rect, top, |flow| {
        account_group(
            flow,
            &localized("settings.about.credits", language),
            metrics,
            |card| {
                let painter = card.painter().clone();
                let card_width = card.max_rect().width();
                let card_left = card.max_rect().left();
                let left = card_left + metrics.settings_row_inset + Space::S4.px(&metrics);
                let right =
                    card_left + card_width - metrics.settings_row_inset - Space::S4.px(&metrics);
                for (index, (credit, name, text)) in rows.into_iter().enumerate() {
                    let row = card
                        .allocate_space(Vec2::new(
                            card_width,
                            credit_row_height(&name, &text, metrics),
                        ))
                        .1;
                    if index > 0 {
                        painter.line_segment(
                            [Pos2::new(left, row.top()), Pos2::new(right, row.top())],
                            egui::Stroke::new(
                                stroke::hairline(),
                                crate::stylesheet::color("outline"),
                            ),
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
            Space::S20.px(&metrics) + metrics.settings_about_action_height
        } else {
            0.0
        };
    flow_area(context, "fluxa-about-page", rect, top, |flow| {
        let hero = flow.allocate_space(Vec2::new(rect.width(), hero_height)).1;
        painter.rect_filled(hero, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            hero,
            metrics.card_radius,
            egui::Stroke::new(1.0, crate::stylesheet::color("divider")),
            egui::StrokeKind::Inside,
        );
        let left = hero.left() + metrics.settings_about_inset;
        let mut y = hero.top() + Space::S20.px(&metrics);
        let wordmark_height = IconSize::Xl.px(&metrics);
        if let Some((texture, aspect)) = assets.logo("wordmark") {
            painter.image(
                texture,
                Rect::from_min_size(
                    Pos2::new(left - Space::S4.px(&metrics), y),
                    Vec2::new(wordmark_height * aspect.x, wordmark_height),
                ),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        y += wordmark_height + Space::S6.px(&metrics);
        let small = metrics.screen_card_subtitle_size + 2.0;
        let version = update.current.clone();
        let version_galley =
            painter.layout_no_wrap(version, crate::fonts::bold(small), metrics.text_primary);
        let chip = Rect::from_min_size(
            Pos2::new(left, y),
            Vec2::new(
                version_galley.size().x + Space::S20.px(&metrics),
                metrics.settings_chip_height,
            ),
        );
        painter.rect_filled(chip, chip.height() * 0.5, Color32::from_white_alpha(14));
        painter.galley(
            chip.center() - version_galley.size() * 0.5,
            version_galley,
            metrics.text_primary,
        );
        let ok = matches!(update.state, UpdateState::UpToDate);
        if ok {
            painter.circle_filled(
                Pos2::new(chip.right() + Space::S14.px(&metrics), chip.center().y),
                3.5,
                Color32::from_rgb(52, 199, 89),
            );
        }
        let action_label = action.map(|key| localized(key, language));
        let beside = action_label.as_ref().filter(|_| !compact).map(|label| {
            let width = label.chars().count() as f32 * small * 0.58 + Space::S24.px(&metrics) * 2.0;
            Rect::from_center_size(
                Pos2::new(
                    hero.right() - metrics.settings_about_inset - width * 0.5,
                    hero.center().y,
                ),
                Vec2::new(width, metrics.button_height),
            )
        });
        let status_right =
            beside.map_or(hero.right(), |button| button.left()) - Space::S24.px(&metrics);
        let status_left = chip.right()
            + if ok {
                Space::S24.px(&metrics)
            } else {
                Space::S12.px(&metrics)
            };
        draw_text(
            painter,
            Pos2::new(status_left, chip.center().y),
            Align2::LEFT_CENTER,
            status,
            crate::fonts::regular(small),
            metrics.text_secondary,
            Some((status_right - status_left).max(80.0)),
        );
        let created = draw_text(
            painter,
            Pos2::new(left, chip.bottom() + Space::S10.px(&metrics)),
            Align2::LEFT_TOP,
            format!(
                "{} {}",
                localized("settings.about.created_by", language),
                localized("settings.about.owner_name", language)
            ),
            crate::fonts::regular(small - 1.0),
            ink(110),
            None,
        );
        if let Some(label) = action_label {
            let button = beside.unwrap_or_else(|| {
                Rect::from_min_size(
                    Pos2::new(left, created.bottom() + Space::S10.px(&metrics)),
                    Vec2::new(
                        hero.width() - (left - hero.left()) * 2.0,
                        metrics.settings_about_action_height,
                    ),
                )
            });
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
                        list.left() + Space::S16.px(&metrics)
                            ..=list.right() - Space::S16.px(&metrics),
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
