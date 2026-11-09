use super::*;

#[derive(Clone, Copy)]
pub(super) struct Frame<'a> {
    pub context: &'a egui::Context,
    pub viewport: Viewport,
    pub settings: &'a SettingsModel,
    pub language: &'a str,
    pub metrics: UiMetrics,
    pub painter: &'a egui::Painter,
    pub scroll_y: f32,
    pub section_top: f32,
    pub active_section: usize,
    pub searching: bool,
}

pub(super) fn search_field(frame: Frame, layout: &mut HomeLayout, rect: Rect) {
    let Frame {
        context,
        settings,
        language,
        metrics,
        ..
    } = frame;
    components::placed_front(context, Id::new("fluxa-settings-search"), rect.min, |ui| {
        let mut query = settings.search.clone();
        let response = components::search_field(
            ui,
            &mut query,
            &localized("settings.search_placeholder", language),
            rect.width(),
            rect.height(),
            metrics,
        );
        layout.focusable.push((NODE_SETTINGS_SEARCH, response.rect));
        if response.changed() {
            layout.text_input = Some(query);
            layout.text_input_node = Some(NODE_SETTINGS_SEARCH);
        }
    });
}

pub(super) fn draw_compact_header(
    frame: Frame,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) -> bool {
    let Frame {
        context,
        viewport,
        settings,
        language,
        metrics,
        painter,
        scroll_y,
        section_top,
        active_section,
        searching,
    } = frame;
    let margin = metrics.settings_margin;
    let search_height = metrics.screen_control_height;
    let top = metrics.settings_top;
    let width = (viewport.width - margin * 2.0).max(1.0);
    let root = !settings.section_open || searching;
    if root {
        draw_text(
            &painter,
            Pos2::new(margin, top - scroll_y),
            Align2::LEFT_TOP,
            localized("nav.settings", language),
            crate::fonts::bold(metrics.screen_title_size),
            text_primary(),
            None,
        );
        search_field(
            frame,
            layout,
            Rect::from_min_size(
                Pos2::new(
                    margin,
                    top + metrics.screen_title_size + metrics.section_gap - scroll_y,
                ),
                Vec2::new(width, search_height),
            ),
        );
    } else {
        let back_rect = Rect::from_min_size(
            Pos2::new(margin - 8.0, top - scroll_y),
            Vec2::splat(metrics.icon_button_size),
        );
        let page_title = if settings.page_open {
            section_label(SETTINGS_SECTIONS[active_section].title, language)
        } else {
            section_label(SETTINGS_SECTIONS[active_section].category, language)
        };
        let sticky_on = scroll_y > 8.0;
        if !sticky_on {
            components::placed_front(
                context,
                Id::new("fluxa-settings-back"),
                back_rect.min,
                |ui| {
                    let response = components::back_button(ui, assets.icon("ArrowLeft"), metrics);
                    if response.clicked() {
                        layout.activated = Some(NODE_SETTINGS_BACK);
                    }
                },
            );
            layout.focusable.push((NODE_SETTINGS_BACK, back_rect));
        }
        if let Some(sticky) = components::sticky_bar(
            context,
            "fluxa-settings",
            viewport,
            margin - 8.0,
            sticky_on,
            components::StickyBar {
                back_node: NODE_SETTINGS_BACK,
                back_icon: assets.icon("ArrowLeft"),
                logo: None,
                title: &page_title,
                actions: Vec::new(),
            },
            metrics,
        ) {
            layout.focusable.extend(sticky.rects);
            layout.activated = sticky.clicked.or(layout.activated);
        }
        draw_text(
            &painter,
            back_rect.right_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            page_title,
            crate::fonts::bold(metrics.text.title + 2.0),
            text_primary(),
            None,
        );
    }
    if !settings.section_open && !searching {
        draw_compact_settings_list(
            context,
            viewport,
            &painter,
            assets,
            language,
            metrics,
            Rect::from_min_size(
                Pos2::new(margin, section_top - scroll_y),
                Vec2::new(width, 0.0),
            ),
            layout,
        );
        return true;
    }
    false
}

