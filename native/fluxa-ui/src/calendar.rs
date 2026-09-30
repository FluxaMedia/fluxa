use super::*;

pub(super) struct CalendarGrid {
    pub margin: f32,
    pub top: f32,
    pub grid_top: f32,
    pub weekday_height: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub gap: f32,
    pub leading: usize,
    pub rows: usize,
}

impl CalendarGrid {
    pub fn new(viewport: Viewport, metrics: UiMetrics, calendar: &CalendarModel) -> Self {
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
        let mut header = metrics.screen_title_size * 1.25
            + metrics.control_gap
            + metrics.screen_control_height
            + metrics.section_gap;
        if compact {
            header += metrics.screen_control_height + metrics.section_gap;
        }
        let grid_top = top + header;
        let gap = if compact {
            metrics.calendar_grid_gap_mobile
        } else {
            metrics.calendar_grid_gap
        };
        let weekday_height = metrics.screen_card_subtitle_size * 2.2;
        let cell_width = ((viewport.width - margin * 2.0 - gap * 6.0) / 7.0).max(1.0);
        let leading = weekday_sunday_zero(calendar.year, calendar.month, 1) as usize;
        let rows = (leading + days_in_month(calendar.year, calendar.month) as usize).div_ceil(7);
        let minimum = if compact {
            metrics.calendar_cell_height_mobile
        } else if tv {
            metrics.calendar_cell_height_tv
        } else {
            metrics.calendar_cell_height_desktop
        };
        let available =
            (viewport.height - grid_top - weekday_height - margin) / rows.max(1) as f32 - gap;
        let cell_height = if compact {
            minimum
        } else {
            (cell_width * 0.62).min(available).max(minimum)
        };
        Self {
            margin,
            top,
            grid_top,
            weekday_height,
            cell_width,
            cell_height,
            gap,
            leading,
            rows,
        }
    }

