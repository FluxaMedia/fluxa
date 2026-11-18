use super::page::Frame;
use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets, Justify};
use crate::gfx::Ui;

pub(super) struct PlacedRow {
    index: usize,
    card: Rect,
    top: f32,
    extra: f32,
    first: bool,
}

#[derive(Clone, Copy)]
enum Control {
    Slider,
    Sheet,
    Segmented,
    Dropdown,
    Toggle,
}

struct RowCtx<'a> {
    context: &'a crate::gfx::Context,
    painter: &'a crate::gfx::Painter,
    settings: &'a SettingsModel,
    setting: &'a SettingsRow,
    language: &'a str,
    metrics: UiMetrics,
    row_metrics: UiMetrics,
    compact: bool,
    tv: bool,
    node: u64,
    row_rect: Rect,
    zone: Rect,
    slot: Rect,
    value_size: f32,
    value_width: f32,
    value_font: FontId,
    value: Option<String>,
    raw_value: Option<&'a str>,
    option_labels: Vec<String>,
    value_below: bool,
    is_app_icon: bool,
}
