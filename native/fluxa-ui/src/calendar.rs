use super::*;
use crate::draw_text;
use crate::kit::layout_items as items;
use crate::kit::{lay, widget};
use std::cell::RefCell;

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
        let margin = metrics.screen_margin;
        let top = metrics.content_header_top;
        let header = metrics.screen_title_size * 1.25
            + metrics.control_gap
            + metrics.screen_control_height
            + metrics.section_gap;
        let grid_top = top + header;
        let gap = metrics.calendar_grid_gap;
        let weekday_height = metrics.screen_card_subtitle_size * 2.2;
        let cell_width = ((viewport.width - margin * 2.0 - gap * 6.0) / 7.0).max(1.0);
        let leading = weekday_sunday_zero(calendar.year, calendar.month, 1) as usize;
        let rows = (leading + days_in_month(calendar.year, calendar.month) as usize).div_ceil(7);
        let minimum = metrics.calendar_cell_height;
        let available =
            (viewport.height - grid_top - weekday_height - margin) / rows.max(1) as f32 - gap;
        let cell_height = (cell_width * 0.62).min(available).max(minimum);
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
    context: &crate::gfx::Context,
    viewport: Viewport,
    calendar: &CalendarModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 5, assets);
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
    let selected_day = calendar
        .selected_day
        .filter(|day| *day >= 1 && *day <= days_in_month(calendar.year, calendar.month) as u32);
    let available = (viewport.width - grid.margin * 2.0).max(1.0);
    let beside = available >= metrics.calendar_panel_width + metrics.calendar_beside_min_grid;
    let panel_width = if beside {
        metrics.calendar_panel_width
    } else {
        available
    };

    kit::page(context, viewport, "calendar", |ui| {
        let width = ui.available_width();
        kit::scroll_body(ui, viewport, "calendar", |ui| {
            let (bar, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), metrics.screen_title_size * 1.25),
                Sense::hover(),
            );
            let title_font = crate::fonts::bold(metrics.screen_title_size);
            let month_font = crate::fonts::bold(metrics.nav_label_size + 3.0);
            let loading_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
            let loading = localized("calendar.loading", &calendar.language);
            let button = metrics.screen_control_height;
            let mut items = vec![
                crate::flex::text(
                    ui.painter(),
                    &localized("nav.library", &calendar.language),
                    &title_font,
                )
                .id(0)
                .shrink(0.0),
                crate::flex::fill().id(1).min_width(0.0),
            ];
            if valid {
                if calendar.is_loading {
                    items.push(crate::flex::text(ui.painter(), &loading, &loading_font).id(2));
                }
                items.push(crate::flex::text(ui.painter(), &month_title, &month_font).id(3));
                items.push(crate::flex::slot(button).height(button).id(4));
                items.push(crate::flex::slot(button).height(button).id(5));
            }
            let placed = crate::flex::row(items)
                .align(crate::flex::Align::Center)
                .gap(metrics.control_gap)
                .solve(bar);
            let label = |ui: &mut crate::gfx::Ui, id: usize, text: RichText| {
                if let Some(rect) = placed.find(id) {
                    ui.put(rect, crate::gfx::Label::new(text));
                }
            };
            label(
                ui,
                0,
                RichText::new(localized("nav.library", &calendar.language))
                    .size(metrics.screen_title_size)
                    .strong()
                    .color(text_primary()),
            );
            if valid {
                label(
                    ui,
                    2,
                    RichText::new(loading.as_str())
                        .size(metrics.screen_card_subtitle_size)
                        .color(metrics.text_muted),
                );
                label(
                    ui,
                    3,
                    RichText::new(month_title.as_str())
                        .size(metrics.nav_label_size + 3.0)
                        .strong()
                        .color(text_primary()),
                );
                let mut arrow = |ui: &mut crate::gfx::Ui, id: usize, glyph: &str| {
                    let rect = placed.find(id).unwrap_or(bar);
                    let mut child = ui.new_child(
                        crate::gfx::UiBuilder::new()
                            .max_rect(rect)
                            .layout(crate::gfx::Layout::top_down(crate::gfx::Align::Center)),
                    );
                    components::button_with_text_size(
                        &mut child,
                        glyph,
                        button,
                        button,
                        components::ButtonKind::Secondary,
                        metrics.nav_label_size + 3.0,
                        metrics,
                    )
                };
                let previous = arrow(ui, 4, "‹");
                let next = arrow(ui, 5, "›");
                layout.focusable.push((NODE_CALENDAR_PREV, previous.rect));
                layout.focusable.push((NODE_CALENDAR_NEXT, next.rect));
                if previous.clicked() {
                    layout.activated = Some(NODE_CALENDAR_PREV);
                }
                if next.clicked() {
                    layout.activated = Some(NODE_CALENDAR_NEXT);
                }
            }
            let layout_cell = RefCell::new(&mut layout);
            let assets_cell = RefCell::new(&mut *assets);
            let sections = widget(ui, |ui| {
                let mut scratch = HomeLayout::default();
                let mut guard = layout_cell.borrow_mut();
                let target = if crate::kit::probing() {
                    &mut scratch
                } else {
                    &mut **guard
                };
                crate::library::library_sections_row(
                    ui,
                    target,
                    1,
                    None,
                    &calendar.language,
                    &**assets_cell.borrow(),
                    metrics,
                );
            });
            lay(
                ui,
                vec![
                    items::gap(metrics.control_gap),
                    sections,
                    items::gap(metrics.section_gap),
                ],
            );
            if !valid {
                let used = ui.min_rect().height() + ui.item_spacing().y;
                let empty = widget(ui, |ui| {
                    ui.label(
                        RichText::new(localized("calendar.empty", &calendar.language))
                            .size(metrics.section_title_size)
                            .strong(),
                    );
                });
                lay(
                    ui,
                    vec![
                        items::gap((metrics.calendar_empty_offset - used).max(metrics.control_gap)),
                        empty,
                    ],
                );
                return;
            }
            let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            let month_width = if beside {
                (width - panel_width - metrics.section_gap).max(grid.cell_width)
            } else {
                width
            };
            let month = widget(ui, |ui| {
                let mut scratch = HomeLayout::default();
                let mut guard = layout_cell.borrow_mut();
                let target = if crate::kit::probing() {
                    &mut scratch
                } else {
                    &mut **guard
                };
                ui.scope(|ui| {
                    ui.set_max_width(month_width);
                    let side = ((ui.available_width() - grid.gap * 6.0) / 7.0).max(1.0);
                    kit::grid(7)
                        .cell(Vec2::new(side, grid.weekday_height))
                        .per_row(7)
                        .gap_px(grid.gap)
                        .show(ui, |ui, column| {
                            let label = localized(
                                &format!("calendar.weekday.{}", weekdays[column].to_lowercase()),
                                &calendar.language,
                            );
                            draw_text(
                                ui.painter(),
                                ui.max_rect().left_center()
                                    + Vec2::new(metrics.calendar_day_padding, 0.0),
                                Align2::LEFT_CENTER,
                                label.to_uppercase(),
                                crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0),
                                metrics.text_muted,
                                None,
                            );
                        });
                    let total = days_in_month(calendar.year, calendar.month) as usize;
                    kit::grid(grid.leading + total)
                        .cell(Vec2::new(side, grid.cell_height))
                        .per_row(7)
                        .gap_px(grid.gap)
                        .every_cell()
                        .show(ui, |ui, index| {
                            if index >= grid.leading {
                                let day = (index - grid.leading + 1) as u32;
                                let rect = ui.max_rect();
                                draw_calendar_cell(
                                    ui,
                                    rect,
                                    day,
                                    calendar,
                                    metrics,
                                    &mut **assets_cell.borrow_mut(),
                                    &mut *target,
                                );
                            }
                        });
                });
            });
            lay(ui, vec![month]);
            if let Some(day) = selected_day
                && !beside
            {
                let panel = widget(ui, |ui| {
                    let mut scratch = HomeLayout::default();
                    let mut guard = layout_cell.borrow_mut();
                    let target = if crate::kit::probing() {
                        &mut scratch
                    } else {
                        &mut **guard
                    };
                    calendar_day_panel(
                        ui,
                        panel_width,
                        day,
                        calendar,
                        metrics,
                        &mut **assets_cell.borrow_mut(),
                        target,
                    );
                });
                lay(ui, vec![items::gap(metrics.section_gap), panel]);
            }
        });
    });
    if let Some(day) = selected_day
        && beside
    {
        components::placed_front(
            context,
            Id::new("fluxa-calendar-day-panel"),
            Pos2::new(viewport.width - grid.margin - panel_width, grid.top),
            |ui| {
                calendar_day_panel(ui, panel_width, day, calendar, metrics, assets, &mut layout);
            },
        );
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
    layout
}