    pub fn bottom(&self) -> f32 {
        self.grid_top + self.weekday_height + self.rows as f32 * (self.cell_height + self.gap)
    }
}

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
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 2, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let valid = calendar.year > 0 && (1..=12).contains(&calendar.month);
    let grid = CalendarGrid::new(viewport, metrics, calendar);
    let month_title = if valid {
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
        String::new()
    };
    egui::Area::new(Id::new("fluxa-shared-calendar-header"))
        .constrain(false)
        .fixed_pos(Pos2::new(grid.margin, grid.top - scroll_y))
        .show(context, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.label(
                RichText::new(localized("nav.library", &calendar.language))
                    .size(metrics.screen_title_size)
                    .strong()
                    .color(Color32::WHITE),
            );
        });
    crate::library::draw_library_sections(
        context,
        &mut layout,
        Pos2::new(
            grid.margin,
            grid.top - scroll_y + metrics.screen_title_size * 1.25 + metrics.control_gap,
        ),
        viewport.width - grid.margin * 2.0,
        1,
        None,
        &calendar.language,
        assets,
        metrics,
    );
    if valid {
        let nav_width = metrics.screen_control_height * 2.0
            + metrics.control_gap * 3.0
            + (metrics.nav_label_size + 3.0) * 8.5;
        let nav_pos = if compact {
            Pos2::new(
                grid.margin,
                grid.grid_top - metrics.screen_control_height - metrics.section_gap - scroll_y,
            )
        } else {
            Pos2::new(
                viewport.width - grid.margin - nav_width,
                grid.top + (metrics.screen_title_size * 1.25 - metrics.screen_control_height) * 0.5
                    - scroll_y,
            )
        };
        egui::Area::new(Id::new("fluxa-shared-calendar-month"))
            .constrain(false)
            .fixed_pos(nav_pos)
            .show(context, |ui| {
                ui.set_width(if compact {
                    viewport.width - grid.margin * 2.0
                } else {
                    nav_width
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.control_gap;
                    let label = RichText::new(month_title.as_str())
                        .size(metrics.nav_label_size + 3.0)
                        .strong()
                        .color(Color32::WHITE);
                    if compact {
                        ui.label(label.clone());
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = metrics.control_gap;
                        let next = components::button_with_text_size(
                            ui,
                            "›",
                            metrics.screen_control_height,
                            metrics.screen_control_height,
                            components::ButtonKind::Secondary,
                            metrics.nav_label_size + 3.0,
                            metrics,
                        );
                        let previous = components::button_with_text_size(
                            ui,
                            "‹",
                            metrics.screen_control_height,
                            metrics.screen_control_height,
                            components::ButtonKind::Secondary,
                            metrics.nav_label_size + 3.0,
                            metrics,
                        );
                        if !compact {
                            ui.label(label);
                        }
                        if calendar.is_loading {
                            ui.label(
                                RichText::new(localized("calendar.loading", &calendar.language))
                                    .size(metrics.screen_card_subtitle_size)
                                    .color(metrics.text_muted),
                            );
                        }
                        layout.focusable.push((NODE_CALENDAR_PREV, previous.rect));
                        layout.focusable.push((NODE_CALENDAR_NEXT, next.rect));
                        if previous.clicked() {
                            layout.activated = Some(NODE_CALENDAR_PREV);
                        }
                        if next.clicked() {
                            layout.activated = Some(NODE_CALENDAR_NEXT);
                        }
                    });
                });
            });
        let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let grid_painter = context.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            Id::new("fluxa-shared-calendar-grid"),
        ));
        let origin = Pos2::new(grid.margin, grid.grid_top - scroll_y);
        for (column, weekday) in weekdays.iter().enumerate() {
            let label = localized(
                &format!("calendar.weekday.{}", weekday.to_lowercase()),
                &calendar.language,
            );
            let column_x = column as f32 * (grid.cell_width + grid.gap);
            grid_painter.text(
                origin
                    + Vec2::new(
                        if compact {
                            column_x + grid.cell_width * 0.5
                        } else {
                            column_x + metrics.calendar_day_padding
                        },
                        grid.weekday_height * 0.5,
                    ),
                if compact {
                    Align2::CENTER_CENTER
                } else {
                    Align2::LEFT_CENTER
                },
                if compact {
                    label.chars().take(1).collect::<String>()
                } else {
                    label.to_uppercase()
                },
                crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0),
                metrics.text_muted,
            );
        }
        let total = days_in_month(calendar.year, calendar.month) as u32;
        egui::Area::new(Id::new("fluxa-shared-calendar-cells"))
            .constrain(false)
            .fixed_pos(origin + Vec2::new(0.0, grid.weekday_height))
            .show(context, |ui| {
                for day in 1..=total {
                    let index = grid.leading + day as usize - 1;
                    let rect = Rect::from_min_size(
                        ui.min_rect().min
                            + Vec2::new(
                                (index % 7) as f32 * (grid.cell_width + grid.gap),
                                (index / 7) as f32 * (grid.cell_height + grid.gap),
                            ),
                        Vec2::new(grid.cell_width, grid.cell_height),
                    );
                    draw_calendar_cell(ui, rect, day, calendar, metrics, assets, &mut layout);
                }
            });
    } else {
        egui::Area::new(Id::new("fluxa-shared-calendar-empty"))
            .constrain(false)
            .fixed_pos(Pos2::new(
                grid.margin,
                grid.top + metrics.calendar_empty_offset,
            ))
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
            (viewport.width - grid.margin * 2.0).max(1.0)
        } else if tv {
            metrics.calendar_panel_width_tv
        } else {
            metrics.calendar_panel_width_desktop
        }
        .min(viewport.width - grid.margin * 2.0);
        let panel_x = if compact {
            grid.margin
        } else {
            viewport.width - grid.margin - panel_width
        };
        let panel_top = if compact {
            grid.bottom() - scroll_y + metrics.section_gap
        } else {
            grid.top
        };
        egui::Area::new(Id::new("fluxa-calendar-day-panel"))
            .constrain(false)
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
    components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
    layout
}

