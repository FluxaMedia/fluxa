use super::*;
use crate::flex::Align;
use crate::kit::{self, Role, Type, UiKit, layout_items};
use std::cell::{Cell, RefCell};

pub(super) fn draw_pin_prompt(
    context: &crate::gfx::Context,
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
    let pin = RefCell::new(prompt.pin.clone());
    let cancel = Cell::new(false);
    let submit = Cell::new(false);
    components::modal(
        context,
        screen,
        "fluxa-profiles-pin",
        |ui| {
            let message = prompt.error.map(|failure| match failure {
                PinFailure::Incorrect => t("profiles.pin_error"),
                PinFailure::Unavailable => t("profiles.pin_unavailable"),
                PinFailure::Locked(seconds) => t("profiles.pin_locked")
                    .replace("{time}", &format!("{}:{:02}", seconds / 60, seconds % 60)),
            });
            let prompt_label = t("profiles.pin_prompt");
            let hint = t("profiles.enter_pin");
            let cancel_label = t("common.cancel");
            let unlock_label = t("profiles.unlock");
            let gap = ui.tokens().space_px(Space::S8);
            let half = (ui.available_width() - gap) * 0.5;
            let escape = ui.input(|input| input.key_pressed(crate::gfx::Key::Escape));
            let items = vec![
                kit::widget(ui, |ui| {
                    kit::avatar(
                        ui,
                        &mut *assets,
                        36.0,
                        &profile.name,
                        profile.avatar_url.as_deref(),
                    );
                })
                .align_self(Align::Center),
                layout_items::space(ui, Space::S12),
                kit::widget(ui, |ui| {
                    kit::text(profile.name.as_str(), Type::Subtitle)
                        .bold()
                        .show(ui);
                })
                .align_self(Align::Center),
                layout_items::space(ui, Space::S4),
                kit::widget(ui, |ui| {
                    kit::text(prompt_label.clone(), Type::Label)
                        .role(Role::Secondary)
                        .show(ui);
                })
                .align_self(Align::Center),
                layout_items::space(ui, Space::S20),
                kit::widget(ui, |ui| {
                    let mut pin = pin.borrow_mut();
                    let field = kit::input(&mut *pin, &hint)
                        .kind(components::InputKind::Number { secret: true })
                        .show(ui);
                    if !field.has_focus() && pin.is_empty() && !crate::kit::probing() {
                        field.request_focus();
                    }
                    pin.retain(|character| character.is_ascii_digit());
                    pin.truncate(4);
                    let entered = pin.len() == 4
                        && field.lost_focus()
                        && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
                    if entered {
                        submit.set(true);
                    }
                }),
                match message.as_deref() {
                    Some(text) => kit::widget(ui, |ui| kit::note(ui, text)),
                    None => layout_items::gap(0.0),
                },
                layout_items::space(ui, Space::S20),
                layout_items::row(vec![
                    kit::widget(ui, |ui| {
                        let response = kit::button(&cancel_label).width(half).show(ui);
                        if !crate::kit::probing() {
                            response.focus_node(ui, NODE_PROFILES_PIN_CANCEL);
                        }
                        if escape {
                            cancel.set(true);
                        }
                    }),
                    kit::widget(ui, |ui| {
                        let response = kit::button(&unlock_label)
                            .primary()
                            .enabled(pin.borrow().len() == 4)
                            .width(half)
                            .show(ui);
                        if !crate::kit::probing() {
                            response.focus_node(ui, NODE_PROFILES_PIN_UNLOCK);
                        }
                    }),
                ])
                .gap(gap),
            ];
            kit::lay(ui, items);
        },
        metrics,
    );
    if submit.get() {
        *request = Some(ProfilesRequest::SubmitPin);
    }
    let pin = pin.into_inner();
    if cancel.get() {
        model.pin_prompt = None;
    } else if let Some(current) = model.pin_prompt.as_mut() {
        if current.pin != pin {
            current.error = None;
        }
        current.pin = pin;
    }
}
