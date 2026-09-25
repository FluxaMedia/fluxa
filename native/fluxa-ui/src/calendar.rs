use super::*;

pub fn draw_calendar(
    context: &egui::Context,
    viewport: Viewport,
    calendar: &CalendarModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-calendar"),
        calendar_scroll_max(viewport, calendar),
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, metrics.background);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 3, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let top = if compact {
        metrics.content_header_top_mobile
    } else {
        metrics.content_header_top
    };
    let title = if calendar.year > 0 && (1..=12).contains(&calendar.month) {
        format!(
            "{} {}",
            localized(
                &format!(
                    "calendar.month_{}",
                    month_name(calendar.month).to_lowercase()
                ),
                &calendar.language
            ),
            calendar.year
        )
    } else {
        "Calendar".to_owned()
    };
    egui::Area::new(Id::new("fluxa-shared-calendar-header"))
        .fixed_pos(Pos2::new(margin, top))
        .show(context, |ui| {
            ui.label(
                RichText::new(localized("nav.calendar", &calendar.language))
                    .size(if compact {
                        metrics.screen_title_size_mobile
                    } else if tv {
                        metrics.screen_title_size_tv
                    } else {
                        metrics.screen_title_size
                    })
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(metrics.control_gap);
            ui.label(
                RichText::new(localized("native.calendar.description", &calendar.language))
                    .size(if tv {
                        metrics.screen_body_size_tv
                    } else {
                        metrics.screen_body_size
                    })
                    .color(Color32::from_white_alpha(170)),
            );
            ui.add_space(metrics.section_gap);
            ui.horizontal(|ui| {
                let previous = components::button_with_text_size(
                    ui,
                    "‹",
                    metrics.screen_control_height,
                    metrics.screen_control_height,
                    components::ButtonKind::Secondary,
                    metrics.nav_label_size + 3.0,
                    metrics,
                );
                layout.focusable.push((NODE_CALENDAR_PREV, previous.rect));
                if previous.clicked() {
                    layout.activated = Some(NODE_CALENDAR_PREV);
                }
                ui.label(
                    RichText::new(title.as_str())
                        .size(metrics.nav_label_size + 3.0)
                        .strong(),
                );
                let next = components::button_with_text_size(
                    ui,
                    "›",
                    metrics.screen_control_height,
                    metrics.screen_control_height,
                    components::ButtonKind::Secondary,
                    metrics.nav_label_size + 3.0,
                    metrics,
                );
                layout.focusable.push((NODE_CALENDAR_NEXT, next.rect));
                if next.clicked() {
                    layout.activated = Some(NODE_CALENDAR_NEXT);
                }
                if calendar.is_loading {
                    ui.add_space(metrics.control_gap);
                    ui.label(
                        RichText::new(localized("native.calendar.loading", &calendar.language))
                            .color(Color32::from_white_alpha(140)),
                    );
                }
            });
        });
    if calendar.year > 0 && (1..=12).contains(&calendar.month) {
        let grid_top =
            top + metrics.screen_control_height * 2.0 + metrics.section_gap + metrics.control_gap
                - scroll_y;
        let grid_width = (viewport.width - margin * 2.0).max(1.0);
        let gap = if compact {
            metrics.calendar_grid_gap_mobile
        } else {
            metrics.calendar_grid_gap
        };
        let cell_width = ((grid_width - gap * 7.0) / 7.0).max(1.0);
        let cell_height = if compact {
            metrics.calendar_cell_height_mobile
        } else if tv {
            metrics.calendar_cell_height_tv
        } else {
            metrics.calendar_cell_height_desktop
        };
        let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        egui::Area::new(Id::new("fluxa-shared-calendar-grid"))
            .fixed_pos(Pos2::new(margin, grid_top))
            .show(context, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.horizontal(|ui| {
                    for weekday in weekdays {
                        ui.add_sized(
                            [cell_width, metrics.screen_control_height],
                            egui::Label::new(
                                RichText::new(localized(
                                    &format!("native.calendar.weekday.{}", weekday.to_lowercase()),
                                    &calendar.language,
                                ))
                                .size(metrics.screen_card_subtitle_size)
                                .strong()
                                .color(Color32::from_white_alpha(130)),
                            ),
                        );
                        ui.add_space(gap);
                    }
                });
                ui.add_space(metrics.control_gap);
                let leading = weekday_sunday_zero(calendar.year, calendar.month, 1) as usize;
                let total = days_in_month(calendar.year, calendar.month) as usize;
                let row_count = (leading + total).div_ceil(7);
                for row in 0..row_count {
                    ui.horizontal(|ui| {
                        for column in 0..7usize {
                            let index = row * 7 + column;
                            let day = index
                                .checked_sub(leading)
                                .and_then(|value| (value < total).then_some(value as u32 + 1));
                            draw_calendar_cell(
                                ui,
                                day,
                                calendar,
                                cell_width,
                                cell_height,
                                gap,
                                metrics,
                                assets,
                                &mut layout,
                            );
                        }
                    });
                    ui.add_space(gap);
                }
            });
    } else {
        egui::Area::new(Id::new("fluxa-shared-calendar-empty"))
            .fixed_pos(Pos2::new(margin, top + metrics.calendar_empty_offset))
            .show(context, |ui| {
                ui.label(
                    RichText::new(localized("calendar.empty", &calendar.language))
                        .size(if tv {
                            metrics.screen_section_title_size_tv
                        } else {
                            metrics.screen_section_title_size
                        })
                        .strong(),
                );
            });
    }
    if let Some(day) = calendar
        .selected_day
        .filter(|day| *day >= 1 && *day <= days_in_month(calendar.year, calendar.month) as u32)
    {
        let panel_width = if compact {
            (viewport.width - margin * 2.0).max(1.0)
        } else if tv {
            metrics.calendar_panel_width_tv
        } else {
            metrics.calendar_panel_width_desktop
        }
        .min(viewport.width - margin * 2.0);
        let panel_x = if compact {
            margin
        } else {
            viewport.width - margin - panel_width
        };
        let panel_top = if compact {
            let grid_top = top
                + metrics.screen_control_height * 2.0
                + metrics.section_gap
                + metrics.control_gap
                - scroll_y;
            let gap = metrics.calendar_grid_gap_mobile;
            let leading = weekday_sunday_zero(calendar.year, calendar.month, 1) as usize;
            let rows =
                (leading + days_in_month(calendar.year, calendar.month) as usize).div_ceil(7);
            grid_top
                + metrics.screen_control_height
                + metrics.control_gap
                + rows as f32 * (metrics.calendar_cell_height_mobile + gap)
                + metrics.section_gap
        } else {
            top
        };
        egui::Area::new(Id::new("fluxa-calendar-day-panel"))
            .fixed_pos(Pos2::new(panel_x, panel_top))
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                ui.set_width(panel_width);
                egui::Frame::new()
                    .fill(metrics.surface)
                    .corner_radius(metrics.card_radius)
                    .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(40)))
                    .inner_margin(metrics.card_content_padding)
                    .show(ui, |ui| {
                        ui.set_width((panel_width - metrics.card_content_padding * 2.0).max(1.0));
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} {}",
                                    day,
                                    localized(
                                        &format!(
                                            "calendar.month_{}",
                                            month_name(calendar.month).to_lowercase()
                                        ),
                                        &calendar.language
                                    )
                                ))
                                .size(metrics.screen_section_title_size)
                                .strong(),
                            );
                            let close = components::button_with_text_size(
                                ui,
                                "×",
                                metrics.screen_control_height,
                                metrics.screen_control_height,
                                components::ButtonKind::Secondary,
                                metrics.nav_label_size + 3.0,
                                metrics,
                            );
                            layout.focusable.push((NODE_CALENDAR_CLOSE_DAY, close.rect));
                            if close.clicked() {
                                layout.activated = Some(NODE_CALENDAR_CLOSE_DAY);
                            }
                        });
                        let entries: Vec<_> = calendar.entries_for_day(day).collect();
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                entries.len(),
                                localized("calendar.releases", &calendar.language)
                            ))
                            .size(metrics.screen_card_subtitle_size)
                            .color(metrics.text_secondary),
                        );
                        ui.add_space(metrics.control_gap);
                        if entries.is_empty() {
                            ui.label(
                                RichText::new(localized(
                                    "calendar.no_releases_this_day",
                                    &calendar.language,
                                ))
                                .color(metrics.text_muted),
                            );
                        }
                        let width = (panel_width - metrics.card_content_padding * 2.0).max(1.0);
                        for (index, entry) in entries.iter().take(10).enumerate() {
                            let response = components::calendar_release_row(
                                ui,
                                &entry.card,
                                width,
                                metrics,
                                assets,
                            );
                            let node = NODE_CALENDAR_EVENT_BASE + index as u64;
                            layout.focusable.push((node, response.rect));
                            if response.clicked() {
                                layout.activated = Some(node);
                            }
                            ui.add_space(metrics.control_gap);
                        }
                    });
            });
    }
    if calendar.selected_day.is_some() {
        layout.focusable.retain(|(id, _)| {
            *id == NODE_CALENDAR_CLOSE_DAY
                || (NODE_CALENDAR_EVENT_BASE..NODE_CALENDAR_EVENT_BASE + 10).contains(id)
                || is_navigation_node(*id)
        });
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    if let Some((_, rect)) = layout.focusable.iter().find(|(id, _)| Some(*id) == focused) {
        painter.rect_stroke(
            rect.expand(metrics.focus_ring_expand),
            metrics.focus_ring_radius,
            egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
    layout
}

fn draw_calendar_cell(
    ui: &mut egui::Ui,
    day: Option<u32>,
    calendar: &CalendarModel,
    width: f32,
    height: f32,
    gap: f32,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    ui.painter().rect_filled(
        rect,
        metrics.calendar_cell_radius,
        if day.is_some() {
            Color32::from_rgb(19, 20, 25)
        } else {
            Color32::from_rgb(10, 10, 12)
        },
    );
    if let Some(day) = day {
        let node = NODE_CALENDAR_DAY_BASE + day as u64;
        layout.focusable.push((node, rect));
        if response.clicked() {
            layout.activated = Some(node);
        }
        ui.painter().text(
            rect.left_top() + Vec2::new(metrics.calendar_day_padding, metrics.calendar_day_top),
            Align2::LEFT_TOP,
            day.to_string(),
            FontId::proportional(metrics.screen_card_title_size),
            Color32::from_white_alpha(195),
        );
        for (index, entry) in calendar.entries_for_day(day).take(2).enumerate() {
            let title = if entry.card.title.is_empty() {
                localized("calendar.new_release", &calendar.language)
            } else {
                entry.card.title.clone()
            };
            ui.painter().text(
                rect.left_top()
                    + Vec2::new(
                        metrics.calendar_day_padding,
                        metrics.calendar_entry_top
                            + index as f32 * metrics.calendar_entry_line_height,
                    ),
                Align2::LEFT_TOP,
                title,
                FontId::proportional(metrics.screen_card_subtitle_size),
                Color32::WHITE,
            );
            components::artwork_image(
                ui.painter(),
                Rect::from_min_size(
                    rect.right_top()
                        + Vec2::new(
                            -metrics.calendar_thumb_right_inset,
                            metrics.calendar_thumb_top,
                        ),
                    Vec2::new(
                        metrics.calendar_day_thumb_width,
                        metrics.calendar_day_thumb_height,
                    ),
                ),
                entry.card.artwork_url.as_deref(),
                [
                    (metrics.calendar_day_thumb_width * 1.35).ceil() as u32,
                    (metrics.calendar_day_thumb_height * 1.35).ceil() as u32,
                ],
                ArtworkPriority::Visible,
                Color32::WHITE,
                assets,
            );
        }
    }
    ui.add_space(gap);
}

fn month_name(month: i32) -> &'static str {
    [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ]
    .get(month.saturating_sub(1) as usize)
    .copied()
    .unwrap_or("Month")
}

pub(super) fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

pub(super) fn weekday_sunday_zero(year: i32, month: i32, day: i32) -> i32 {
    let (mut year, mut month) = (year, month);
    if month < 3 {
        year -= 1;
        month += 12;
    }
    (day + (13 * (month + 1)) / 5 + year + year / 4 - year / 100 + year / 400 + 6).rem_euclid(7)
}