fn draw_calendar_cell(
    ui: &mut egui::Ui,
    rect: Rect,
    day: u32,
    calendar: &CalendarModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let compact = rect.width() < 90.0;
    let response = ui.interact(rect, ui.id().with(("calendar-day", day)), Sense::click());
    let node = NODE_CALENDAR_DAY_BASE + day as u64;
    layout.focusable.push((node, rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
    let painter = ui.painter_at(rect);
    let radius = metrics.calendar_cell_radius;
    let entries: Vec<_> = calendar.entries_for_day(day).collect();
    let date = format!("{:04}-{:02}-{:02}", calendar.year, calendar.month, day);
    let today = calendar.today.as_deref() == Some(date.as_str());
    let past = calendar
        .today
        .as_deref()
        .is_some_and(|today| date.as_str() < today);
    let padding = metrics.calendar_day_padding;
    let mut has_art = false;
    if let Some(entry) = entries.first() {
        let url = entry.still_url.as_deref();
        let size = [
            (rect.width() * 1.5).ceil() as u32,
            (rect.height() * 1.5).ceil() as u32,
        ];
        if components::rounded_artwork(
            &painter,
            rect,
            radius,
            url,
            size,
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        ) {
            has_art = true;
        }
    }
    if !has_art {
        painter.rect_filled(
            rect,
            radius,
            if entries.is_empty() {
                Color32::from_white_alpha(6)
            } else {
                Color32::from_white_alpha(14)
            },
        );
    } else {
        if past {
            painter.rect_filled(rect, radius, Color32::from_black_alpha(110));
        }
        let scrim_top = rect.top() + rect.height() * 0.35;
        paint_vertical_gradient(
            &painter,
            Rect::from_min_max(Pos2::new(rect.left(), scrim_top), rect.right_bottom()),
            Color32::TRANSPARENT,
            Color32::from_black_alpha(220),
        );
        paint_vertical_gradient(
            &painter,
            Rect::from_min_max(
                rect.left_top(),
                Pos2::new(
                    rect.right(),
                    rect.top() + metrics.screen_card_title_size * 2.4,
                ),
            ),
            Color32::from_black_alpha(140),
            Color32::TRANSPARENT,
        );
    }
    let day_font = FontId::proportional(metrics.screen_card_title_size);
    let centered = compact && !has_art;
    let day_pos = if centered {
        rect.center()
    } else {
        rect.left_top() + Vec2::new(padding, metrics.calendar_day_top)
    };
    if calendar.selected_day == Some(day) {
        painter.rect_stroke(
            rect,
            radius,
            egui::Stroke::new(1.5, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }
    if today {
        let size = metrics.screen_card_title_size * 1.7;
        let badge = if centered {
            Rect::from_center_size(rect.center(), Vec2::splat(size))
        } else {
            Rect::from_min_size(day_pos - Vec2::splat(size * 0.2), Vec2::splat(size))
        };
        painter.circle_filled(badge.center(), size * 0.5, Color32::WHITE);
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            day.to_string(),
            day_font,
            Color32::from_rgb(6, 6, 6),
        );
    } else {
        painter.text(
            day_pos,
            if centered {
                Align2::CENTER_CENTER
            } else {
                Align2::LEFT_TOP
            },
            day.to_string(),
            day_font,
            if entries.is_empty() {
                Color32::from_white_alpha(if past { 70 } else { 150 })
            } else {
                Color32::WHITE
            },
        );
    }
    let text_width = rect.width() - padding * 2.0;
    if entries.len() > 1 {
        let more_font = crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0);
        let label = format!("+{}", entries.len() - 1);
        let galley = painter.layout_no_wrap(label, more_font, Color32::WHITE);
        let pill = Rect::from_min_size(
            Pos2::new(
                rect.right() - padding - galley.size().x - padding,
                rect.top() + metrics.calendar_day_top,
            ),
            galley.size() + Vec2::new(padding, padding * 0.5),
        );
        painter.rect_filled(pill, pill.height() * 0.5, Color32::from_black_alpha(170));
        painter.galley(pill.center() - galley.size() * 0.5, galley, Color32::WHITE);
    }
    let Some(entry) = entries.first() else {
        return;
    };
    if text_width < 90.0 {
        return;
    }
    let title_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 1.0);
    let episode_font = crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0);
    let mut baseline = rect.bottom() - padding;
    if !entry.episode.is_empty() {
        painter.text(
            Pos2::new(rect.left() + padding, baseline),
            Align2::LEFT_BOTTOM,
            truncate_to_width(&painter, &entry.episode, &episode_font, text_width),
            episode_font.clone(),
            metrics.text_secondary,
        );
        baseline -= episode_font.size * 1.35;
    }
    let show = if entry.show.is_empty() {
        localized("calendar.new_release", &calendar.language)
    } else {
        entry.show.clone()
    };
    painter.text(
        Pos2::new(rect.left() + padding, baseline),
        Align2::LEFT_BOTTOM,
        truncate_to_width(&painter, &show, &title_font, text_width),
        title_font,
        Color32::WHITE,
    );
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
