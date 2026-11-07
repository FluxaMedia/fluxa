use super::*;

pub(crate) struct StickyAction {
    pub node: u64,
    pub icon: Option<egui::TextureId>,
    pub active: bool,
    pub hint: String,
}

pub(crate) struct StickyBar<'a> {
    pub back_node: u64,
    pub back_icon: Option<egui::TextureId>,
    pub logo: Option<(egui::TextureId, Vec2)>,
    pub title: &'a str,
    pub actions: Vec<StickyAction>,
}

pub(crate) struct StickyOutput {
    pub clicked: Option<u64>,
    pub rects: Vec<(u64, Rect)>,
}

pub(crate) fn sticky_bar(
    context: &egui::Context,
    id: &'static str,
    viewport: Viewport,
    margin: f32,
    shown: bool,
    bar: StickyBar,
    metrics: UiMetrics,
) -> Option<StickyOutput> {
    let fade = crate::motion::toggle(
        context,
        Id::new((id, "sticky-fade")),
        shown,
        crate::motion::fast(),
    );
    if fade <= 0.0 {
        return None;
    }
    let top = if viewport.is_compact() {
        metrics.content_header_top_mobile - 8.0
    } else {
        metrics.detail_back_top
    };
    let height = top + 48.0;
    egui::Area::new(Id::new((id, "sticky")))
        .constrain(false)
        .fixed_pos(Pos2::ZERO)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            ui.set_opacity(fade);
            let strip = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, height));
            ui.interact(
                strip,
                Id::new((id, "sticky-block")),
                Sense::click_and_drag(),
            );
            ui.painter().rect_filled(strip, 0.0, metrics.background);
            ui.painter().line_segment(
                [strip.left_bottom(), strip.right_bottom()],
                egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("track")),
            );
            let mut output = StickyOutput {
                clicked: None,
                rects: Vec::new(),
            };
            let button = metrics.icon_button_size;
            let line = Rect::from_min_max(
                Pos2::new(margin, top + 4.0),
                Pos2::new(viewport.width - margin, top + 4.0 + button),
            );
            let count = bar.actions.len();
            let trailing = button * count as f32 + 8.0 * count.saturating_sub(1) as f32;
            let mut cells = vec![crate::flex::slot(button).id(0), crate::flex::fill().id(1)];
            if count > 0 {
                cells.push(crate::flex::slot(trailing).id(2));
            }
            let placed = crate::flex::row(cells)
                .gap(12.0)
                .align(crate::flex::Align::Stretch)
                .solve(line);
            let back_rect = placed.find(0).unwrap_or(line);
            let title = placed.find(1).unwrap_or(line);
            let actions = placed.find(2).unwrap_or(Rect::NOTHING);
            let back = ui
                .scope_builder(egui::UiBuilder::new().max_rect(back_rect), |ui| {
                    back_button(ui, bar.back_icon, metrics)
                })
                .inner;
            output.rects.push((bar.back_node, back_rect));
            if back.clicked() {
                output.clicked = Some(bar.back_node);
            }
            let slots =
                crate::flex::row((0..count).map(|index| crate::flex::slot(button).id(index)))
                    .gap(8.0)
                    .align(crate::flex::Align::Stretch)
                    .solve(actions)
                    .children
                    .into_iter()
                    .map(|slot| slot.rect)
                    .collect::<Vec<_>>();
            for (action, rect) in bar.actions.iter().zip(slots) {
                let response = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        icon_button(
                            ui,
                            action.icon,
                            button,
                            text_primary(),
                            true,
                            action.active,
                            true,
                        )
                    })
                    .inner
                    .on_hover_text(&action.hint);
                output.rects.push((action.node, rect));
                if response.clicked() {
                    output.clicked = Some(action.node);
                }
            }
            let left = title.left();
            let room = title.width();
            let center = top + 24.0;
            match bar.logo {
                Some((texture, size)) => {
                    let scale = (room / size.x).min(1.0);
                    let size = size * scale;
                    ui.painter().image(
                        texture,
                        Rect::from_min_size(Pos2::new(left, center - size.y * 0.5), size),
                        full_uv(),
                        Color32::WHITE,
                    );
                }
                None => {
                    let font = crate::fonts::bold(metrics.text.subtitle);
                    draw_text(
                        ui.painter(),
                        Pos2::new(left, center),
                        Align2::LEFT_CENTER,
                        bar.title,
                        font,
                        text_primary(),
                        Some(room),
                    );
                }
            }
            output
        })
        .inner
        .into()
}
