use super::*;
use crate::Space;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ButtonKind {
    Primary,
    Secondary,
    Selected,
    Danger,
}

pub(crate) fn button(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    metrics: UiMetrics,
) -> Response {
    button_with_text_size(
        ui,
        label,
        width,
        height,
        kind,
        metrics.nav_label_size + Dp::D2.px(),
        metrics,
    )
}

pub(crate) fn button_auto_width(
    ui: &mut Ui,
    label: &str,
    kind: ButtonKind,
    text_size: f32,
    metrics: UiMetrics,
) -> Response {
    let font = FontId::proportional(text_size);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font, Color32::WHITE)
            .size()
            .x
    });
    button_with_text_size(
        ui,
        label,
        text_width + metrics.control_gap * 2.0,
        metrics.screen_control_height,
        kind,
        text_size,
        metrics,
    )
}

pub(crate) fn overlay_chip(painter: &Painter, rect: Rect, outline: Option<Color32>) {
    painter.rect_filled(rect, radius::chip(), badge_fill());
    if let Some(color) = outline {
        painter.rect_stroke(
            rect,
            radius::chip(),
            crate::gfx::Stroke::new(stroke::hairline(), color),
            crate::gfx::StrokeKind::Inside,
        );
    }
}

pub(crate) fn focus_action(layout: &mut HomeLayout, node: u64, response: &Response) {
    layout.focusable.push((node, response.rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
}

pub(crate) fn placed<R>(
    context: &crate::gfx::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> crate::gfx::InnerResponse<R> {
    crate::gfx::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .show(context, add)
}

pub(crate) fn placed_front<R>(
    context: &crate::gfx::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> crate::gfx::InnerResponse<R> {
    crate::gfx::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .order(crate::gfx::Order::Foreground)
        .show(context, add)
}

pub(crate) fn button_with_text_size(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    text_size: f32,
    metrics: UiMetrics,
) -> Response {
    button_core(
        ui,
        label,
        width,
        height,
        kind,
        text_size,
        true,
        Some(metrics),
    )
}

impl ButtonKind {
    fn rule(self) -> &'static str {
        match self {
            ButtonKind::Primary => "button.primary",
            ButtonKind::Secondary => "button.secondary",
            ButtonKind::Selected => "button.selected",
            ButtonKind::Danger => "button.danger",
        }
    }
}

pub(crate) fn button_core(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    text_size: f32,
    enabled: bool,
    metrics: Option<UiMetrics>,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let style = crate::stylesheet::resolve(
        ui.ctx(),
        kind.rule(),
        response.id,
        crate::stylesheet::State {
            selected: false,
            hovered: response.hovered(),
            pressed: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
            enabled,
        },
        metrics.as_ref(),
    );
    let (fill, text) = (style.fill, style.text);
    let painter = ui.painter();
    crate::motion::press_scale(painter, rect, || {
        painter.rect_filled(rect, height * 0.5, fill);
        draw_text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            crate::fonts::bold(text_size),
            text,
            None,
        );
    });
    if enabled && response.has_focus() {
        let look = metrics.unwrap_or_else(crate::metrics::active);
        let (expand, width) = (look.focus_ring_expand, look.focus_ring_width);
        painter.rect_stroke(
            rect.expand(expand),
            height * 0.5 + expand,
            crate::gfx::Stroke::new(width, accent()),
            crate::gfx::StrokeKind::Outside,
        );
    }
    response
}

pub(crate) fn play_button(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    progress: Option<f32>,
) -> Response {
    let unavailable = assets.playback_unavailable().map(str::to_owned);
    let metrics = crate::metrics::active();
    let chrome = Dp::D44.px() + Dp::D24.px();
    let width = match (&unavailable, width) {
        (Some(reason), Some(limit)) => Some(limit.max(
            crate::paint::text_width(ui.painter(), reason, FontId::proportional(text_size))
                + chrome,
        )),
        _ => width,
    };
    pill_button_styled(
        ui,
        assets.icon("PlayFilled"),
        unavailable.as_deref().unwrap_or(label),
        width,
        height,
        text_size,
        true,
        progress.filter(|_| unavailable.is_none()),
        unavailable.is_some(),
    )
}

pub(crate) fn link_tile(
    ui: &mut Ui,
    size: Vec2,
    icon: Option<crate::gfx::TextureId>,
    arrow: Option<crate::gfx::TextureId>,
    title: &str,
    subtitle: &str,
    framed: bool,
    metrics: UiMetrics,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    if framed {
        painter.rect_filled(rect, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            rect,
            metrics.card_radius,
            crate::gfx::Stroke::new(stroke::hairline(), ink(34)),
            crate::gfx::StrokeKind::Inside,
        );
    }
    if response.hovered() {
        painter.rect_filled(
            rect,
            if framed { metrics.card_radius } else { 0.0 },
            Color32::from_white_alpha(10),
        );
    }
    let badge_size = Vec2::splat(Dp::D44.px());
    let side = Space::S8.px(&metrics);
    let nudge = Space::S4.px(&metrics);
    let arrow_size = Vec2::splat(Dp::D16.px());
    let reserved =
        Space::S16.px(&metrics) + badge_size.x + side + nudge + side + arrow_size.x + Dp::D14.px();
    let text_width = (rect.width() - reserved).max(0.0);
    let title_galley = painter.layout(
        title.to_owned(),
        crate::fonts::bold(metrics.screen_card_subtitle_size + 4.0),
        text_primary(),
        text_width,
    );
    let subtitle_galley = painter.layout(
        subtitle.to_owned(),
        crate::fonts::regular(metrics.screen_card_subtitle_size + 1.0),
        metrics.text_secondary,
        text_width,
    );
    let tree = flex::row([
        flex::leaf(badge_size).id(0),
        flex::column([
            flex::leaf(title_galley.size()).id(1),
            flex::leaf(subtitle_galley.size()).id(2),
        ])
        .gap(Dp::D4.px())
        .padding(Insets {
            left: nudge,
            ..Insets::default()
        })
        .grow(1.0)
        .align(crate::flex::Align::Start),
        flex::leaf(arrow_size).id(3),
    ])
    .align(Align::Center)
    .gap(side)
    .padding(Insets {
        left: Space::S16.px(&metrics),
        right: Dp::D14.px(),
        ..Insets::default()
    });
    let placed = tree.solve(rect);
    let badge = placed.find(0).unwrap_or(rect);
    painter.rect_filled(badge, Dp::D14.px(), Color32::from_white_alpha(12));
    if let Some(icon) = icon {
        painter.image(
            icon,
            flex::anchored(badge, Align2::CENTER_CENTER, Vec2::splat(Dp::D24.px()), 0.0),
            full_uv(),
            Color32::WHITE,
        );
    }
    if let Some(slot) = placed.find(1) {
        painter.galley(slot.min, title_galley, text_primary());
    }
    if let Some(slot) = placed.find(2) {
        painter.galley(slot.min, subtitle_galley, metrics.text_secondary);
    }
    if let (Some(arrow), Some(slot)) = (arrow, placed.find(3)) {
        painter.image(arrow, slot, full_uv(), ink(110));
    }
    response
}

pub(crate) fn step_button(ui: &mut Ui, size: f32, plus: bool, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::splat(size),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let painter = ui.painter();
    let alpha = if enabled { 255 } else { 80 };
    let fill = if enabled && response.hovered() {
        Color32::from_white_alpha(28)
    } else {
        Color32::from_white_alpha(16)
    };
    painter.circle_filled(rect.center(), size * 0.5, fill);
    let metrics = crate::metrics::active();
    let line = crate::gfx::Stroke::new(
        metrics.control_step_stroke,
        text_primary().gamma_multiply(alpha as f32 / 255.0),
    );
    let arm = size * metrics.control_step_arm;
    painter.line_segment(
        [
            rect.center() - Vec2::new(arm, 0.0),
            rect.center() + Vec2::new(arm, 0.0),
        ],
        line,
    );
    if plus {
        painter.line_segment(
            [
                rect.center() - Vec2::new(0.0, arm),
                rect.center() + Vec2::new(0.0, arm),
            ],
            line,
        );
    }
    response
}

pub(crate) fn pill_button(
    ui: &mut Ui,
    icon: Option<crate::gfx::TextureId>,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    primary: bool,
    progress: Option<f32>,
) -> Response {
    pill_button_styled(
        ui, icon, label, width, height, text_size, primary, progress, false,
    )
}

pub(super) fn pill_button_styled(
    ui: &mut Ui,
    icon: Option<crate::gfx::TextureId>,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    primary: bool,
    progress: Option<f32>,
    muted: bool,
) -> Response {
    let font = FontId::proportional(text_size);
    let look = crate::metrics::active();
    let lead = if icon.is_some() {
        Dp::D44.px()
    } else {
        Dp::D16.px()
    };
    let trail = if icon.is_some() {
        Dp::D24.px()
    } else {
        Dp::D16.px()
    };
    let chrome = lead + trail;
    let label = match width {
        Some(limit) => truncate_to_width(ui.painter(), label, &font, limit - chrome),
        None => label.to_owned(),
    };
    let label = label.as_str();
    let measure = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font.clone(), Color32::BLACK)
            .size()
            .x
            + chrome
    };
    let natural = measure(label);
    let width = width.unwrap_or(natural).max(natural);
    let inset = (width - natural) * 0.5;
    let lift = if progress.is_some() { Dp::D4.px() } else { 0.0 };
    let icon_node = icon.map(|_| {
        flex::stack(
            Align2::LEFT_CENTER,
            [flex::leaf(Vec2::splat(Dp::D16.px())).id(0)],
        )
        .padding(Insets {
            left: Dp::D20.px(),
            ..Insets::default()
        })
        .width(lead)
    });
    let label_size = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), Color32::BLACK)
        .size();
    let tree = flex::row(
        icon_node
            .into_iter()
            .chain(icon.is_none().then(|| flex::slot(lead)))
            .chain([flex::leaf(label_size).id(1)]),
    )
    .align(Align::Center)
    .padding(Insets {
        left: inset,
        bottom: lift * 2.0,
        ..Insets::default()
    });
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let painter = ui.painter();
    let style = crate::stylesheet::resolve(
        ui.ctx(),
        if primary {
            "pill.primary"
        } else {
            "pill.secondary"
        },
        response.id,
        crate::stylesheet::State {
            selected: false,
            hovered: response.hovered(),
            pressed: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
            enabled: !muted,
        },
        None,
    );
    let radius = style.radius.of(height);
    let placed = tree.solve(rect);
    crate::motion::press_scale(painter, rect, || {
        let (fill, ink) = (style.fill, style.text);
        if primary && !muted {
            painter.rect_filled(rect, radius, fill);
        } else {
            glass(painter, rect, radius, fill);
            painter.rect_stroke(
                rect,
                radius,
                crate::gfx::Stroke::new(stroke::hairline(), style.border),
                crate::gfx::StrokeKind::Inside,
            );
        }
        if let (Some(icon), Some(slot)) = (icon, placed.find(0)) {
            painter.image(icon, slot, full_uv(), ink);
        }
        let text_slot = placed.find(1).unwrap_or(rect);
        draw_text(
            painter,
            text_slot.left_center(),
            Align2::LEFT_CENTER,
            label,
            font,
            ink,
            None,
        );
        if let Some(progress) = progress {
            let band = Rect::from_min_max(
                Pos2::new(rect.left(), rect.center().y + text_size * 0.5),
                rect.right_bottom(),
            );
            let bar = flex::column([
                flex::leaf(Vec2::new(0.0, Dp::D2.px())),
                flex::leaf(Vec2::new(0.0, Dp::D3.px())).id(2),
            ])
            .align(Align::Stretch)
            .padding(Insets {
                left: lead,
                right: Dp::D24.px(),
                ..Insets::default()
            })
            .solve(band);
            let track = bar.find(2).unwrap_or(band);
            painter.rect_filled(track, radius::BAR, ink.gamma_multiply(0.16));
            painter.rect_filled(
                flex::fraction(track, progress),
                look.control_pill_progress_radius,
                ink,
            );
        }
    });
    response
}

pub(crate) fn progress_pill(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    text_size: f32,
    fraction: f32,
) -> Response {
    let response = ui.interact(
        rect,
        Id::new(("fluxa-progress-pill", label)),
        Sense::click(),
    );
    let painter = ui.painter();
    let radius = rect.height() * 0.5;
    let font = FontId::proportional(text_size);
    crate::motion::press_scale(painter, rect, || {
        let base = if response.hovered() { 46 } else { 28 };
        painter.rect_filled(rect, radius, shade(Shade::Scrim));
        glass(painter, rect, radius, ink(base));
        draw_text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            text_primary(),
            None,
        );
        let filled = rect.width() * fraction.clamp(0.0, 1.0);
        if filled > 0.5 {
            let covered = painter.with_clip_rect(flex::fraction(rect, fraction));
            covered.rect_filled(rect, radius, text_primary());
            covered.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                font,
                background(),
            );
        }
    });
    response
}
