use super::*;
use crate::components::{ButtonKind, InputKind};
use crate::gfx::{Align, Layout, Response, Ui, UiBuilder};
use std::sync::Arc;

mod dialog;
mod image;
mod items;
mod lattice;
mod page;
mod part;
mod solve;
mod view;
pub(crate) use image::fit_inside;
pub(crate) use items::{
    Item, direct, item, lay, lay_hug, lay_in, lay_overflow, lay_point, probing, shared, widget,
};
pub(crate) use lattice::Lattice;
pub(crate) mod layout_items {
    pub(crate) use super::items::{column, fill, gap, row, space, stack};
}
pub(crate) use page::{overlay_body, screen_body, screen_body_below, scroll_state};
pub(crate) use part::{Part, Scene};
pub(crate) use solve::{Flow, Justify, Slot, solve, solve_with};
pub(crate) use view::{Surface, view};

const THEME_KEY: &str = "fluxa-kit-theme";

pub(crate) fn provide(context: &crate::gfx::Context, metrics: UiMetrics) {
    context.data_mut(|data| data.insert_temp(Id::new(THEME_KEY), Arc::new(metrics)));
}

pub(crate) trait UiKit {
    fn tokens(&self) -> Arc<UiMetrics>;
}

impl UiMetrics {}

impl UiKit for Ui {
    fn tokens(&self) -> Arc<UiMetrics> {
        self.data(|data| data.get_temp::<Arc<UiMetrics>>(Id::new(THEME_KEY)))
            .expect("kit::provide must run before kit components")
    }
}

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
    pub(crate) fn size(self, metrics: &UiMetrics) -> f32 {
        match self {
            Type::Headline => metrics.text.title * 1.6,
            Type::Title => metrics.text.title,
            Type::Subtitle => metrics.text.subtitle,
            Type::Body => metrics.text.body,
            Type::Meta => metrics.text.meta,
            Type::Label => metrics.text.label,
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
    pub(crate) fn color(self, metrics: &UiMetrics) -> Color32 {
        match self {
            Role::Primary => metrics.text_primary,
            Role::Secondary => metrics.text_secondary,
            Role::Muted => metrics.text_muted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Modifier {
    max_width: Option<f32>,
    margin: Option<Space>,
    gap: Option<Space>,
    gap_px: Option<f32>,
    center_x: bool,
    wrap: bool,
}

impl Modifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn margin(mut self, space: Space) -> Self {
        self.margin = Some(space);
        self
    }

    pub fn gap(mut self, space: Space) -> Self {
        self.gap = Some(space);
        self
    }

    pub fn gap_px(mut self, gap: f32) -> Self {
        self.gap_px = Some(gap);
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    pub fn center_x(mut self) -> Self {
        self.center_x = true;
        self
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
        let theme = ui.tokens();
        let weight = if self.bold {
            Weight::Bold
        } else {
            self.ty.weight()
        };
        let font = weight.font(self.size.unwrap_or_else(|| self.ty.size(&theme)));
        let color = self.color.unwrap_or_else(|| self.role.color(&theme));
        let job =
            crate::gfx::text::LayoutJob::simple(self.content.clone(), font, color, wrap_width);
        (ui.ctx().fonts_mut(|fonts| fonts.layout_job(job)), color)
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        let weight = if self.bold {
            Weight::Bold
        } else {
            self.ty.weight()
        };
        let mut rich = RichText::new(self.content)
            .font(weight.font(self.size.unwrap_or_else(|| self.ty.size(&theme))))
            .color(self.color.unwrap_or_else(|| self.role.color(&theme)));
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

pub(crate) struct Button<'a> {
    label: &'a str,
    kind: ButtonKind,
    enabled: bool,
    autofocus: bool,
    width: Option<f32>,
}

impl Button<'_> {}

pub(crate) struct Input<'a> {
    value: &'a mut String,
    hint: &'a str,
    kind: InputKind,
    trailing: f32,
    width: Option<f32>,
}

impl Input<'_> {}

pub(crate) struct IconButton<'a> {
    name: &'a str,
    framed: bool,
    danger: bool,
    enabled: bool,
}

impl IconButton<'_> {}