pub(super) fn draw_wide_nav(frame: Frame, assets: &mut impl HomeAssets, layout: &mut HomeLayout) {
    let Frame {
        context,
        viewport,
        language,
        settings: _,
        metrics,
        painter,
        scroll_y,
        section_top,
        active_section,
        searching,
    } = frame;
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let desktop = !compact && !tv;
    let margin = metrics.settings_margin;
    let search_height = metrics.screen_control_height;
    let nav_width = metrics
        .settings_nav_width
        .min((viewport.width - margin * 2.0).max(1.0) * 0.32);
    let nav_item_height = if tv {
        metrics.screen_control_height
    } else {
        metrics.settings_nav_item_height_desktop
    };
    let nav_item_gap = if tv {
        metrics.control_gap * 0.75
    } else {
        metrics.settings_nav_item_gap_desktop
    };
    let nav_scroll = if desktop { 0.0 } else { scroll_y };
    let nav_x = if desktop { 0.0 } else { margin };
    let nav_inset = if desktop { 16.0 } else { metrics.control_gap };
    let panel_width = if desktop {
        nav_width + margin
    } else {
        nav_width
    };
    let nav_panel = if desktop {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(panel_width, viewport.height))
    } else {
        Rect::from_min_size(
            Pos2::new(margin, section_top - nav_scroll),
            Vec2::new(
                nav_width,
                (viewport.height - section_top + nav_scroll - margin).max(nav_item_height),
            ),
        )
    };
    if desktop {
        painter.rect_filled(nav_panel, 0.0, surface_color());
        painter.line_segment(
            [nav_panel.right_top(), nav_panel.right_bottom()],
            egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("divider")),
        );
    } else {
        painter.rect_filled(nav_panel, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            nav_panel,
            metrics.card_radius,
            egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("divider")),
            egui::StrokeKind::Inside,
        );
    }
    if desktop {
        draw_text(
            &painter,
            Pos2::new(nav_x + nav_inset + 2.0, section_top - nav_scroll + 18.0),
            Align2::LEFT_TOP,
            localized("nav.settings", language),
            crate::fonts::bold(metrics.settings_section_title_size_desktop + 2.0),
            text_primary(),
            None,
        );
    }
    let search_top = if desktop { 58.0 } else { metrics.control_gap };
    search_field(
        frame,
        layout,
        Rect::from_min_size(
            Pos2::new(nav_x + nav_inset, section_top + search_top - nav_scroll),
            Vec2::new(panel_width - nav_inset * 2.0, search_height),
        ),
    );
    let nav_top_inset = search_top + search_height + metrics.control_gap * 1.5;
    let active_category = SETTINGS_SECTIONS[active_section].category;
    let nav_left = nav_x + nav_inset;
    let mut cursor = section_top + nav_top_inset - nav_scroll;
    if let Some(status) = crate::update::nav_status() {
        let rect = Rect::from_min_size(
            Pos2::new(nav_left, cursor),
            Vec2::new(panel_width - nav_inset * 2.0, nav_item_height),
        );
        cursor += nav_item_height + nav_item_gap;
        layout.focusable.push((NODE_UPDATE_OPEN, rect));
        let mut clicked = false;
        components::placed(context, Id::new("fluxa-settings-update"), rect.min, |ui| {
            let (item_rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
            if response.hovered() {
                ui.painter().rect_filled(
                    item_rect,
                    metrics.screen_control_radius,
                    crate::stylesheet::color("well"),
                );
            }
            let icon_left = item_rect.left() + metrics.settings_nav_row_inset;
            if let Some(icon) = assets.icon(status.icon) {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(
                        Pos2::new(
                            icon_left + metrics.settings_nav_icon * 0.5,
                            item_rect.center().y,
                        ),
                        Vec2::splat(metrics.settings_nav_icon),
                    ),
                    full_uv(),
                    accent(),
                );
            }
            draw_text(
                ui.painter(),
                Pos2::new(
                    icon_left + metrics.settings_nav_icon + 12.0,
                    item_rect.center().y,
                ),
                Align2::LEFT_CENTER,
                status.title.clone(),
                crate::fonts::medium(metrics.nav_label_size + metrics.settings_nav_label_boost),
                text_primary(),
                None,
            );
            draw_text(
                ui.painter(),
                Pos2::new(
                    item_rect.right() - metrics.settings_nav_row_inset,
                    item_rect.center().y,
                ),
                Align2::RIGHT_CENTER,
                status.detail.clone(),
                crate::fonts::medium(metrics.nav_label_size),
                crate::stylesheet::color("glyph"),
                None,
            );
            if let Some(fraction) = status.progress {
                let track = Rect::from_min_max(
                    Pos2::new(
                        item_rect.left() + metrics.settings_nav_row_inset,
                        item_rect.bottom() - 4.0,
                    ),
                    Pos2::new(
                        item_rect.right() - metrics.settings_nav_row_inset,
                        item_rect.bottom() - 1.5,
                    ),
                );
                ui.painter()
                    .rect_filled(track, 1.25, crate::stylesheet::color("track"));
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        track.min,
                        Vec2::new(track.width() * fraction, track.height()),
                    ),
                    1.25,
                    accent(),
                );
            }
            clicked = response.clicked();
        });
        if clicked {
            layout.activated = Some(NODE_UPDATE_OPEN);
        }
    }
    let mut last_group = "";
    for index in categories(viewport) {
        let section = &SETTINGS_SECTIONS[index];
        let node = NODE_SETTINGS_SECTION_BASE + index as u64;
        let group = category_group(section.category);
        if group != last_group {
            if !last_group.is_empty() {
                cursor += metrics.settings_nav_group_gap;
            }
            nav_heading(
                &painter,
                Pos2::new(
                    nav_left + metrics.settings_nav_row_inset,
                    cursor + metrics.settings_nav_group_height * 0.6,
                ),
                group,
                language,
                metrics,
            );
            cursor += metrics.settings_nav_group_height;
            last_group = group;
        }
        let rect = Rect::from_min_size(
            Pos2::new(nav_left, cursor),
            Vec2::new(panel_width - nav_inset * 2.0, nav_item_height),
        );
        cursor += nav_item_height + nav_item_gap;
        layout.focusable.push((node, rect));
        let mut clicked = false;
        components::placed(
            context,
            Id::new(("fluxa-settings-category", node)),
            rect.min,
            |ui| {
                let (item_rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                let active = section.category == active_category && !searching;
                if active || response.hovered() {
                    ui.painter().rect_filled(
                        item_rect,
                        metrics.screen_control_radius,
                        ink(if active { tone::soft() } else { tone::faint() }),
                    );
                }
                let tint = if active || response.hovered() {
                    text_primary()
                } else {
                    crate::stylesheet::color("text.subtle")
                };
                let icon_left = item_rect.left() + metrics.settings_nav_row_inset;
                settings_category_icon(
                    assets,
                    ui.painter(),
                    Pos2::new(
                        icon_left + metrics.settings_nav_icon * 0.5,
                        item_rect.center().y,
                    ),
                    section.category,
                    tint,
                    metrics.settings_nav_icon,
                );
                draw_text(
                    ui.painter(),
                    Pos2::new(
                        icon_left + metrics.settings_nav_icon + 12.0,
                        item_rect.center().y,
                    ),
                    Align2::LEFT_CENTER,
                    section_label(section.category, language),
                    crate::fonts::medium(metrics.nav_label_size + metrics.settings_nav_label_boost),
                    tint,
                    None,
                );
                clicked = response.clicked();
            },
        );
        if clicked {
            layout.activated = Some(node);
        }
    }
}
