use super::*;
use crate::components::{ButtonKind, InputKind};
use egui::{Align, Layout, Response, Ui, UiBuilder};
use std::sync::Arc;

mod grid;
mod image;
mod view;
pub(crate) use grid::grid;
pub(crate) use image::image;
pub(crate) use view::{Surface, view};

const THEME_KEY: &str = "fluxa-kit-theme";

pub(crate) fn provide(context: &egui::Context, metrics: UiMetrics) {
    context.data_mut(|data| data.insert_temp(Id::new(THEME_KEY), Arc::new(metrics)));
}

pub(crate) trait UiKit {
    fn tokens(&self) -> Arc<UiMetrics>;
}

impl UiMetrics {
    pub(crate) fn space_px(&self, space: Space) -> f32 {
        space.px(self)
    }
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
    Meta,
    Label,
}

impl Type {
    fn size(self, metrics: &UiMetrics) -> f32 {
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

fn container<R>(
    ui: &mut Ui,
    modifier: Modifier,
    layout: Layout,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let theme = ui.tokens();
    let margin = modifier.margin.map_or(0.0, |space| space.px(&theme));
    let gap = modifier
        .gap_px
        .or_else(|| modifier.gap.map(|space| space.px(&theme)))
        .unwrap_or(0.0);
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
    let layout = Layout::left_to_right(Align::Min).with_main_wrap(modifier.wrap);
    container(ui, modifier, layout, add)
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
    bold: bool,
    badge: bool,
}

pub(crate) fn text(content: impl Into<String>, ty: Type) -> Text {
    Text {
        content: content.into(),
        ty,
        role: Role::Primary,
        centered: false,
        bold: false,
        badge: false,
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

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn badge(mut self) -> Self {
        self.badge = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        let weight = if self.bold {
            Weight::Bold
        } else {
            self.ty.weight()
        };
        let mut rich = RichText::new(self.content)
            .font(weight.font(self.ty.size(&theme)))
            .color(self.role.color(&theme));
        if self.badge {
            rich = rich.background_color(ink(tone::soft()));
        }
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
    autofocus: bool,
    width: Option<f32>,
}

pub(crate) fn button(label: &str) -> Button<'_> {
    Button {
        label,
        kind: ButtonKind::Secondary,
        enabled: true,
        autofocus: false,
        width: None,
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

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn autofocus(mut self) -> Self {
        self.autofocus = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        let response = crate::components::button_core(
            ui,
            self.label,
            self.width.unwrap_or_else(|| ui.available_width()),
            theme.button_height,
            self.kind,
            size::BUTTON_TEXT,
            self.enabled,
            Some(*theme),
        );
        if self.autofocus && !response.has_focus() {
            response.request_focus();
        }
        response
    }
}

pub(crate) struct Input<'a> {
    value: &'a mut String,
    hint: &'a str,
    kind: InputKind,
    trailing: f32,
    width: Option<f32>,
}

pub(crate) fn input<'a>(value: &'a mut String, hint: &'a str) -> Input<'a> {
    Input {
        value,
        hint,
        kind: InputKind::Text,
        trailing: 0.0,
        width: None,
    }
}

impl Input<'_> {
    pub fn kind(mut self, kind: InputKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn trailing(mut self, width: f32) -> Self {
        self.trailing = width;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ui.tokens();
        let width = self.width.unwrap_or_else(|| ui.available_width());
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

pub(crate) fn wordmark(ui: &mut Ui, texture: Option<(egui::TextureId, Vec2)>, height: f32) {
    let width = ui.available_width();
    crate::components::wordmark(ui, texture, width, height);
}

pub(crate) fn qr_card(ui: &mut Ui, side: f32, modules: &[bool], columns: usize) {
    crate::components::qr_card(ui, side, modules, columns);
}

pub(crate) fn step(ui: &mut Ui, number: usize, text: &str) {
    let theme = ui.tokens();
    crate::components::numbered_step(ui, number, text, *theme);
}

pub(crate) fn center<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.vertical_centered(add).inner
}

pub(crate) fn section(ui: &mut Ui, flat: bool, add: impl FnOnce(&mut Ui)) {
    let theme = ui.tokens();
    let width = ui.available_width();
    crate::components::section(ui, flat, width, *theme, add);
}

pub(crate) fn field(ui: &mut Ui, label: &str) {
    let theme = ui.tokens();
    crate::components::field_label(ui, label, *theme);
}

pub(crate) fn note(ui: &mut Ui, text: &str) {
    let theme = ui.tokens();
    crate::components::note(ui, text, *theme);
}

pub(crate) fn checkbox(ui: &mut Ui, value: &mut bool, label: &str) {
    let theme = ui.tokens();
    ui.checkbox(value, RichText::new(label).size(theme.text.meta));
}

pub(crate) fn link(ui: &mut Ui, label: &str) -> Response {
    crate::components::text_button(ui, label, 12.0, 120)
}

pub(crate) fn pill(ui: &mut Ui, label: &str) -> Response {
    let theme = ui.tokens();
    crate::components::pill_button(ui, None, label, None, 40.0, theme.text.meta, false, None)
}

pub(crate) fn avatar(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    radius: f32,
    name: &str,
    url: Option<&str>,
) {
    let theme = ui.tokens();
    crate::components::avatar_tile(ui, assets, radius, name, url, *theme);
}

pub(crate) fn avatar_button(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    radius: f32,
    name: &str,
    url: Option<&str>,
    hover_label: &str,
) -> Response {
    let theme = ui.tokens();
    crate::components::avatar_button(ui, assets, radius, name, url, hover_label, *theme)
}

pub(crate) struct IconButton<'a> {
    name: &'a str,
    framed: bool,
    danger: bool,
    enabled: bool,
}

pub(crate) fn icon_button(name: &str) -> IconButton<'_> {
    IconButton {
        name,
        framed: false,
        danger: false,
        enabled: true,
    }
}

impl IconButton<'_> {
    pub fn framed(mut self) -> Self {
        self.framed = true;
        self
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut Ui, assets: &impl HomeAssets) -> Response {
        let theme = ui.tokens();
        let (size, color) = if self.framed {
            (theme.icon_button_size, ink(tone::body()))
        } else if self.danger {
            (32.0, danger_color())
        } else {
            (32.0, ink(tone::subtle()))
        };
        crate::components::icon_button(
            ui,
            assets.icon(self.name),
            size,
            color,
            self.framed,
            false,
            self.enabled,
        )
    }
}

pub(crate) fn back_link(ui: &mut Ui, label: &str) -> Response {
    crate::components::text_button(ui, &format!("‹  {label}"), 14.0, 255)
}

pub(crate) fn avatar_grid(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    pack_id: &str,
    avatars: &[ProfileAvatar],
    selected: Option<&str>,
) -> Option<usize> {
    let theme = ui.tokens();
    let width = ui.available_width();
    crate::components::avatar_grid(ui, assets, width, pack_id, avatars, selected, *theme)
}

pub(crate) fn pack_row(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    compact: bool,
    pack: &ProfileAvatarPack,
    busy: bool,
    tips: (&str, &str),
) -> Option<crate::components::PackAction> {
    let theme = ui.tokens();
    crate::components::pack_row(ui, assets, compact, pack, busy, tips, *theme)
}

pub(crate) fn pill_wide(ui: &mut Ui, label: &str, width: f32, primary: bool) -> Response {
    let theme = ui.tokens();
    crate::components::pill_button(
        ui,
        None,
        label,
        Some(width),
        theme.button_height,
        theme.text.body,
        primary,
        None,
    )
}
