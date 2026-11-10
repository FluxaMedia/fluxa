use super::{
    Align2, Color32, Context, CornerRadius, FontId, Galley, LayerId, Pos2, Rect, Shape, Stroke,
    StrokeKind, TextureId, transparent,
};
use egui::epaint::text::LayoutJob;
use egui::layers::ShapeIdx;
use std::sync::Arc;

#[derive(Clone)]
#[repr(transparent)]
pub struct Painter(egui::Painter);

transparent!(Painter, egui::Painter);

impl Painter {
    pub(in crate::gfx) fn from_egui(inner: egui::Painter) -> Self {
        Self(inner)
    }

    pub fn with_clip_rect(&self, rect: Rect) -> Self {
        Self(self.0.with_clip_rect(rect))
    }

    pub fn clip_rect(&self) -> Rect {
        self.0.clip_rect()
    }

    pub fn multiply_opacity(&mut self, opacity: f32) {
        self.0.multiply_opacity(opacity);
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.0.set_opacity(opacity);
    }

    pub fn ctx(&self) -> &Context {
        Context::wrap(self.0.ctx())
    }

    pub fn layer_id(&self) -> LayerId {
        self.0.layer_id()
    }

    pub fn add(&self, shape: impl Into<Shape>) -> ShapeIdx {
        self.0.add(shape)
    }

    pub fn set(&self, idx: ShapeIdx, shape: impl Into<Shape>) {
        self.0.set(idx, shape);
    }

    pub fn line_segment(&self, points: [Pos2; 2], stroke: impl Into<Stroke>) -> ShapeIdx {
        self.0.line_segment(points, stroke)
    }

    pub fn line(&self, points: Vec<Pos2>, stroke: impl Into<egui::epaint::PathStroke>) -> ShapeIdx {
        self.0.line(points, stroke)
    }

    pub fn hline(
        &self,
        x: impl Into<egui::emath::Rangef>,
        y: f32,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx {
        self.0.hline(x, y, stroke)
    }

    pub fn circle(
        &self,
        center: Pos2,
        radius: f32,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx {
        self.0.circle(center, radius, fill, stroke)
    }

    pub fn circle_filled(&self, center: Pos2, radius: f32, fill: impl Into<Color32>) -> ShapeIdx {
        self.0.circle_filled(center, radius, fill)
    }

    pub fn circle_stroke(&self, center: Pos2, radius: f32, stroke: impl Into<Stroke>) -> ShapeIdx {
        self.0.circle_stroke(center, radius, stroke)
    }

    pub fn rect(
        &self,
        rect: Rect,
        radius: impl Into<CornerRadius>,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
        kind: StrokeKind,
    ) -> ShapeIdx {
        self.0.rect(rect, radius, fill, stroke, kind)
    }

    pub fn rect_filled(
        &self,
        rect: Rect,
        radius: impl Into<CornerRadius>,
        fill: impl Into<Color32>,
    ) -> ShapeIdx {
        self.0.rect_filled(rect, radius, fill)
    }

    pub fn rect_stroke(
        &self,
        rect: Rect,
        radius: impl Into<CornerRadius>,
        stroke: impl Into<Stroke>,
        kind: StrokeKind,
    ) -> ShapeIdx {
        self.0.rect_stroke(rect, radius, stroke, kind)
    }

    pub fn image(&self, texture: TextureId, rect: Rect, uv: Rect, tint: Color32) -> ShapeIdx {
        self.0.image(texture, rect, uv, tint)
    }

    pub fn text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font: FontId,
        color: Color32,
    ) -> Rect {
        self.0.text(pos, anchor, text, font, color)
    }

    pub fn layout(
        &self,
        text: String,
        font: FontId,
        color: Color32,
        wrap_width: f32,
    ) -> Arc<Galley> {
        self.0.layout(text, font, color, wrap_width)
    }

    pub fn layout_no_wrap(&self, text: String, font: FontId, color: Color32) -> Arc<Galley> {
        self.0.layout_no_wrap(text, font, color)
    }

    pub fn layout_job(&self, job: LayoutJob) -> Arc<Galley> {
        self.0.layout_job(job)
    }

    pub fn galley(&self, pos: Pos2, galley: Arc<Galley>, fallback: Color32) {
        self.0.galley(pos, galley, fallback);
    }

    pub fn galley_with_override_text_color(&self, pos: Pos2, galley: Arc<Galley>, color: Color32) {
        self.0.galley_with_override_text_color(pos, galley, color);
    }

    pub fn pixels_per_point(&self) -> f32 {
        self.0.pixels_per_point()
    }
}
