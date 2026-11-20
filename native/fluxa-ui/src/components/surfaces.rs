use super::*;
use crate::flex::Space;
use crate::kit;

pub(crate) fn secondary_button(
    ui: &mut crate::gfx::Ui,
    label: &str,
    width: f32,
) -> crate::gfx::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Secondary,
        size::BUTTON_TEXT,
        true,
        None,
    )
}

pub(crate) fn primary_button(
    ui: &mut crate::gfx::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> crate::gfx::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Primary,
        size::BUTTON_TEXT,
        enabled,
        None,
    )
}
