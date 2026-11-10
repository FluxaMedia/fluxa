use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets, Node};

const TITLE: usize = 1;
const SEARCH: usize = 2;
const BACK: usize = 3;
const ICON: usize = 4;
const LABEL: usize = 5;
const DETAIL: usize = 6;
const TRACK: usize = 7;

fn block(height: f32) -> Node {
    flex::leaf(Vec2::ZERO).height(height)
}

pub(super) fn at(bounds: Rect, left: f32, top: f32, size: Vec2) -> Rect {
    flex::column([flex::leaf(Vec2::ZERO).width(size.x).height(size.y).id(0)])
        .padding(Insets {
            left,
            top,
            ..Insets::default()
        })
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}

pub(super) fn padded(bounds: Rect, insets: Insets) -> Rect {
    flex::stack(Align2::CENTER_CENTER, [flex::fill().id(0)])
        .padding(insets)
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}

pub(super) fn stacked_lines(row: Rect, above: f32, below: f32) -> (Rect, Rect) {
    let lines = flex::column([
        flex::leaf(Vec2::ZERO).height(above * 2.0).id(0),
        flex::leaf(Vec2::ZERO).height(below * 2.0).id(1),
    ])
    .align(Align::Stretch)
    .padding(Insets {
        top: row.height() * 0.5 - above * 2.0,
        ..Insets::default()
    })
    .solve(row);
    (lines.find(0).unwrap_or(row), lines.find(1).unwrap_or(row))
}

pub(super) fn aligned(bounds: Rect, align: Align2, size: Vec2, inset: f32) -> Rect {
    flex::stack(align, [flex::leaf(size).id(0)])
        .padding(Insets::xy(inset, 0.0))
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}

pub(super) fn screen(viewport: Viewport) -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height))
}

#[derive(Clone, Copy)]
pub(super) struct Frame<'a> {
    pub context: &'a crate::gfx::Context,
    pub viewport: Viewport,
    pub settings: &'a SettingsModel,
    pub language: &'a str,
    pub metrics: UiMetrics,
    pub painter: &'a crate::gfx::Painter,
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
    let top = metrics.settings_top - scroll_y;
    let width = (viewport.width - margin * 2.0).max(1.0);
    let edge = Space::S8.px(&metrics);
    let root = !settings.section_open || searching;
    if root {
        let header = flex::column([
            flex::leaf(Vec2::new(0.0, metrics.screen_title_size)).id(TITLE),
            block(search_height).id(SEARCH),
        ])
        .align(Align::Stretch)
        .gap(metrics.section_gap)
        .padding(Insets {
            left: margin,
            right: margin,
            top,
            bottom: 0.0,
        })
        .solve(screen(viewport));
        if let Some(rect) = header.find(TITLE) {
            flex::paint_text(
                &painter,
                rect,
                &localized("nav.settings", language),
                &crate::fonts::bold(metrics.screen_title_size),
                text_primary(),
                Align2::LEFT_TOP,
            );
        }
        if let Some(rect) = header.find(SEARCH) {
            search_field(frame, layout, rect);
        }
    } else {
        let page_title = if settings.page_open {
            section_label(SETTINGS_SECTIONS[active_section].title, language)
        } else {
            section_label(SETTINGS_SECTIONS[active_section].category, language)
        };
        let title_font = crate::fonts::bold(metrics.text.title + Dp::D2.px());
        let bar = flex::column([flex::row([
            flex::leaf(Vec2::splat(metrics.icon_button_size)).id(BACK),
            flex::leaf(flex::text_size(&painter, &page_title, &title_font))
                .grow(1.0)
                .id(LABEL),
        ])
        .align(Align::Center)
        .gap(Space::S4.px(&metrics))
        .height(metrics.icon_button_size)])
        .padding(Insets {
            left: margin - edge,
            top,
            ..Insets::default()
        })
        .solve(screen(viewport));
        let back_rect = bar.find(BACK).unwrap_or(Rect::NOTHING);
        let sticky_on = scroll_y > edge;
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
            margin - edge,
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
        if let Some(rect) = bar.find(LABEL) {
            flex::paint_text(
                &painter,
                rect,
                &page_title,
                &title_font,
                text_primary(),
                Align2::LEFT_CENTER,
            );
        }
    }
    if !settings.section_open && !searching {
        draw_compact_settings_list(
            context,
            viewport,
            &painter,
            assets,
            language,
            metrics,
            at(
                screen(viewport),
                margin,
                section_top - scroll_y,
                Vec2::new(width, 0.0),
            ),
            layout,
        );
        return true;
    }
    false
}

