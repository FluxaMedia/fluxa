use super::*;
use crate::kit::{self, Modifier, Role, Type, UiKit};

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
            kit::center(ui, |ui| {
                kit::avatar(
                    ui,
                    assets,
                    36.0,
                    &profile.name,
                    profile.avatar_url.as_deref(),
                );
                kit::space(ui, Space::S12);
                kit::text(profile.name.as_str(), Type::Subtitle)
                    .bold()
                    .show(ui);
                kit::space(ui, Space::S4);
                kit::text(t("profiles.pin_prompt"), Type::Label)
                    .role(Role::Secondary)
                    .show(ui);
            });
            kit::space(ui, Space::S20);
            let width = ui.available_width();
            let field = kit::input(&mut pin, &t("profiles.enter_pin"))
                .kind(components::InputKind::Number { secret: true })
                .show(ui);
            if !field.has_focus() && pin.is_empty() {
                field.request_focus();
            }
            pin.retain(|character| character.is_ascii_digit());
            pin.truncate(4);
            if let Some(failure) = prompt.error {
                let message = match failure {
                    PinFailure::Incorrect => t("profiles.pin_error"),
                    PinFailure::Unavailable => t("profiles.pin_unavailable"),
                    PinFailure::Locked(seconds) => t("profiles.pin_locked")
                        .replace("{time}", &format!("{}:{:02}", seconds / 60, seconds % 60)),
                };
                kit::note(ui, &message);
            }
            kit::space(ui, Space::S20);
            let submit = pin.len() == 4
                && (field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)));
            let half = (width - ui.tokens().space_px(Space::S8)) * 0.5;
            kit::row(ui, Modifier::new().gap(Space::S8), |ui| {
                kit::button(&t("common.cancel"))
                    .width(half)
                    .show(ui)
                    .focus_node(ui, NODE_PROFILES_PIN_CANCEL);
                if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                    cancel = true;
                }
                kit::button(&t("profiles.unlock"))
                    .primary()
                    .enabled(pin.len() == 4)
                    .width(half)
                    .show(ui)
                    .focus_node(ui, NODE_PROFILES_PIN_UNLOCK);
                if submit {
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
            current.error = None;
        }
        current.pin = pin;
    }
}
