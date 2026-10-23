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

pub fn begin_focus_frame(context: &egui::Context, focused: Option<u64>) {
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

pub fn take_focus_targets(context: &egui::Context) -> (Vec<(u64, Rect)>, Option<u64>) {
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

pub fn take_focus_neighbors(context: &egui::Context) -> Vec<(u64, Dir, u64)> {
    context.data_mut(|data| {
        let mut frame = data.get_temp::<FocusFrame>(frame_id()).unwrap_or_default();
        let neighbors = std::mem::take(&mut frame.neighbors);
        data.insert_temp(frame_id(), frame);
        neighbors
    })
}

pub(crate) trait FocusNode {
    fn focus_node(self, ui: &egui::Ui, node: u64, metrics: &UiMetrics) -> Self;
    fn focus_circle(self, ui: &egui::Ui, node: u64, circle: Rect, metrics: &UiMetrics) -> Self;
    fn focus_neighbors(self, ui: &egui::Ui, node: u64, neighbors: Neighbors) -> Self;
}

impl FocusNode for egui::Response {
    fn focus_node(self, ui: &egui::Ui, node: u64, metrics: &UiMetrics) -> Self {
        if register(ui, node, self.rect, self.clicked()) {
            paint_focus_ring(ui.painter(), self.rect, metrics);
        }
        self
    }

    fn focus_circle(self, ui: &egui::Ui, node: u64, circle: Rect, metrics: &UiMetrics) -> Self {
        if register(ui, node, circle, self.clicked()) {
            paint_focus_circle(ui.painter(), circle, metrics);
        }
        self
    }

    fn focus_neighbors(self, ui: &egui::Ui, node: u64, neighbors: Neighbors) -> Self {
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

fn register(ui: &egui::Ui, node: u64, rect: Rect, clicked: bool) -> bool {
    ui.ctx().data_mut(|data| {
        let mut frame = data.get_temp::<FocusFrame>(frame_id()).unwrap_or_default();
        frame.targets.push((node, rect));
        if clicked {
            frame.activated = Some(node);
        }
        let focused = frame.focused == Some(node);
        data.insert_temp(frame_id(), frame);
        focused
    })
}

pub(crate) fn paint_focus_circle(painter: &egui::Painter, rect: Rect, metrics: &UiMetrics) {
    painter.circle_stroke(
        rect.center(),
        rect.width().min(rect.height()) * 0.5
            + metrics.focus_ring_expand
            + metrics.focus_ring_width * 0.5,
        egui::Stroke::new(metrics.focus_ring_width, metrics.focus),
    );
}

pub(crate) fn paint_focus_ring(painter: &egui::Painter, rect: Rect, metrics: &UiMetrics) {
    painter.rect_stroke(
        rect.expand(metrics.focus_ring_expand),
        rect.height().min(metrics.focus_ring_radius * 4.0) * 0.5 + metrics.focus_ring_expand,
        egui::Stroke::new(metrics.focus_ring_width, metrics.focus),
        egui::StrokeKind::Outside,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Gesture {
    pub long_press: bool,
    pub drag_started: bool,
    pub drag_delta: Vec2,
    pub drag_released: bool,
}

pub(crate) fn gesture(response: &egui::Response) -> Gesture {
    Gesture {
        long_press: response.long_touched(),
        drag_started: response.drag_started(),
        drag_delta: if response.dragged() {
            response.drag_delta()
        } else {
            Vec2::ZERO
        },
        drag_released: response.drag_stopped(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pinch {
    pub scale: f32,
    pub center: Pos2,
    pub touches: usize,
}

pub fn pinch(context: &egui::Context) -> Option<Pinch> {
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

    fn touch(id: u64, phase: egui::TouchPhase, pos: Pos2) -> egui::Event {
        egui::Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId(id),
            phase,
            pos,
            force: None,
        }
    }

    fn run(context: &egui::Context, tick: usize, events: Vec<egui::Event>) -> Option<Pinch> {
        let input = egui::RawInput {
            time: Some(tick as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
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
        let context = egui::Context::default();
        begin_focus_frame(&context, None);
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 100.0))),
            ..Default::default()
        };
        context
            .run_ui(input, |root| {
                egui::CentralPanel::default().show(root, |ui| {
                    let response = ui.allocate_response(Vec2::splat(20.0), egui::Sense::click());
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
        let context = egui::Context::default();
        run(
            &context,
            0,
            vec![
                egui::Event::PointerMoved(Pos2::new(200.0, 150.0)),
                touch(1, egui::TouchPhase::Start, Pos2::new(180.0, 150.0)),
                touch(2, egui::TouchPhase::Start, Pos2::new(220.0, 150.0)),
            ],
        );
        run(
            &context,
            1,
            vec![
                touch(1, egui::TouchPhase::Move, Pos2::new(170.0, 150.0)),
                touch(2, egui::TouchPhase::Move, Pos2::new(230.0, 150.0)),
            ],
        );
        let seen = run(
            &context,
            2,
            vec![
                touch(1, egui::TouchPhase::Move, Pos2::new(150.0, 150.0)),
                touch(2, egui::TouchPhase::Move, Pos2::new(250.0, 150.0)),
            ],
        )
        .expect("two touches are a pinch");
        assert_eq!(seen.touches, 2);
        assert!(seen.scale > 1.2);
    }

    #[test]
    fn bound_components_register_targets_and_activation() {
        let context = egui::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let metrics = UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Tv));
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            ..Default::default()
        };
        let mut targets = Vec::new();
        context
            .run_ui(input, |root| {
                begin_focus_frame(root.ctx(), Some(7));
                egui::CentralPanel::default().show(root, |ui| {
                    crate::components::chip(ui, "One", false, metrics).focus_node(ui, 7, &metrics);
                    crate::components::chip(ui, "Two", false, metrics).focus_node(ui, 8, &metrics);
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
