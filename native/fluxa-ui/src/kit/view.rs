use super::*;
use crate::interact::FocusNode;
use egui::{Rect, Sense, Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Surface {
    Clear,
    Raised,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    Column,
    Row,
}

#[derive(Clone, Copy)]
enum Scroll {
    None,
    X(&'static str),
    Y(&'static str),
}

pub(crate) struct View {
    axis: Option<Axis>,
    gap: Option<Space>,
    padding: Option<Space>,
    surface: Surface,
    rounded: bool,
    scroll: Scroll,
    max_width: Option<f32>,
    center_x: bool,
    wrap: bool,
    clickable: bool,
    focus: Option<u64>,
    salt: Option<Id>,
    class: Option<&'static str>,
    modifiers: Vec<&'static str>,
}

pub(crate) struct Shown<R> {
    pub inner: R,
    pub response: Response,
}

pub(crate) fn view() -> View {
    View {
        axis: None,
        gap: None,
        padding: None,
        surface: Surface::Clear,
        rounded: false,
        scroll: Scroll::None,
        max_width: None,
        center_x: false,
        wrap: false,
        clickable: false,
        focus: None,
        salt: None,
        class: None,
        modifiers: Vec::new(),
    }
}

impl View {
    pub fn row(mut self) -> Self {
        self.axis = Some(Axis::Row);
        self
    }

    pub fn column(mut self) -> Self {
        self.axis = Some(Axis::Column);
        self
    }

    pub fn gap(mut self, space: Space) -> Self {
        self.gap = Some(space);
        self
    }

    pub fn padding(mut self, space: Space) -> Self {
        self.padding = Some(space);
        self
    }

    pub fn surface(mut self, surface: Surface) -> Self {
        self.surface = surface;
        self
    }

    pub fn rounded(mut self) -> Self {
        self.rounded = true;
        self
    }

    pub fn scroll_x(mut self, name: &'static str) -> Self {
        self.scroll = Scroll::X(name);
        self
    }

    pub fn scroll_y(mut self, name: &'static str) -> Self {
        self.scroll = Scroll::Y(name);
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn center_x(mut self) -> Self {
        self.center_x = true;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    pub fn clickable(mut self) -> Self {
        self.clickable = true;
        self
    }

    pub fn focus(mut self, node: u64) -> Self {
        self.focus = Some(node);
        self.clickable = true;
        self
    }

    pub fn id(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.salt = Some(Id::new(salt));
        self
    }

    pub fn class(mut self, rule: &'static str) -> Self {
        self.class = Some(rule);
        self
    }

    pub fn modifier(mut self, name: &'static str) -> Self {
        self.modifiers.push(name);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> Shown<R> {
        let tokens = ui.tokens();
        let rest = self.class.map(|rule| {
            crate::stylesheet::resolve_with(
                ui.ctx(),
                rule,
                Id::new(("view-rest", rule)),
                crate::stylesheet::State {
                    enabled: true,
                    ..Default::default()
                },
                None,
                &self.modifiers,
            )
        });
        let pad = self
            .padding
            .map(|space| space.px(&tokens))
            .or(rest.map(|style| style.padding))
            .unwrap_or(0.0);
        let gap = self
            .gap
            .map(|space| space.px(&tokens))
            .or(rest.map(|style| style.gap))
            .unwrap_or(0.0);
        let available = ui.available_width();
        let widest = self.max_width.or(rest.and_then(|style| style.max_width));
        let width = widest.map_or(available, |max| max.min(available));
        let inset = if self.center_x {
            (available - width) * 0.5
        } else {
            0.0
        };
        let origin = ui.cursor().min + Vec2::new(inset, 0.0);
        let rect = Rect::from_min_size(origin, Vec2::new(width, ui.available_height()));
        let axis = self
            .axis
            .or(rest
                .and_then(|style| style.direction)
                .map(|direction| match direction {
                    crate::stylesheet::Direction::Row => Axis::Row,
                    crate::stylesheet::Direction::Column => Axis::Column,
                }))
            .unwrap_or(Axis::Column);
        let cross = match rest.and_then(|style| style.align) {
            Some(crate::stylesheet::Alignment::Center) => Align::Center,
            Some(crate::stylesheet::Alignment::End) => Align::Max,
            _ => Align::Min,
        };
        let layout = match axis {
            Axis::Column => Layout::top_down(cross),
            Axis::Row => Layout::left_to_right(cross).with_main_wrap(self.wrap),
        };
        let sense = if self.clickable {
            Sense::click()
        } else {
            Sense::hover()
        };
        let mut builder = UiBuilder::new().max_rect(rect).sense(sense);
        if let Some(salt) = self.salt {
            builder = builder.id_salt(salt);
        }
        let background = ui.painter().add(Shape::Noop);
        let scope = ui.scope_builder(builder, |ui| {
            if let Some(style) = rest {
                ui.multiply_opacity(style.opacity);
            }
            ui.set_width(width);
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.add_space(pad);
            let inner = Rect::from_min_max(
                ui.cursor().min + Vec2::new(pad, 0.0),
                Pos2::new(rect.right() - pad, rect.bottom()),
            );
            let value = ui
                .scope_builder(UiBuilder::new().max_rect(inner).layout(layout), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(gap);
                    let inherit = rest.as_ref();
                    let class = self.class.unwrap_or("");
                    crate::stylesheet::scoped(class, inherit, || match self.scroll {
                        Scroll::None => add(ui),
                        Scroll::X(name) => crate::components::pan_strip(ui, name, add).inner,
                        Scroll::Y(name) => {
                            crate::components::pan_area(
                                ui,
                                name,
                                |area| area.auto_shrink([false, false]),
                                add,
                            )
                            .inner
                        }
                    })
                })
                .inner;
            ui.add_space(pad);
            value
        });
        let response = scope.response;
        let paint = response.rect;
        let styled = self.class.map(|rule| {
            crate::stylesheet::resolve(
                ui.ctx(),
                rule,
                response.id,
                crate::stylesheet::State {
                    hovered: self.clickable && response.hovered(),
                    pressed: self.clickable && response.is_pointer_button_down_on(),
                    focused: response.has_focus(),
                    enabled: true,
                    ..Default::default()
                },
                None,
            )
        });
        let fill = match (self.surface, styled) {
            (_, Some(style)) => Some(style.fill),
            (Surface::Clear, None) => None,
            (Surface::Raised, None) => Some(tokens.surface_raised),
        };
        let radius = match styled {
            Some(style) => style.radius.of(paint.height()),
            None if self.rounded => tokens.card_radius,
            None => 0.0,
        };
        let mut shapes = Vec::new();
        if let Some(fill) = fill {
            shapes.push(Shape::rect_filled(paint, radius, fill));
        }
        if let Some(style) = styled.filter(|style| style.border.a() > 0 && style.stroke > 0.0) {
            shapes.push(Shape::rect_stroke(
                paint.shrink(style.stroke * 0.5),
                radius,
                egui::Stroke::new(style.stroke, style.border),
                egui::StrokeKind::Inside,
            ));
        }
        if styled.is_none()
            && self.clickable
            && (response.hovered() || response.is_pointer_button_down_on())
        {
            shapes.push(Shape::rect_filled(
                paint,
                radius,
                crate::stylesheet::color("well"),
            ));
        }
        ui.painter().set(background, Shape::Vec(shapes));
        let response = match self.focus {
            Some(node) => response.focus_node(ui, node),
            None => response,
        };
        Shown {
            inner: scope.inner,
            response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interact::{begin_focus_frame, take_focus_targets};

    fn frame(add: impl FnOnce(&mut Ui, &mut Vec<Rect>) + 'static) -> (Vec<Rect>, Vec<(u64, Rect)>) {
        let context = egui::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let metrics = UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop));
        provide(&context, metrics);
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            ..Default::default()
        };
        let mut rects = Vec::new();
        let mut targets = Vec::new();
        let mut add = Some(add);
        context
            .run_ui(input, |root| {
                begin_focus_frame(root.ctx(), None);
                egui::CentralPanel::default().show(root, |ui| {
                    if let Some(add) = add.take() {
                        add(ui, &mut rects)
                    }
                });
                targets = take_focus_targets(root.ctx()).0;
            })
            .drop_without_applying_deltas();
        (rects, targets)
    }

    #[test]
    fn padding_grows_the_view_around_its_content() {
        let (rects, _) = frame(|ui, rects| {
            let shown = view().padding(Space::S16).show(ui, |ui| {
                ui.allocate_space(Vec2::new(50.0, 20.0));
            });
            rects.push(shown.response.rect);
        });
        let tokens = UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop));
        let pad = Space::S16.px(&tokens);
        assert!((rects[0].height() - (20.0 + pad * 2.0)).abs() < 0.5);
    }

    #[test]
    fn focus_registers_the_view_rect_as_a_target() {
        let (rects, targets) = frame(|ui, rects| {
            let shown = view().focus(42).show(ui, |ui| {
                ui.allocate_space(Vec2::new(60.0, 30.0));
            });
            rects.push(shown.response.rect);
        });
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].0, 42);
        assert_eq!(targets[0].1, rects[0]);
    }

    #[test]
    fn class_supplies_padding_and_gap() {
        let (rects, _) = frame(|ui, rects| {
            let shown = view().class("stack").show(ui, |ui| {
                rects.push(ui.allocate_space(Vec2::new(40.0, 10.0)).1);
                rects.push(ui.allocate_space(Vec2::new(40.0, 10.0)).1);
            });
            rects.push(shown.response.rect);
        });
        let tokens = UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop));
        let pad = Space::S16.px(&tokens);
        let gap = Space::S8.px(&tokens);
        assert!((rects[1].top() - rects[0].bottom() - gap).abs() < 0.5);
        assert!((rects[2].height() - (20.0 + gap + pad * 2.0)).abs() < 0.5);
    }

    #[test]
    fn explicit_padding_beats_the_class() {
        let (rects, _) = frame(|ui, rects| {
            let shown = view().class("stack").padding(Space::S4).show(ui, |ui| {
                ui.allocate_space(Vec2::new(40.0, 10.0));
            });
            rects.push(shown.response.rect);
        });
        let tokens = UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop));
        assert!((rects[0].height() - (10.0 + Space::S4.px(&tokens) * 2.0)).abs() < 0.5);
    }

    #[test]
    fn row_lays_children_side_by_side() {
        let (rects, _) = frame(|ui, rects| {
            view().row().gap(Space::S8).show(ui, |ui| {
                rects.push(ui.allocate_space(Vec2::new(40.0, 10.0)).1);
                rects.push(ui.allocate_space(Vec2::new(40.0, 10.0)).1);
            });
        });
        assert!(rects[1].left() > rects[0].right());
        assert_eq!(rects[0].top(), rects[1].top());
    }
}
