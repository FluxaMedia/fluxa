use super::*;

const SIZE: f32 = 56.0;
const MARGIN: f32 = 14.0;

pub fn draw_window_fullscreen_button(
    context: &egui::Context,
    viewport: Viewport,
    fullscreen: bool,
    assets: &impl HomeAssets,
    layout: &mut HomeLayout,
) {
    if viewport.form_factor != UiFormFactor::Desktop {
        return;
    }
    let pos = Pos2::new(viewport.width - MARGIN - SIZE, MARGIN);
    let icon = assets.icon(if fullscreen {
        "CornersIn"
    } else {
        "CornersOut"
    });
    egui::Area::new(Id::new("fluxa-window-fullscreen"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let response =
                components::icon_button(ui, icon, SIZE, ink(tone::subtle()), false, false, true);
            layout
                .focusable
                .push((NODE_WINDOW_FULLSCREEN, response.rect));
            if response.clicked() {
                layout.activated = Some(NODE_WINDOW_FULLSCREEN);
            }
        });
}