fn calendar_day_panel(
    ui: &mut crate::gfx::Ui,
    panel_width: f32,
    day: u32,
    calendar: &CalendarModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    ui.set_width(panel_width);
    let layout = RefCell::new(layout);
    let assets = RefCell::new(assets);
    crate::gfx::Frame::new()
        .fill(metrics.surface)
        .corner_radius(metrics.card_radius)
        .stroke(crate::gfx::Stroke::new(
            stroke::hairline(),
            crate::stylesheet::color("raised"),
        ))
        .inner_margin(metrics.card_content_padding)
        .show(ui, |ui| {
            ui.set_width((panel_width - metrics.card_content_padding * 2.0).max(1.0));
            let title = widget(ui, |ui| {
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
            });
            let close = widget(ui, |ui| {
                let close = components::button_with_text_size(
                    ui,
                    "×",
                    metrics.screen_control_height,
                    metrics.screen_control_height,
                    components::ButtonKind::Secondary,
                    metrics.nav_label_size + 3.0,
                    metrics,
                );
                if !crate::kit::probing() {
                    components::focus_action(
                        &mut **layout.borrow_mut(),
                        NODE_CALENDAR_CLOSE_DAY,
                        &close,
                    );
                }
            });
            kit::view()
                .row()
                .gap(Space::S12)
                .items(ui, vec![title, close]);
            let entries: Vec<_> = calendar.entries_for_day(day).collect();
            let count = widget(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        entries.len(),
                        localized("calendar.releases", &calendar.language)
                    ))
                    .size(metrics.screen_card_subtitle_size)
                    .color(metrics.text_secondary),
                );
            });
            let mut children = vec![count, items::gap(metrics.control_gap)];
            if entries.is_empty() {
                children.push(widget(ui, |ui| {
                    ui.label(
                        RichText::new(localized(
                            "calendar.no_releases_this_day",
                            &calendar.language,
                        ))
                        .color(metrics.text_muted),
                    );
                }));
            }
            let width = (panel_width - metrics.card_content_padding * 2.0).max(1.0);
            let (layout, assets) = (&layout, &assets);
            for (index, entry) in entries.iter().take(10).enumerate() {
                children.push(widget(ui, move |ui| {
                    let response = components::calendar_release_row(
                        ui,
                        &entry.card,
                        width,
                        metrics,
                        &mut **assets.borrow_mut(),
                    );
                    if !crate::kit::probing() {
                        let node = NODE_CALENDAR_EVENT_BASE + index as u64;
                        components::focus_action(&mut **layout.borrow_mut(), node, &response);
                    }
                }));
                children.push(items::gap(metrics.control_gap));
            }
            lay(ui, children);
        });
}

