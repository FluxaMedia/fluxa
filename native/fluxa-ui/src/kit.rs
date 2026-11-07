use super::*;
use crate::components::{ButtonKind, InputKind};
use egui::{Align, Layout, Response, Ui, UiBuilder};
use std::sync::Arc;

const THEME_KEY: &str = "fluxa-kit-theme";

pub(crate) fn provide(context: &egui::Context, metrics: UiMetrics) {
    context.data_mut(|data| data.insert_temp(Id::new(THEME_KEY), Arc::new(metrics)));
}

pub(crate) trait UiKit {
    fn tokens(&self) -> Arc<UiMetrics>;
}

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
    Label,
}

impl Type {
    fn size(self, metrics: &UiMetrics) -> f32 {
        match self {
            Type::Headline => metrics.text.title * 1.6,
            Type::Title => metrics.text.title,
            Type::Subtitle => metrics.text.subtitle,
            Type::Body => metrics.text.body,
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
    fn color(self, metrics: &UiMetrics) -> Color32 {
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
    center_x: bool,
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

    pub fn center_x(mut self) -> Self {
        self.center_x = true;
        self
    }
}

fn container<R>(
    ui: &mut Ui,
    modifier: Modifier,
    layout: Layout,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let theme = ui.tokens();
    let margin = modifier.margin.map_or(0.0, |space| space.px(&theme));
    let gap = modifier.gap.map_or(0.0, |space| space.px(&theme));
    let available = ui.available_width();
    let width = modifier
        .max_width
        .unwrap_or(f32::INFINITY)
        .min(available - margin * 2.0)
        .max(0.0);
    let inset = if modifier.center_x {
        (available - width) * 0.5
    } else {
        margin
    };
    let origin = ui.cursor().min + Vec2::new(inset, 0.0);
    let rect = Rect::from_min_size(origin, Vec2::new(width, ui.available_height()));
    ui.scope_builder(UiBuilder::new().max_rect(rect).layout(layout), |ui| {
        ui.set_width(width);
        ui.spacing_mut().item_spacing = Vec2::splat(gap);
        add(ui)
    })
    .inner
}

pub(crate) fn column<R>(ui: &mut Ui, modifier: Modifier, add: impl FnOnce(&mut Ui) -> R) -> R {
    container(ui, modifier, Layout::top_down(Align::Min), add)
}

pub(crate) fn row<R>(ui: &mut Ui, modifier: Modifier, add: impl FnOnce(&mut Ui) -> R) -> R {
    container(ui, modifier, Layout::left_to_right(Align::Center), add)
}

pub(crate) fn space(ui: &mut Ui, space: Space) {
    let theme = ui.tokens();
    ui.add_space(space.px(&theme));
}

pub(crate) struct Text {
    content: String,
    ty: Type,
    role: Role,
    centered: bool,
}

pub(crate) fn text(content: impl Into<String>, ty: Type) -> Text {
    Text {
        content: content.into(),
        ty,
        role: Role::Primary,
        centered: false,
    }
}

impl Text {
    pub fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }

    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        let rich = RichText::new(self.content)
            .font(self.ty.weight().font(self.ty.size(&theme)))
            .color(self.role.color(&theme));
        let label = egui::Label::new(rich).wrap();
        if self.centered {
            ui.vertical_centered(|ui| ui.add(label)).inner
        } else {
            ui.add(label)
        }
    }
}

pub(crate) struct Button<'a> {
    label: &'a str,
    kind: ButtonKind,
    enabled: bool,
}

pub(crate) fn button(label: &str) -> Button<'_> {
    Button {
        label,
        kind: ButtonKind::Secondary,
        enabled: true,
    }
}

impl Button<'_> {
    pub fn primary(mut self) -> Self {
        self.kind = ButtonKind::Primary;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        crate::components::button_core(
            ui,
            self.label,
            ui.available_width(),
            theme.button_height,
            self.kind,
            size::BUTTON_TEXT,
            self.enabled,
            Some(*theme),
        )
    }
}

pub(crate) struct Input<'a> {
    value: &'a mut String,
    hint: &'a str,
    kind: InputKind,
    trailing: f32,
}

pub(crate) fn input<'a>(value: &'a mut String, hint: &'a str) -> Input<'a> {
    Input {
        value,
        hint,
        kind: InputKind::Text,
        trailing: 0.0,
    }
}

impl Input<'_> {
    pub fn kind(mut self, kind: InputKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn trailing(mut self, width: f32) -> Self {
        self.trailing = width;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        let width = ui.available_width();
        if self.trailing > 0.0 {
            crate::components::text_input_trailing(
                ui,
                self.value,
                self.hint,
                width,
                self.kind,
                self.trailing,
                *theme,
            )
        } else {
            crate::components::text_input(ui, self.value, self.hint, width, self.kind, *theme)
        }
    }
}

pub(crate) fn labeled_divider(ui: &mut Ui, label: &str) {
    let theme = ui.tokens();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 16.0), Sense::hover());
    let painter = ui.painter();
    let galley = painter.layout_no_wrap(
        label.to_owned(),
        FontId::proportional(theme.text.label),
        theme.text_muted,
    );
    let half = galley.size().x * 0.5 + 12.0;
    let line = egui::Stroke::new(stroke::hairline(), theme.border);
    painter.hline(rect.left()..=rect.center().x - half, rect.center().y, line);
    painter.hline(rect.center().x + half..=rect.right(), rect.center().y, line);
    painter.galley(
        rect.center() - galley.size() * 0.5,
        galley,
        theme.text_muted,
    );
}
