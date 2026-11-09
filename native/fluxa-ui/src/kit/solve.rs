use egui::{Pos2, Rect, Vec2};
use taffy::geometry::Rect as Edges;
use taffy::prelude::{
    AlignItems, AvailableSpace, Display, FlexDirection, NodeId, Size, Style, TaffyTree, auto,
    length, zero,
};

#[derive(Clone, Copy)]
pub(crate) enum Flow {
    Row,
    Column,
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
    let mut tree: TaffyTree<()> = TaffyTree::new();
    let children: Vec<NodeId> = slots
        .iter()
        .map(|slot| {
            let margin = match flow {
                Flow::Row => Edges {
                    left: length(slot.lead),
                    right: zero(),
                    top: zero(),
                    bottom: zero(),
                },
                Flow::Column => Edges {
                    left: zero(),
                    right: zero(),
                    top: length(slot.lead),
                    bottom: zero(),
                },
            };
            tree.new_leaf(Style {
                size: Size {
                    width: slot.width.map_or(auto(), length),
                    height: slot.height.map_or(auto(), length),
                },
                margin,
                flex_grow: if slot.grow { 1.0 } else { 0.0 },
                flex_shrink: 0.0,
                ..Style::default()
            })
            .expect("leaf")
        })
        .collect();
    let root = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: match flow {
                    Flow::Row => FlexDirection::Row,
                    Flow::Column => FlexDirection::Column,
                },
                align_items: AlignItems::START,
                gap: Size {
                    width: length(gap),
                    height: length(gap),
                },
                size: Size {
                    width: length(available.x),
                    height: length(available.y),
                },
                ..Style::default()
            },
            &children,
        )
        .expect("root");
    tree.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(available.x),
            height: AvailableSpace::Definite(available.y),
        },
    )
    .expect("layout");
    children
        .iter()
        .map(|child| {
            let layout = tree.layout(*child).expect("child layout");
            Rect::from_min_size(
                origin + Vec2::new(layout.location.x, layout.location.y),
                Vec2::new(layout.size.width, layout.size.height),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