const UPDATE: usize = 1000;
const HEADING: usize = 2000;
const ITEM: usize = 3000;

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
        .min((viewport.width - margin * 2.0).max(1.0) * metrics.settings_nav_max_share);
    let nav_item_height = if tv {
        metrics.screen_control_height
    } else {
        metrics.settings_nav_item_height_desktop
    };
    let nav_item_gap = if tv {
        metrics.control_gap * metrics.settings_nav_gap_share_tv
    } else {
        metrics.settings_nav_item_gap_desktop
    };
    let nav_scroll = if desktop { 0.0 } else { scroll_y };
    let nav_x = if desktop { 0.0 } else { margin };
    let nav_inset = if desktop {
        Dp::D16.px()
    } else {
        metrics.control_gap
    };
    let panel_width = if desktop {
        nav_width + margin
    } else {
        nav_width
    };
    let screen = screen(viewport);
    let nav_panel = if desktop {
        at(screen, 0.0, 0.0, Vec2::new(panel_width, viewport.height))
    } else {
        at(
            screen,
            margin,
            section_top - nav_scroll,
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
            crate::gfx::Stroke::new(stroke::hairline(), crate::stylesheet::color("divider")),
        );
    } else {
        painter.rect_filled(nav_panel, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            nav_panel,
            metrics.card_radius,
            crate::gfx::Stroke::new(stroke::hairline(), crate::stylesheet::color("divider")),
            crate::gfx::StrokeKind::Inside,
        );
    }
    if desktop {
        let title = localized("nav.settings", language);
        let font = crate::fonts::bold(metrics.settings_section_title_size_desktop + Dp::D2.px());
        let size = flex::text_size(&painter, &title, &font);
        flex::paint_text(
            &painter,
            at(
                screen,
                nav_x + nav_inset + Space::S2.px(&metrics),
                section_top - nav_scroll + Dp::D16.px(),
                size,
            ),
            &title,
            &font,
            text_primary(),
            Align2::LEFT_TOP,
        );
    }
    let search_top = if desktop {
        Dp::D56.px()
    } else {
        metrics.control_gap
    };
    let active_category = SETTINGS_SECTIONS[active_section].category;
    let status = crate::update::nav_status();
    let mut children = vec![
        block(search_height).id(SEARCH),
        block(metrics.control_gap * metrics.settings_nav_top_share),
    ];
    if status.is_some() {
        children.push(block(nav_item_height).id(UPDATE));
        children.push(block(nav_item_gap));
    }
    let mut last_group = "";
    for index in categories(viewport) {
        let group = category_group(SETTINGS_SECTIONS[index].category);
        if group != last_group {
            if !last_group.is_empty() {
                children.push(block(metrics.settings_nav_group_gap));
            }
            children.push(block(metrics.settings_nav_group_height).id(HEADING + index));
            last_group = group;
        }
        children.push(block(nav_item_height).id(ITEM + index));
        children.push(block(nav_item_gap));
    }
    let nav_left = nav_x + nav_inset;
    let list = flex::column(children)
        .align(Align::Stretch)
        .width(nav_left + panel_width - nav_inset * 2.0)
        .padding(Insets {
            left: nav_left,
            top: section_top + search_top - nav_scroll,
            ..Insets::default()
        });
    let placed = list.solve(Rect::from_min_size(Pos2::ZERO, list.outer_size()));
    if let Some(rect) = placed.find(SEARCH) {
        search_field(frame, layout, rect);
    }
    if let (Some(status), Some(rect)) = (status, placed.find(UPDATE)) {
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
            let title_font =
                crate::fonts::medium(metrics.nav_label_size + metrics.settings_nav_label_boost);
            let detail_font = crate::fonts::medium(metrics.nav_label_size);
            let inset = metrics.settings_nav_row_inset;
            let line = flex::row([
                flex::leaf(Vec2::splat(metrics.settings_nav_icon)).id(ICON),
                flex::leaf(flex::text_size(ui.painter(), &status.title, &title_font))
                    .grow(1.0)
                    .id(LABEL),
                flex::leaf(flex::text_size(ui.painter(), &status.detail, &detail_font)).id(DETAIL),
            ])
            .align(Align::Center)
            .gap(Space::S12.px(&metrics))
            .padding(Insets {
                left: inset,
                right: inset,
                ..Insets::default()
            })
            .solve(item_rect);
            if let (Some(icon), Some(slot)) = (assets.icon(status.icon), line.find(ICON)) {
                ui.painter().image(icon, slot, full_uv(), accent());
            }
            if let Some(slot) = line.find(LABEL) {
                flex::paint_text(
                    ui.painter(),
                    slot,
                    &status.title,
                    &title_font,
                    text_primary(),
                    Align2::LEFT_CENTER,
                );
            }
            if let Some(slot) = line.find(DETAIL) {
                flex::paint_text(
                    ui.painter(),
                    slot,
                    &status.detail,
                    &detail_font,
                    crate::stylesheet::color("glyph"),
                    Align2::RIGHT_CENTER,
                );
            }
            if let Some(fraction) = status.progress {
                let track = flex::column([flex::leaf(Vec2::new(
                    0.0,
                    metrics.settings_nav_progress_height,
                ))
                .id(TRACK)])
                .align(Align::Stretch)
                .justify(flex::Justify::End)
                .padding(Insets {
                    left: inset,
                    right: inset,
                    bottom: metrics.settings_nav_progress_inset,
                    top: 0.0,
                })
                .solve(item_rect)
                .find(TRACK)
                .unwrap_or(item_rect);
                ui.painter().rect_filled(
                    track,
                    metrics.settings_nav_progress_radius,
                    crate::stylesheet::color("track"),
                );
                ui.painter().rect_filled(
                    flex::anchored(
                        track,
                        Align2::LEFT_CENTER,
                        Vec2::new(track.width() * fraction, track.height()),
                        0.0,
                    ),
                    metrics.settings_nav_progress_radius,
                    accent(),
                );
            }
            clicked = response.clicked();
        });
        if clicked {
            layout.activated = Some(NODE_UPDATE_OPEN);
        }
    }
    for index in categories(viewport) {
        let section = &SETTINGS_SECTIONS[index];
        let node = NODE_SETTINGS_SECTION_BASE + index as u64;
        if let Some(band) = placed.find(HEADING + index) {
            nav_heading(
                &painter,
                Rect::from_min_max(
                    Pos2::new(band.left() + metrics.settings_nav_row_inset, band.top()),
                    band.right_bottom(),
                ),
                category_group(section.category),
                language,
                metrics,
            );
        }
        let Some(rect) = placed.find(ITEM + index) else {
            continue;
        };
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
                let label = section_label(section.category, language);
                let font =
                    crate::fonts::medium(metrics.nav_label_size + metrics.settings_nav_label_boost);
                let line = flex::row([
                    flex::leaf(Vec2::splat(metrics.settings_nav_icon)).id(ICON),
                    flex::leaf(flex::text_size(ui.painter(), &label, &font))
                        .grow(1.0)
                        .id(LABEL),
                ])
                .align(Align::Center)
                .gap(Space::S12.px(&metrics))
                .padding(Insets {
                    left: metrics.settings_nav_row_inset,
                    ..Insets::default()
                })
                .solve(item_rect);
                if let Some(slot) = line.find(ICON) {
                    settings_category_icon(
                        assets,
                        ui.painter(),
                        slot,
                        section.category,
                        tint,
                        metrics.settings_nav_icon,
                    );
                }
                if let Some(slot) = line.find(LABEL) {
                    flex::paint_text(ui.painter(), slot, &label, &font, tint, Align2::LEFT_CENTER);
                }
                clicked = response.clicked();
            },
        );
        if clicked {
            layout.activated = Some(node);
        }
    }
}

