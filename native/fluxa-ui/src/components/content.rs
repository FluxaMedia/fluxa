use super::*;

pub(crate) fn labeled_action(
    ui: &mut Ui,
    icon: Option<crate::gfx::TextureId>,
    label: &str,
    width: f32,
    text_size: f32,
    active: bool,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, 40.0 + text_size), Sense::click());
    let painter = ui.painter();
    let center = Pos2::new(rect.center().x, rect.top() + 18.0);
    if active {
        painter.circle_filled(center, 18.0, text_primary());
    } else if response.hovered() {
        painter.circle_filled(center, 18.0, crate::stylesheet::color("track"));
    }
    if let Some(icon) = icon {
        painter.image(
            icon,
            crate::gfx::square_around(center, 20.0),
            full_uv(),
            if active { background() } else { text_primary() },
        );
    }
    draw_text(
        painter,
        Pos2::new(rect.center().x, rect.bottom()),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(text_size),
        text_primary(),
        None,
    );
    response
}

pub(crate) fn wrapped_text(
    painter: &Painter,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<crate::gfx::Galley> {
    let mut job = crate::gfx::text::LayoutJob::simple(
        text.to_owned(),
        FontId::proportional(size),
        color,
        width,
    );
    job.wrap.max_rows = rows;
    painter.layout_job(job)
}

pub(crate) fn texture_image(
    painter: &Painter,
    texture: crate::gfx::TextureId,
    rect: Rect,
    uv: Rect,
    tint: Color32,
) {
    painter.image(texture, rect, uv, tint);
}

pub(crate) fn rounded_image(
    painter: &Painter,
    texture: crate::gfx::TextureId,
    rect: Rect,
    radius: impl Into<crate::gfx::CornerRadius>,
    uv: Rect,
    tint: Color32,
) {
    painter
        .add(crate::gfx::epaint::RectShape::filled(rect, radius, tint).with_texture(texture, uv));
}

pub(crate) fn empty_state(
    painter: &Painter,
    rect: Rect,
    icon: Option<crate::gfx::TextureId>,
    title: &str,
    description: &str,
    metrics: UiMetrics,
) {
    let center = rect.center();
    let icon_size = 36.0;
    let title_font = crate::fonts::bold(metrics.screen_section_title_size);
    let description_font = crate::fonts::regular(metrics.screen_body_size);
    let mut y = center.y - (icon_size + metrics.section_gap + title_font.size * 2.6) * 0.5;
    if let Some(icon) = icon {
        let icon_rect = crate::flex::centered(
            crate::gfx::rect_at(
                crate::gfx::Pos2::new(rect.left(), y),
                Vec2::new(rect.width(), icon_size),
            ),
            Vec2::splat(icon_size),
        );
        texture_image(painter, icon, icon_rect, full_uv(), metrics.text_muted);
        y += icon_size + metrics.section_gap;
    }
    let width = rect.width() - metrics.section_gap * 2.0;
    draw_text(
        painter,
        crate::gfx::Pos2::new(center.x, y),
        Align2::CENTER_TOP,
        title,
        title_font.clone(),
        metrics.text_primary,
        Some(width),
    );
    draw_text(
        painter,
        crate::gfx::Pos2::new(center.x, y + title_font.size * 1.5),
        Align2::CENTER_TOP,
        description,
        description_font,
        metrics.text_secondary,
        Some(width),
    );
}

pub(crate) fn text_tabs(
    ui: &mut Ui,
    labels: &[String],
    selected: usize,
    height: f32,
    metrics: UiMetrics,
) -> Vec<Response> {
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    let mut responses = Vec::with_capacity(labels.len());
    crate::kit::view()
        .row()
        .hug()
        .gap(crate::flex::Space::S24)
        .show(ui, |ui| {
            for (index, label) in labels.iter().enumerate() {
                let active = index == selected;
                let galley =
                    ui.painter()
                        .layout_no_wrap(label.clone(), font.clone(), Color32::WHITE);
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(galley.size().x, height), Sense::click());
                let color = if active {
                    accent()
                } else if response.hovered() {
                    metrics.ink(tone::body())
                } else {
                    metrics.ink(tone::muted())
                };
                ui.painter().galley(
                    crate::gfx::Pos2::new(rect.left(), rect.center().y - galley.size().y * 0.5),
                    galley,
                    color,
                );
                if active {
                    ui.painter().rect_filled(
                        Rect::from_min_max(
                            crate::gfx::Pos2::new(rect.left(), rect.bottom() - 2.0),
                            rect.right_bottom(),
                        ),
                        1.0,
                        accent(),
                    );
                }
                responses.push(response);
            }
        });
    responses
}
