use super::{
    Align, Color32, Context, Data, InnerResponse, Layout, Painter, Rect, Response, Sense, Stroke,
    UiBuilder, Vec2, transparent,
};

#[repr(transparent)]
pub struct Ui(egui::Ui);

transparent!(Ui, egui::Ui);

impl Ui {
    pub(in crate::gfx) fn from_egui(inner: egui::Ui) -> Self {
        Self(inner)
    }

    pub fn ctx(&self) -> &Context {
        Context::wrap(self.0.ctx())
    }

    pub fn painter(&self) -> &Painter {
        Painter::wrap(self.0.painter())
    }

    pub fn painter_at(&self, rect: Rect) -> Painter {
        Painter::from_egui(self.0.painter_at(rect))
    }

    pub fn id(&self) -> super::Id {
        self.0.id()
    }

    pub fn auto_id_with(&self, salt: impl std::hash::Hash + std::fmt::Debug) -> super::Id {
        self.0.auto_id_with(salt)
    }

    pub fn make_persistent_id(&self, salt: impl std::hash::Hash + std::fmt::Debug) -> super::Id {
        self.0.make_persistent_id(salt)
    }

    pub fn layer_id(&self) -> super::LayerId {
        self.0.layer_id()
    }

    pub fn data<R>(&self, reader: impl FnOnce(&Data) -> R) -> R {
        self.ctx().data(reader)
    }

    pub fn data_mut<R>(&self, writer: impl FnOnce(&mut Data) -> R) -> R {
        self.ctx().data_mut(writer)
    }

    pub fn input<R>(&self, reader: impl FnOnce(&super::InputState) -> R) -> R {
        self.ctx().input(reader)
    }

    pub fn available_width(&self) -> f32 {
        self.0.available_width()
    }

    pub fn available_height(&self) -> f32 {
        self.0.available_height()
    }

    pub fn available_size(&self) -> Vec2 {
        self.0.available_size()
    }

    pub fn available_rect_before_wrap(&self) -> Rect {
        self.0.available_rect_before_wrap()
    }

    pub fn max_rect(&self) -> Rect {
        self.0.max_rect()
    }

    pub fn min_rect(&self) -> Rect {
        self.0.min_rect()
    }

    pub fn clip_rect(&self) -> Rect {
        self.0.clip_rect()
    }

    pub fn set_clip_rect(&mut self, rect: Rect) {
        self.0.set_clip_rect(rect);
    }

    pub fn is_rect_visible(&self, rect: Rect) -> bool {
        self.0.is_rect_visible(rect)
    }

    pub fn rect_contains_pointer(&self, rect: Rect) -> bool {
        self.0.rect_contains_pointer(rect)
    }

    pub fn layout(&self) -> &Layout {
        self.0.layout()
    }

    pub fn cursor(&self) -> Rect {
        self.0.cursor()
    }

    pub fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    pub fn set_width(&mut self, width: f32) {
        self.0.set_width(width);
    }

    pub fn set_height(&mut self, height: f32) {
        self.0.set_height(height);
    }

    pub fn set_min_width(&mut self, width: f32) {
        self.0.set_min_width(width);
    }

    pub fn set_min_height(&mut self, height: f32) {
        self.0.set_min_height(height);
    }

    pub fn set_max_width(&mut self, width: f32) {
        self.0.set_max_width(width);
    }

    pub fn set_max_height(&mut self, height: f32) {
        self.0.set_max_height(height);
    }

    pub fn set_min_size(&mut self, size: Vec2) {
        self.0.set_min_size(size);
    }

    pub fn set_max_size(&mut self, size: Vec2) {
        self.0.set_max_size(size);
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.0.set_opacity(opacity);
    }

    pub fn multiply_opacity(&mut self, opacity: f32) {
        self.0.multiply_opacity(opacity);
    }

    pub fn set_item_spacing(&mut self, spacing: Vec2) {
        self.0.spacing_mut().item_spacing = spacing;
    }

    pub fn set_item_spacing_x(&mut self, spacing: f32) {
        self.0.spacing_mut().item_spacing.x = spacing;
    }

    pub fn set_item_spacing_y(&mut self, spacing: f32) {
        self.0.spacing_mut().item_spacing.y = spacing;
    }

    pub fn item_spacing(&self) -> Vec2 {
        self.0.spacing().item_spacing
    }

    pub fn hide_text_selection(&mut self) {
        let selection = &mut self.0.visuals_mut().selection;
        selection.bg_fill = Color32::TRANSPARENT;
        selection.stroke = Stroke::NONE;
    }

    pub fn strong_text_color(&self) -> Color32 {
        self.0.visuals().strong_text_color()
    }

    pub fn fonts_mut<R>(&self, writer: impl FnOnce(&mut super::Fonts<'_>) -> R) -> R {
        self.ctx().fonts_mut(writer)
    }

    pub fn has_focus(&self, id: super::Id) -> bool {
        self.ctx().has_focus(id)
    }

    pub fn request_focus(&self, id: super::Id) {
        self.ctx().request_focus(id);
    }

    pub fn expand_to_include_rect(&mut self, rect: Rect) {
        self.0.expand_to_include_rect(rect);
    }

    pub fn checkbox(&mut self, value: &mut bool, label: impl Into<egui::WidgetText>) -> Response {
        Response::from_egui(self.0.checkbox(value, label))
    }

    pub fn allocate_exact_size(&mut self, size: Vec2, sense: Sense) -> (Rect, Response) {
        let (rect, response) = self.0.allocate_exact_size(size, sense);
        (rect, Response::from_egui(response))
    }

    pub fn allocate_space(&mut self, size: Vec2) -> (super::Id, Rect) {
        self.0.allocate_space(size)
    }

    pub fn allocate_rect(&mut self, rect: Rect, sense: Sense) -> Response {
        Response::from_egui(self.0.allocate_rect(rect, sense))
    }

    pub fn allocate_response(&mut self, size: Vec2, sense: Sense) -> Response {
        Response::from_egui(self.0.allocate_response(size, sense))
    }

    pub fn advance_cursor_after_rect(&mut self, rect: Rect) -> super::Id {
        self.0.advance_cursor_after_rect(rect)
    }

    pub fn add_space(&mut self, amount: f32) {
        self.0.add_space(amount);
    }

    pub fn interact(&self, rect: Rect, id: super::Id, sense: Sense) -> Response {
        Response::from_egui(self.0.interact(rect, id, sense))
    }

    pub fn add(&mut self, widget: impl egui::Widget) -> Response {
        Response::from_egui(self.0.add(widget))
    }

    pub fn put(&mut self, rect: Rect, widget: impl egui::Widget) -> Response {
        Response::from_egui(self.0.put(rect, widget))
    }

    pub fn place(&mut self, rect: Rect, widget: impl egui::Widget) -> Response {
        Response::from_egui(self.0.put(rect, widget))
    }

    pub fn label(&mut self, text: impl Into<egui::WidgetText>) -> Response {
        Response::from_egui(self.0.label(text))
    }

    pub fn scope<R>(&mut self, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        InnerResponse::from_egui(self.0.scope(|ui| body(Ui::wrap_mut(ui))))
    }

    pub fn scope_builder<R>(
        &mut self,
        builder: UiBuilder,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        InnerResponse::from_egui(self.0.scope_builder(builder, |ui| body(Ui::wrap_mut(ui))))
    }

    pub fn new_child(&mut self, builder: UiBuilder) -> Ui {
        Ui::from_egui(self.0.new_child(builder))
    }

    pub fn scroll_to_rect(&self, rect: Rect, align: Option<Align>) {
        self.0.scroll_to_rect(rect, align);
    }
}
