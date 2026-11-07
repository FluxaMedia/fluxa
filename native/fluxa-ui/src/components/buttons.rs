use super::*;

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
        metrics.nav_label_size + 2.0,
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
            egui::Stroke::new(stroke::hairline(), color),
            egui::StrokeKind::Inside,
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
    context: &egui::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .show(context, add)
}

pub(crate) fn placed_front<R>(
    context: &egui::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
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
        let (expand, width) =
            metrics.map_or((3.0, 2.0), |m| (m.focus_ring_expand, m.focus_ring_width));
        painter.rect_stroke(
            rect.expand(expand),
            height * 0.5 + expand,
            egui::Stroke::new(width, accent()),
            egui::StrokeKind::Outside,
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
    let width = match (&unavailable, width) {
        (Some(reason), Some(limit)) => Some(limit.max(
            crate::paint::text_width(ui.painter(), reason, FontId::proportional(text_size)) + 70.0,
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
    icon: Option<egui::TextureId>,
    arrow: Option<egui::TextureId>,
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
            egui::Stroke::new(stroke::hairline(), ink(34)),
            egui::StrokeKind::Inside,
        );
    }
    if response.hovered() {
        painter.rect_filled(
            rect,
            if framed { metrics.card_radius } else { 0.0 },
            Color32::from_white_alpha(10),
        );
    }
    let badge =
        Rect::from_center_size(rect.left_center() + Vec2::new(38.0, 0.0), Vec2::splat(44.0));
    painter.rect_filled(badge, 14.0, Color32::from_white_alpha(12));
    if let Some(icon) = icon {
        painter.image(
            icon,
            Rect::from_center_size(badge.center(), Vec2::splat(24.0)),
            full_uv(),
            Color32::WHITE,
        );
    }
    let text_x = badge.right() + 12.0;
    let gutter = 14.0 + 16.0 + 8.0;
    let text_width = (rect.right() - gutter - text_x).max(0.0);
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
    let gap = 4.0;
    let top = rect.center().y - (title_galley.size().y + gap + subtitle_galley.size().y) * 0.5;
    let subtitle_top = top + title_galley.size().y + gap;
    painter.galley(Pos2::new(text_x, top), title_galley, text_primary());
    painter.galley(
        Pos2::new(text_x, subtitle_top),
        subtitle_galley,
        metrics.text_secondary,
    );
    if let Some(arrow) = arrow {
        painter.image(
            arrow,
            Rect::from_center_size(
                rect.right_center() - Vec2::new(14.0 + 8.0, 0.0),
                Vec2::splat(16.0),
            ),
            full_uv(),
            ink(110),
        );
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
    let line = egui::Stroke::new(1.6, text_primary().gamma_multiply(alpha as f32 / 255.0));
    let arm = size * 0.2;
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
    icon: Option<egui::TextureId>,
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
    icon: Option<egui::TextureId>,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    primary: bool,
    progress: Option<f32>,
    muted: bool,
) -> Response {
    let font = FontId::proportional(text_size);
    let lead = if icon.is_some() { 44.0 } else { 16.0 };
    let trail = if icon.is_some() { 26.0 } else { 16.0 };
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
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let painter = ui.painter();
    let look = crate::metrics::active();
    let radius = height * look.pill_ratio;
    crate::motion::press_scale(painter, rect, || {
        let (fill, ink) = if muted {
            (ink(tone::soft()), ink(tone::muted()))
        } else {
            match (primary, response.hovered()) {
                (true, false) => (look.button_primary, look.button_primary_text),
                (true, true) => (ink(tone::body()), look.button_primary_text),
                (false, false) => (ink(tone::strong()), text_primary()),
                (false, true) => (ink(tone::dim()), text_primary()),
            }
        };
        if primary && !muted {
            painter.rect_filled(rect, radius, fill);
        } else {
            glass(painter, rect, radius, fill);
            painter.rect_stroke(
                rect,
                radius,
                egui::Stroke::new(
                    stroke::hairline(),
                    Color32::from_white_alpha(look.button_border),
                ),
                egui::StrokeKind::Inside,
            );
        }
        let lift = if progress.is_some() { 4.0 } else { 0.0 };
        if let Some(icon) = icon {
            painter.image(
                icon,
                Rect::from_center_size(
                    rect.left_center() + Vec2::new(inset + 28.0, -lift),
                    Vec2::splat(18.0),
                ),
                full_uv(),
                ink,
            );
        }
        draw_text(
            painter,
            rect.left_center() + Vec2::new(inset + lead, -lift),
            Align2::LEFT_CENTER,
            label,
            font,
            ink,
            None,
        );
        if let Some(progress) = progress {
            let track = Rect::from_min_max(
                Pos2::new(rect.left() + lead, rect.center().y + text_size * 0.5 + 2.0),
                Pos2::new(rect.right() - 24.0, rect.center().y + text_size * 0.5 + 5.0),
            );
            painter.rect_filled(track, radius::BAR, ink.gamma_multiply(0.16));
            painter.rect_filled(
                Rect::from_min_size(
                    track.min,
                    Vec2::new(track.width() * progress.clamp(0.0, 1.0), track.height()),
                ),
                1.5,
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
            let covered = painter.with_clip_rect(Rect::from_min_size(
                rect.min,
                Vec2::new(filled, rect.height()),
            ));
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
