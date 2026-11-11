use super::{Color32, FontId, TextureId, Vec2};

#[derive(Clone)]
pub struct RichText(egui::RichText);

impl RichText {
    pub fn new(text: impl Into<String>) -> Self {
        Self(egui::RichText::new(text))
    }

    pub fn size(self, size: f32) -> Self {
        Self(self.0.size(size))
    }

    pub fn color(self, color: Color32) -> Self {
        Self(self.0.color(color))
    }

    pub fn strong(self) -> Self {
        Self(self.0.strong())
    }

    pub fn italics(self) -> Self {
        Self(self.0.italics())
    }

    pub fn underline(self) -> Self {
        Self(self.0.underline())
    }

    pub fn monospace(self) -> Self {
        Self(self.0.monospace())
    }

    pub fn font(self, font: FontId) -> Self {
        Self(self.0.font(font))
    }

    pub fn background_color(self, color: Color32) -> Self {
        Self(self.0.background_color(color))
    }

    pub(in crate::gfx) fn into_egui(self) -> egui::RichText {
        self.0
    }
}

impl From<RichText> for egui::WidgetText {
    fn from(text: RichText) -> Self {
        text.0.into()
    }
}

pub struct Label(egui::Label);

impl Label {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self(egui::Label::new(text))
    }

    pub fn wrap(self) -> Self {
        Self(self.0.wrap())
    }

    pub fn truncate(self) -> Self {
        Self(self.0.truncate())
    }
}

impl egui::Widget for Label {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.0.ui(ui)
    }
}

pub struct Button(egui::Button<'static>);

impl Button {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self(egui::Button::new(text))
    }

    pub fn frame(self, frame: bool) -> Self {
        Self(self.0.frame(frame))
    }
}

impl egui::Widget for Button {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.0.ui(ui)
    }
}

pub struct Image(egui::Image<'static>);

impl Image {
    pub fn new(source: (TextureId, Vec2)) -> Self {
        Self(egui::Image::new(source))
    }
}

impl egui::Widget for Image {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.0.ui(ui)
    }
}