fn draw_calendar_cell(
    ui: &mut crate::gfx::Ui,
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
                crate::stylesheet::color("well")
            } else {
                crate::stylesheet::color("well")
            },
        );
    } else {
        if past {
            painter.rect_filled(rect, radius, shade(Shade::Veil));
        }
        let scrim_top = rect.top() + rect.height() * 0.35;
        paint_vertical_gradient(
            &painter,
            Rect::from_min_max(Pos2::new(rect.left(), scrim_top), rect.right_bottom()),
            Color32::TRANSPARENT,
            shade(Shade::Strong),
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
            shade(Shade::Veil),
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
            crate::gfx::Stroke::new(stroke::icon(), text_primary()),
            crate::gfx::StrokeKind::Inside,
        );
    }
    if today {
        let size = metrics.screen_card_title_size * 1.7;
        let badge = if centered {
            crate::flex::centered(rect, Vec2::splat(size))
        } else {
            crate::flex::anchored(
                rect,
                Align2::LEFT_TOP,
                Vec2::splat(size),
                0.0,
            )
            .translate(
                Vec2::new(padding, metrics.calendar_day_top) - Vec2::splat(size * 0.2),
            )
        };
        painter.circle_filled(badge.center(), size * 0.5, text_primary());
        draw_text(
            &painter,
            badge.center(),
            Align2::CENTER_CENTER,
            day.to_string(),
            day_font,
            background(),
            None,
        );
    } else {
        draw_text(
            &painter,
            day_pos,
            if centered {
                Align2::CENTER_CENTER
            } else {
                Align2::LEFT_TOP
            },
            day.to_string(),
            day_font,
            if entries.is_empty() {
                ink(if past { 70 } else { 150 })
            } else {
                text_primary()
            },
            None,
        );
    }
    let text_width = rect.width() - padding * 2.0;
    if entries.len() > 1 {
        let more_font = crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0);
        let label = format!("+{}", entries.len() - 1);
        let galley = painter.layout_no_wrap(label, more_font, Color32::WHITE);
        let pill = crate::flex::anchored(
            rect,
            Align2::RIGHT_TOP,
            galley.size() + Vec2::new(padding, padding * 0.5),
            0.0,
        )
        .translate(Vec2::new(-padding, metrics.calendar_day_top));
        painter.rect_filled(pill, pill.height() * 0.5, shade(Shade::Scrim));
        painter.galley(pill.center() - galley.size() * 0.5, galley, text_primary());
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
        draw_text(
            &painter,
            Pos2::new(rect.left() + padding, baseline),
            Align2::LEFT_BOTTOM,
            &entry.episode,
            episode_font.clone(),
            metrics.text_secondary,
            Some(text_width),
        );
        baseline -= episode_font.size * 1.35;
    }
    let show = if entry.show.is_empty() {
        localized("calendar.new_release", &calendar.language)
    } else {
        entry.show.clone()
    };
    draw_text(
        &painter,
        Pos2::new(rect.left() + padding, baseline),
        Align2::LEFT_BOTTOM,
        &show,
        title_font,
        text_primary(),
        Some(text_width),
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
