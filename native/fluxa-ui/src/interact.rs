use super::*;

#[derive(Clone, Default)]
struct FocusFrame {
    focused: Option<u64>,
    targets: Vec<(u64, Rect)>,
    activated: Option<u64>,
    neighbors: Vec<(u64, Dir, u64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Neighbors {
    pub up: Option<u64>,
    pub down: Option<u64>,
    pub left: Option<u64>,
    pub right: Option<u64>,
}

fn frame_id() -> Id {
    Id::new("fluxa-focus-frame")
}

pub fn begin_focus_frame(context: &crate::gfx::Context, focused: Option<u64>) {
    context.data_mut(|data| {
        data.insert_temp(
            frame_id(),
            FocusFrame {
                focused,
                ..Default::default()
            },
        )
    });
}

pub fn take_focus_targets(context: &crate::gfx::Context) -> (Vec<(u64, Rect)>, Option<u64>) {
    context.data_mut(|data| {
        let frame = data.get_temp::<FocusFrame>(frame_id()).unwrap_or_default();
        data.insert_temp(
            frame_id(),
            FocusFrame {
                focused: frame.focused,
                neighbors: frame.neighbors.clone(),
                ..Default::default()
            },
        );
        (frame.targets, frame.activated)
    })
}

pub fn take_focus_neighbors(context: &crate::gfx::Context) -> Vec<(u64, Dir, u64)> {
    context.data_mut(|data| {
        let mut frame = data.get_temp::<FocusFrame>(frame_id()).unwrap_or_default();
        let neighbors = std::mem::take(&mut frame.neighbors);
        data.insert_temp(frame_id(), frame);
        neighbors
    })
}

pub(crate) trait FocusNode {
    fn focus_node(self, ui: &crate::gfx::Ui, node: u64) -> Self;
    fn focus_circle(self, ui: &crate::gfx::Ui, node: u64, circle: Rect) -> Self;
    fn focus_neighbors(self, ui: &crate::gfx::Ui, node: u64, neighbors: Neighbors) -> Self;
}

impl FocusNode for crate::gfx::Response {
    fn focus_node(self, ui: &crate::gfx::Ui, node: u64) -> Self {
        register(ui, node, self.rect, self.clicked());
        self
    }

    fn focus_circle(self, ui: &crate::gfx::Ui, node: u64, circle: Rect) -> Self {
        register(ui, node, circle, self.clicked());
        self
    }

    fn focus_neighbors(self, ui: &crate::gfx::Ui, node: u64, neighbors: Neighbors) -> Self {
        ui.ctx().data_mut(|data| {
            let mut frame = data.get_temp::<FocusFrame>(frame_id()).unwrap_or_default();
            for (dir, target) in [
                (Dir::Up, neighbors.up),
                (Dir::Down, neighbors.down),
                (Dir::Left, neighbors.left),
                (Dir::Right, neighbors.right),
            ] {
                if let Some(target) = target {
                    frame.neighbors.push((node, dir, target));
                }
            }
            data.insert_temp(frame_id(), frame);
        });
        self
    }
}

fn register(ui: &crate::gfx::Ui, node: u64, rect: Rect, clicked: bool) {
    let rect = ui.ctx().layer_rect_to_global(ui.layer_id(), rect);
    ui.ctx().data_mut(|data| {
        let mut frame = data.get_temp::<FocusFrame>(frame_id()).unwrap_or_default();
        frame.targets.push((node, rect));
        if clicked {
            frame.activated = Some(node);
        }
        data.insert_temp(frame_id(), frame);
    });
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pinch {
    pub scale: f32,
    pub center: Pos2,
    pub touches: usize,
}

pub fn pinch(context: &crate::gfx::Context) -> Option<Pinch> {
    let touch = context.multi_touch()?;
    Some(Pinch {
        scale: touch.zoom_delta,
        center: touch.center_pos,
        touches: touch.num_touches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(id: u64, phase: crate::gfx::TouchPhase, pos: Pos2) -> crate::gfx::Event {
        crate::gfx::Event::Touch {
            device_id: crate::gfx::TouchDeviceId(0),
            id: crate::gfx::TouchId(id),
            phase,
            pos,
            force: None,
        }
    }

    fn run(
        context: &crate::gfx::Context,
        tick: usize,
        events: Vec<crate::gfx::Event>,
    ) -> Option<Pinch> {
        let input = crate::gfx::RawInput {
            time: Some(tick as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            screen_rect: Some(crate::gfx::rect_at(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            events,
            ..Default::default()
        };
        let mut seen = None;
        context
            .run_pass(input, |context| seen = pinch(context))
            .drop_without_applying_deltas();
        seen
    }

    #[test]
    fn neighbors_declared_by_a_component_are_collected() {
        let context = crate::gfx::Context::default();
        begin_focus_frame(&context, None);
        let input = crate::gfx::RawInput {
            screen_rect: Some(crate::gfx::rect_at(Pos2::ZERO, Vec2::new(200.0, 100.0))),
            ..Default::default()
        };
        context
            .run_ui(input, |root| {
                crate::gfx::CentralPanel::default().show(root, |ui| {
                    let response =
                        ui.allocate_response(Vec2::splat(20.0), crate::gfx::Sense::click());
                    response.focus_neighbors(
                        ui,
                        5,
                        Neighbors {
                            right: Some(9),
                            ..Default::default()
                        },
                    );
                });
            })
            .drop_without_applying_deltas();
        assert_eq!(take_focus_neighbors(&context), vec![(5, Dir::Right, 9)]);
    }

    #[test]
    fn two_fingers_spreading_report_a_zoom_above_one() {
        let context = crate::gfx::Context::default();
        run(
            &context,
            0,
            vec![
                crate::gfx::Event::PointerMoved(Pos2::new(200.0, 150.0)),
                touch(1, crate::gfx::TouchPhase::Start, Pos2::new(180.0, 150.0)),
                touch(2, crate::gfx::TouchPhase::Start, Pos2::new(220.0, 150.0)),
            ],
        );
        run(
            &context,
            1,
            vec![
                touch(1, crate::gfx::TouchPhase::Move, Pos2::new(170.0, 150.0)),
                touch(2, crate::gfx::TouchPhase::Move, Pos2::new(230.0, 150.0)),
            ],
        );
        let seen = run(
            &context,
            2,
            vec![
                touch(1, crate::gfx::TouchPhase::Move, Pos2::new(150.0, 150.0)),
                touch(2, crate::gfx::TouchPhase::Move, Pos2::new(250.0, 150.0)),
            ],
        )
        .expect("two touches are a pinch");
        assert_eq!(seen.touches, 2);
        assert!(seen.scale > 1.2);
    }

    #[test]
    fn bound_components_register_targets_and_activation() {
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let metrics = UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Tv));
        crate::kit::provide(&context, metrics);
        let input = crate::gfx::RawInput {
            screen_rect: Some(crate::gfx::rect_at(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            ..Default::default()
        };
        let mut targets = Vec::new();
        context
            .run_ui(input, |root| {
                begin_focus_frame(root.ctx(), Some(7));
                crate::gfx::CentralPanel::default().show(root, |ui| {
                    crate::components::chip(ui, "One", false, metrics).focus_node(ui, 7);
                    crate::components::chip(ui, "Two", false, metrics).focus_node(ui, 8);
                });
                targets = take_focus_targets(root.ctx()).0;
            })
            .drop_without_applying_deltas();
        assert_eq!(
            targets.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            [7, 8]
        );
        assert!(targets[0].1.bottom() <= targets[1].1.top());
    }
}
