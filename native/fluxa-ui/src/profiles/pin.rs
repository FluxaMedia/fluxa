use super::*;

pub(super) fn draw_pin_prompt(
    context: &egui::Context,
    screen: Rect,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    let Some(prompt) = model.pin_prompt.clone() else {
        return;
    };
    let profile = model
        .profile(&prompt.profile_id)
        .cloned()
        .unwrap_or_default();
    let mut pin = prompt.pin.clone();
    let mut cancel = false;
    components::modal(
        context,
        screen,
        "fluxa-profiles-pin",
        |ui| {
            ui.vertical_centered(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(72.0), Sense::hover());
                components::avatar(
                    ui.painter(),
                    assets,
                    rect.center(),
                    36.0,
                    &profile.name,
                    profile.avatar_url.as_deref(),
                );
                ui.add_space(12.0);
                ui.label(
                    RichText::new(&profile.name)
                        .size(18.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(t("profiles.pin_prompt"))
                        .size(13.0)
                        .color(Color32::from_white_alpha(140)),
                );
            });
            ui.add_space(18.0);
            let width = ui.available_width();
            let field = components::text_input(
                ui,
                &mut pin,
                &t("profiles.enter_pin"),
                width,
                true,
                metrics,
            );
            if !field.has_focus() && pin.is_empty() {
                field.request_focus();
            }
            pin.retain(|character| character.is_ascii_digit());
            pin.truncate(4);
            if prompt.error {
                components::note(ui, &t("profiles.pin_error"));
            }
            ui.add_space(18.0);
            let submit = pin.len() == 4
                && (field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let half = (width - 8.0) * 0.5;
                if components::secondary_button(ui, &t("common.cancel"), half).clicked()
                    || ui.input(|input| input.key_pressed(egui::Key::Escape))
                {
                    cancel = true;
                }
                if components::primary_button(ui, &t("profiles.unlock"), half, pin.len() == 4)
                    .clicked()
                    || submit
                {
                    *request = Some(ProfilesRequest::SubmitPin);
                }
            });
        },
        metrics,
    );
    if cancel {
        model.pin_prompt = None;
    } else if let Some(current) = model.pin_prompt.as_mut() {
        if current.pin != pin {
            current.error = false;
        }
        current.pin = pin;
    }
}
