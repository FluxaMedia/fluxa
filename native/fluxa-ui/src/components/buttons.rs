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
