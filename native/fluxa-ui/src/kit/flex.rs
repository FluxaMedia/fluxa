use super::*;
use crate::gfx::{Sense, Vec2};

type Child<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

pub(crate) struct Flex<'a> {
    flow: Flow,
    gap: Option<Space>,
    justify: Justify,
    center: bool,
    salt: Id,
    children: Vec<(bool, Child<'a>)>,
}

pub(crate) fn flex<'a>(salt: impl std::hash::Hash + std::fmt::Debug) -> Flex<'a> {
    Flex {
        flow: Flow::Row,
        gap: None,
        justify: Justify::Start,
        center: false,
        salt: Id::new(salt),
        children: Vec::new(),
    }
}

impl<'a> Flex<'a> {
    pub fn column(mut self) -> Self {
        self.flow = Flow::Column;
        self
    }

    pub fn gap(mut self, space: Space) -> Self {
        self.gap = Some(space);
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn center(mut self) -> Self {
        self.center = true;
        self
    }

    pub fn child(mut self, add: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.children.push((false, Box::new(add)));
        self
    }

    pub fn grow(mut self, add: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.children.push((true, Box::new(add)));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Vec<Rect> {
        let tokens = ui.tokens();
        let gap = self.gap.map_or(0.0, |space| space.px(&tokens));
        let id = ui.make_persistent_id(self.salt);
        let known: Vec<Vec2> = ui.data(|data| data.get_temp(id)).unwrap_or_default();
        let width = ui.available_width();
        let room = ui.available_height();
        let slots: Vec<Slot> = self
            .children
            .iter()
            .enumerate()
            .map(|(index, (grow, _))| {
                let size = known.get(index).copied().unwrap_or(Vec2::ZERO);
                let slot = Slot {
                    width: Some(size.x),
                    height: Some(size.y),
                    ..Slot::default()
                };
                match (self.flow, grow) {
                    (Flow::Row, true) => Slot {
                        width: None,
                        ..slot
                    }
                    .grow(),
                    (Flow::Column, true) => Slot {
                        height: None,
                        ..slot
                    }
                    .grow(),
                    _ => slot,
                }
            })
            .collect();
        let tallest = known.iter().map(|size| size.y).fold(0.0, f32::max);
        let stacked: f32 = known.iter().map(|size| size.y).sum::<f32>()
            + gap * known.len().saturating_sub(1) as f32;
        let height = match self.flow {
            Flow::Row => tallest,
            Flow::Column => stacked,
        };
        let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let justify = if known.is_empty() {
            Justify::Start
        } else {
            self.justify
        };
        let place = || {
            solve_with(
                self.flow,
                gap,
                justify,
                self.center,
                area.min,
                area.size(),
                &slots,
            )
        };
        let rects = crate::flex::muted(place);
        let mut measured = Vec::with_capacity(rects.len());
        for ((_, add), rect) in self.children.into_iter().zip(&rects) {
            let mut child = ui.new_child(UiBuilder::new().max_rect(crate::gfx::rect_at(
                rect.min,
                Vec2::new(
                    if matches!(self.flow, Flow::Row) && rect.width() > 0.0 {
                        rect.width()
                    } else {
                        width
                    },
                    room,
                ),
            )));
            add(&mut child);
            measured.push(child.min_rect().size());
        }
        let changed = measured.len() != known.len()
            || measured
                .iter()
                .zip(&known)
                .any(|(now, before)| (*now - *before).abs().max_elem() > 0.5);
        if changed {
            ui.data_mut(|data| data.insert_temp(id, measured));
            ui.ctx().request_repaint();
        } else {
            place();
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn between_pushes_children_to_both_edges() {
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        provide(
            &context,
            UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop)),
        );
        let mut rects = Vec::new();
        for _ in 0..3 {
            let input = crate::gfx::RawInput {
                screen_rect: Some(crate::gfx::rect_at(Pos2::ZERO, Vec2::new(400.0, 300.0))),
                ..Default::default()
            };
            context
                .run_ui(input, |root| {
                    crate::gfx::CentralPanel::default()
                        .frame(crate::gfx::Frame::NONE)
                        .show(root, |ui| {
                            rects = flex("t")
                                .justify(Justify::Between)
                                .child(|ui| {
                                    ui.allocate_space(Vec2::new(50.0, 20.0));
                                })
                                .child(|ui| {
                                    ui.allocate_space(Vec2::new(50.0, 20.0));
                                })
                                .show(ui);
                        });
                })
                .drop_without_applying_deltas();
        }
        assert!(rects[0].left().abs() < 0.5, "{rects:?}");
        assert!((rects[1].right() - 400.0).abs() < 0.5, "{rects:?}");
    }
}
