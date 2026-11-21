use super::*;
use crate::gfx::{Align, Layout, Response, Ui, UiBuilder};

mod dialog;
mod image;
mod items;
mod page;
mod view;
pub(crate) use image::fit_inside;
pub(crate) use items::{
    Item, direct, item, lay, lay_hug, lay_in, lay_overflow, lay_point, probing, shared, widget,
};
pub(crate) mod layout_items {
    pub(crate) use super::items::{column, fill, gap, row, space, stack};
}
pub(crate) use page::{overlay_body, screen_body, scroll_state};
pub(crate) use view::view;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Headline,
    Title,
    Subtitle,
    Body,
    Meta,
    Label,
}

impl Type {
    pub(crate) fn size(self) -> f32 {
        match self {
            Type::Headline => crate::vars::px("text-title") * 1.6,
            Type::Title => crate::vars::px("text-title"),
            Type::Subtitle => crate::vars::px("text-subtitle"),
            Type::Body => crate::vars::px("text-body"),
            Type::Meta => crate::vars::px("text-meta"),
            Type::Label => crate::vars::px("text-label"),
        }
    }

    fn weight(self) -> Weight {
        match self {
            Type::Headline | Type::Title | Type::Label => Weight::Bold,
            _ => Weight::Regular,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Primary,
    Secondary,
    Muted,
}

impl Role {
    pub(crate) fn color(self) -> Color32 {
        match self {
            Role::Primary => crate::metrics::theme().text_primary,
            Role::Secondary => crate::metrics::theme().text_secondary,
            Role::Muted => crate::metrics::theme().text_muted,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Text {
    content: String,
    ty: Type,
    role: Role,
    centered: bool,
    bold: bool,
    badge: bool,
    size: Option<f32>,
    color: Option<Color32>,
    inert: bool,
}

pub(crate) fn text(content: impl Into<String>, ty: Type) -> Text {
    Text {
        content: content.into(),
        ty,
        role: Role::Primary,
        centered: false,
        bold: false,
        badge: false,
        size: None,
        color: None,
        inert: false,
    }
}

impl Text {
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn inert(mut self) -> Self {
        self.inert = true;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn galley(
        &self,
        ui: &Ui,
        wrap_width: f32,
    ) -> (std::sync::Arc<crate::gfx::text::Galley>, Color32) {
        let weight = if self.bold {
            Weight::Bold
        } else {
            self.ty.weight()
        };
        let font = weight.font(self.size.unwrap_or_else(|| self.ty.size()));
        let color = self.color.unwrap_or_else(|| self.role.color());
        let job =
            crate::gfx::text::LayoutJob::simple(self.content.clone(), font, color, wrap_width);
        (ui.ctx().fonts_mut(|fonts| fonts.layout_job(job)), color)
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let weight = if self.bold {
            Weight::Bold
        } else {
            self.ty.weight()
        };
        let mut rich = RichText::new(self.content)
            .font(weight.font(self.size.unwrap_or_else(|| self.ty.size())))
            .color(self.color.unwrap_or_else(|| self.role.color()));
        if self.badge {
            rich = rich.background_color(crate::stylesheet::color("track"));
        }
        let inert = self.inert;
        if self.centered {
            let mut response = None;
            let label = widget(ui, |ui| {
                response = Some(ui.add(label(rich.clone(), inert)));
            })
            .align_self(crate::flex::Align::Center);
            let used = lay(ui, vec![label]);
            response.unwrap_or_else(|| {
                ui.interact(used, ui.id().with("text"), crate::gfx::Sense::hover())
            })
        } else {
            ui.add(label(rich, self.inert))
        }
    }
}

fn label(rich: RichText, inert: bool) -> crate::gfx::Label {
    let label = crate::gfx::Label::new(rich).wrap();
    if inert { label.inert() } else { label }
}
