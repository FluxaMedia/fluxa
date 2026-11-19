use super::*;
use crate::components::{ButtonKind, InputKind};
use crate::gfx::{Align, Layout, Response, Ui, UiBuilder};
use std::sync::Arc;

mod dialog;
mod flex;
mod grid;
mod image;
mod items;
mod page;
mod part;
mod solve;
mod view;
pub(crate) use dialog::{dialog, dialog_with};
pub(crate) use flex::flex;
pub(crate) use grid::{Lattice, grid};
pub(crate) use image::fit_inside;
pub(crate) use items::{
    Item, at, direct, item, lay, lay_hug, lay_overflow, lay_in, lay_point, probing, shared, widget,
};
pub(crate) mod layout_items {
    pub(crate) use super::items::{column, fill, gap, row, space, stack};
}
pub(crate) use page::{
    overlay_body, page, screen_body, screen_body_below, scroll_body, scroll_body_keyed,
    scroll_state,
};
pub(crate) use part::{Part, Scene};
pub(crate) use solve::{Flow, Justify, Slot, solve, solve_with};
pub(crate) use view::{Surface, view};

const THEME_KEY: &str = "fluxa-kit-theme";

pub(crate) fn provide(context: &crate::gfx::Context, metrics: UiMetrics) {
    context.data_mut(|data| data.insert_temp(Id::new(THEME_KEY), Arc::new(metrics)));
}

pub(crate) trait UiKit {
    fn tokens(&self) -> Arc<UiMetrics>;
    fn gap(&mut self, space: Space);
}

impl UiMetrics {
    pub(crate) fn space_px(&self, space: Space) -> f32 {
        space.px(self)
    }
}

impl UiKit for Ui {
    fn gap(&mut self, space: Space) {
        let px = space.px(&crate::metrics::active());
        let size = if self.layout().is_horizontal() {
            Vec2::new(px, 0.0)
        } else {
            Vec2::new(0.0, px)
        };
        self.allocate_exact_size(size, crate::gfx::Sense::hover());
    }

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
    let room = ui.available_rect_before_wrap();
    let rect = view::column_slot(
        room,
        width,
        room.height(),
        modifier.center_x,
        if modifier.center_x { 0.0 } else { margin },
    );
    ui.scope_builder(UiBuilder::new().max_rect(rect).layout(layout), |ui| {
        ui.set_width(width);
        ui.set_item_spacing(Vec2::splat(gap));
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

pub(crate) fn center<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope_builder(
        UiBuilder::new().layout(Layout::top_down(Align::Center)),
        add,
    )
    .inner
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
            (
                theme.icon_button_size,
                crate::stylesheet::color("text.body"),
            )
        } else if self.danger {
            (32.0, danger_color())
        } else {
            (32.0, crate::stylesheet::color("text.subtle"))
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
    crate::components::avatar_grid(ui, assets, pack_id, avatars, selected, *theme)
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
