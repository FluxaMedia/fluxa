use crate::flex::{self, Insets, Node};
use crate::gfx::{Pos2, Rect, Vec2};

#[derive(Clone, Copy)]
pub(crate) enum Flow {
    Row,
    Column,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Justify {
    #[default]
    Start,
    Center,
    End,
    Between,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Slot {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub lead: f32,
    pub grow: bool,
}

impl Slot {
    pub fn sized(width: f32, height: f32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            ..Self::default()
        }
    }

    pub fn lead(mut self, lead: f32) -> Self {
        self.lead = lead;
        self
    }

    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }
}

pub(crate) fn solve(
    flow: Flow,
    gap: f32,
    origin: Pos2,
    available: Vec2,
    slots: &[Slot],
) -> Vec<Rect> {
    solve_with(flow, gap, Justify::Start, false, origin, available, slots)
}

pub(crate) fn solve_with(
    flow: Flow,
    gap: f32,
    justify: Justify,
    center: bool,
    origin: Pos2,
    available: Vec2,
    slots: &[Slot],
) -> Vec<Rect> {
    if let Some(rects) = packed_row(flow, gap, justify, center, origin, available, slots) {
        return rects;
    }
    solve_tree(flow, gap, justify, center, origin, available, slots)
}

fn packed_row(
    flow: Flow,
    gap: f32,
    justify: Justify,
    center: bool,
    origin: Pos2,
    available: Vec2,
    slots: &[Slot],
) -> Option<Vec<Rect>> {
    if !matches!(flow, Flow::Row) || justify != Justify::Start || center {
        return None;
    }
    let mut x = origin.x;
    let mut rects = Vec::with_capacity(slots.len());
    for (index, slot) in slots.iter().enumerate() {
        let (width, height) = (slot.width?, slot.height?);
        if slot.grow || height > available.y {
            return None;
        }
        if index > 0 {
            x += gap;
        }
        x += slot.lead;
        rects.push(Rect::from_min_size(
            Pos2::new(x, origin.y),
            Vec2::new(width, height),
        ));
        x += width;
    }
    (x - origin.x <= available.x).then_some(rects)
}

fn solve_tree(
    flow: Flow,
    gap: f32,
    justify: Justify,
    center: bool,
    origin: Pos2,
    available: Vec2,
    slots: &[Slot],
) -> Vec<Rect> {
    let children = slots.iter().enumerate().map(|(index, slot)| {
        let mut leaf = flex::leaf(Vec2::new(
            slot.width.unwrap_or(0.0),
            slot.height.unwrap_or(0.0),
        ));
        if let Some(width) = slot.width {
            leaf = leaf.width(width);
        }
        if let Some(height) = slot.height {
            leaf = leaf.height(height);
        }
        if slot.grow {
            leaf = leaf.grow(1.0);
        }
        let lead = match flow {
            Flow::Row => Insets {
                left: slot.lead,
                ..Insets::default()
            },
            Flow::Column => Insets {
                top: slot.lead,
                ..Insets::default()
            },
        };
        let wrapped: Node = match flow {
            Flow::Row => flex::row([leaf.id(index)]).align(crate::flex::Align::Start),
            Flow::Column => flex::column([leaf.id(index)]).align(crate::flex::Align::Start),
        };
        let wrapped = wrapped.padding(lead);
        if slot.grow {
            wrapped.grow(1.0)
        } else {
            wrapped
        }
    });
    let tree: Node = match flow {
        Flow::Row => flex::row(children).align(crate::flex::Align::Start),
        Flow::Column => flex::column(children).align(crate::flex::Align::Start),
    }
    .gap(gap)
    .justify(match justify {
        Justify::Start => flex::Justify::Start,
        Justify::Center => flex::Justify::Center,
        Justify::End => flex::Justify::End,
        Justify::Between => flex::Justify::SpaceBetween,
    })
    .align(if center {
        flex::Align::Center
    } else {
        flex::Align::Start
    });
    let placed = tree
        .solve(Rect::from_min_size(origin, available))
        .rects_by_id();
    (0..slots.len())
        .map(|index| {
            placed
                .get(&index)
                .copied()
                .unwrap_or(Rect::from_min_size(origin, Vec2::ZERO))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_of_slots_keep_their_order() {
        let slots = vec![Slot::sized(10.0, 20.0); 3000];
        let rects = solve(Flow::Row, 2.0, Pos2::ZERO, Vec2::new(1.0e7, 100.0), &slots);
        assert_eq!(rects.len(), 3000);
        assert_eq!(rects[2999].min.x, 2999.0 * 12.0);
        assert!(rects.windows(2).all(|pair| pair[1].min.x > pair[0].min.x));
    }

    #[test]
    fn packed_rows_match_the_flex_tree() {
        let slots: Vec<Slot> = (0..40)
            .map(|index| {
                let slot = Slot::sized(30.0 + (index % 7) as f32 * 11.5, 20.0 + (index % 3) as f32);
                if index % 5 == 0 {
                    slot.lead(6.25)
                } else {
                    slot
                }
            })
            .collect();
        let origin = Pos2::new(13.5, -4.0);
        let available = Vec2::new(1.0e7, 1.0e4);
        let packed = packed_row(
            Flow::Row,
            9.0,
            Justify::Start,
            false,
            origin,
            available,
            &slots,
        )
        .expect("plain sized rows take the packed path");
        let tree = solve_tree(
            Flow::Row,
            9.0,
            Justify::Start,
            false,
            origin,
            available,
            &slots,
        );
        for (packed, tree) in packed.iter().zip(&tree) {
            assert!((packed.min - tree.min).length() < 0.01);
            assert!((packed.size() - tree.size()).length() < 0.01);
        }
    }

    #[test]
    fn rows_wider_than_the_space_fall_back_to_the_tree() {
        let slots = vec![Slot::sized(80.0, 10.0); 5];
        assert!(
            packed_row(
                Flow::Row,
                0.0,
                Justify::Start,
                false,
                Pos2::ZERO,
                Vec2::new(200.0, 50.0),
                &slots
            )
            .is_none()
        );
    }

    #[test]
    fn row_places_slots_with_gap_and_lead() {
        let rects = solve(
            Flow::Row,
            10.0,
            Pos2::new(5.0, 7.0),
            Vec2::new(1000.0, 100.0),
            &[Slot::sized(40.0, 20.0), Slot::sized(30.0, 20.0).lead(8.0)],
        );
        assert_eq!(rects[0].min, Pos2::new(5.0, 7.0));
        assert!((rects[1].min.x - (5.0 + 40.0 + 10.0 + 8.0)).abs() < 0.01);
        assert!((rects[1].width() - 30.0).abs() < 0.01);
    }

    #[test]
    fn growing_slot_takes_the_remaining_width() {
        let rects = solve(
            Flow::Row,
            0.0,
            Pos2::ZERO,
            Vec2::new(200.0, 30.0),
            &[Slot::sized(50.0, 30.0), Slot::default().grow()],
        );
        assert!((rects[1].width() - 150.0).abs() < 0.01);
    }
}