pub(super) fn slot(ui: &mut crate::gfx::Ui, size: Vec2, margin: Insets) -> Rect {
    let found = std::cell::Cell::new(Rect::NOTHING);
    crate::kit::lay(
        ui,
        vec![crate::kit::item(size, |_, rect| found.set(rect)).margin(margin)],
    );
    found.get()
}

pub(super) fn padded_body<R>(
    ui: &mut crate::gfx::Ui,
    top: f32,
    bottom: f32,
    add: impl FnOnce(&mut crate::gfx::Ui) -> R,
) -> R {
    let bounds = ui.available_rect_before_wrap();
    let inner = at(
        bounds,
        0.0,
        top,
        Vec2::new(bounds.width(), (bounds.height() - top).max(0.0)),
    );
    let mut body = ui.new_child(crate::gfx::UiBuilder::new().max_rect(inner));
    body.spacing_mut().item_spacing = ui.spacing().item_spacing;
    let result = add(&mut body);
    let height = top + body.min_rect().height() + bottom;
    crate::kit::lay(
        ui,
        vec![crate::kit::item(Vec2::new(bounds.width(), height), |_, _| {})],
    );
    result
}

pub(super) fn tight<R>(ui: &mut crate::gfx::Ui, add: impl FnOnce(&mut crate::gfx::Ui) -> R) -> R {
    ui.scope(|inner| {
        inner.spacing_mut().item_spacing = Vec2::ZERO;
        add(inner)
    })
    .inner
}
