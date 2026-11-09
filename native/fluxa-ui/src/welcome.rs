use super::*;
use crate::components::InputKind;
use crate::kit::{self, Role, Type, UiKit};

#[derive(Clone, Debug, Default)]
pub struct WelcomeQr {
    pub modules: Vec<bool>,
    pub width: usize,
}

#[derive(Clone, Debug, Default)]
pub struct WelcomeModel {
    pub language: String,
    pub email: String,
    pub password: String,
    pub show_password: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub qr: Option<WelcomeQr>,
    pub pair_code: Option<(String, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WelcomeRequest {
    SignIn,
    Offline,
}

pub fn draw_welcome(
    context: &crate::gfx::Context,
    viewport: Viewport,
    model: &mut WelcomeModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    painter.rect_filled(screen, 0.0, background());
    let metrics = metrics_for_assets(viewport, assets);
    kit::provide(context, metrics);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let wordmark = assets.logo("wordmark");
    let eye = assets.icon(if model.show_password { "EyeOff" } else { "Eye" });

    crate::gfx::Area::new(Id::new("fluxa-welcome"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(screen.size());
            ui.set_max_size(screen.size());
            crate::components::pan_area(
                ui,
                "fluxa-welcome-scroll",
                |area| area.auto_shrink([false, false]),
                |ui| {
                    if viewport.is_tv() {
                        draw_pairing(ui, viewport, model, wordmark, &t, &mut request);
                    } else {
                        draw_login(ui, viewport, model, wordmark, eye, &t, &mut request);
                    }
                },
            );
        });

    HomeLayout {
        welcome: request,
        ..HomeLayout::default()
    }
}

fn draw_login(
    ui: &mut crate::gfx::Ui,
    viewport: Viewport,
    model: &mut WelcomeModel,
    wordmark: Option<(crate::gfx::TextureId, Vec2)>,
    eye: Option<crate::gfx::TextureId>,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) {
    let theme = ui.tokens();
    ui.add_space(((viewport.height - 620.0) * 0.5).max(32.0));
    kit::view()
        .column()
        .max_width(theme.welcome_max_width)
        .padding(Space::S16)
        .center_x()
        .show(ui, |ui| {
            let width = ui.available_width();
            kit::wordmark(ui, wordmark, 52.0);
            kit::space(ui, Space::S20);
            kit::text(t("auth.welcome_back"), Type::Title)
                .centered()
                .show(ui);
            kit::space(ui, Space::S8);
            kit::text(t("auth.nuvio.subtitle"), Type::Subtitle)
                .role(Role::Secondary)
                .centered()
                .show(ui);
            kit::space(ui, Space::S24);

            field_label(ui, &t("auth.field.email"));
            let email = kit::input(&mut model.email, &t("auth.placeholder.email")).show(ui);
            keep_in_view(ui, &email, viewport.height);
            kit::space(ui, Space::S24);

            field_label(ui, &t("auth.field.password"));
            let kind = if model.show_password {
                InputKind::Text
            } else {
                InputKind::Password
            };
            let field_top = ui.cursor().min;
            let field = Rect::from_min_size(field_top, Vec2::new(width, theme.button_height));
            let hit = Rect::from_center_size(
                Pos2::new(field.right() - 28.0, field.center().y),
                Vec2::splat(40.0),
            );
            let eye_clicked = ui.input(|input| {
                input.pointer.any_click()
                    && input
                        .pointer
                        .interact_pos()
                        .is_some_and(|pos| hit.contains(pos))
            });
            if eye_clicked {
                ui.ctx().options_mut(|options| {
                    options.input_options.surrender_focus_on = crate::gfx::SurrenderFocusOn::Never
                });
            }
            let password = kit::input(&mut model.password, &t("auth.placeholder.password_login"))
                .kind(kind)
                .trailing(56.0)
                .show(ui);
            if eye_clicked {
                ui.ctx().options_mut(|options| {
                    options.input_options.surrender_focus_on = crate::gfx::SurrenderFocusOn::Clicks
                });
            }
            keep_in_view(ui, &password, viewport.height);
            let toggled = ui.interact(hit, Id::new("welcome-show-password"), crate::gfx::Sense::CLICK);
            if let Some(texture) = eye {
                let tint = if toggled.hovered() {
                    theme.text_primary
                } else {
                    theme.text_secondary
                };
                ui.painter().image(
                    texture,
                    Rect::from_center_size(hit.center(), Vec2::splat(20.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    tint,
                );
            }
            if toggled
                .on_hover_cursor(crate::gfx::CursorIcon::PointingHand)
                .clicked()
            {
                model.show_password = !model.show_password;
            }
            kit::space(ui, Space::S4);
            if let Some(key) = model.error.as_deref() {
                components::note(ui, &t(key), *theme);
            }
            kit::space(ui, Space::S48);
            let submitted =
                password.lost_focus() && ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
            let label = if model.busy {
                t("auth.nuvio.signing_in")
            } else {
                t("auth.log_in")
            };
            let sign_in = kit::button(&label).primary().enabled(!model.busy).show(ui);
            if sign_in.clicked() || (submitted && !model.busy) {
                *request = Some(WelcomeRequest::SignIn);
            }
            kit::space(ui, Space::S20);
            kit::labeled_divider(ui, &t("auth.or"));
            kit::space(ui, Space::S20);
            if kit::button(&t("auth.continue_without_account"))
                .show(ui)
                .clicked()
            {
                *request = Some(WelcomeRequest::Offline);
            }
            kit::space(ui, Space::S48);
        });
}

fn keep_in_view(ui: &crate::gfx::Ui, field: &crate::gfx::Response, height: f32) {
    let key = Id::new("welcome-visible-height");
    let changed = ui.data_mut(|data| {
        let before = data.get_temp::<f32>(key);
        data.insert_temp(key, height);
        before != Some(height)
    });
    if field.has_focus() && (changed || field.gained_focus()) {
        let reach = Vec2::new(0.0, 110.0);
        ui.scroll_to_rect(
            Rect::from_min_max(field.rect.min, field.rect.max + reach),
            None,
        );
    }
}

fn field_label(ui: &mut crate::gfx::Ui, text: &str) {
    kit::text(text, Type::Label).role(Role::Secondary).show(ui);
    kit::space(ui, Space::S8);
}

fn draw_pairing(
    ui: &mut crate::gfx::Ui,
    viewport: Viewport,
    model: &mut WelcomeModel,
    wordmark: Option<(crate::gfx::TextureId, Vec2)>,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) {
    let theme = ui.tokens();
    let qr_side = theme.welcome_qr.of(viewport.height);
    let card_side = qr_side + 64.0;
    let text_width = theme.welcome_text.of(viewport.width);
    let gap = Space::S48.px(&theme);
    let total = card_side + gap + text_width;
    ui.add_space(theme.gutter * 0.5);
    kit::view()
        .column()
        .max_width(total)
        .padding(Space::S48)
        .center_x()
        .show(ui, |ui| {
            kit::wordmark(ui, wordmark, theme.welcome_wordmark.of(viewport.height));
            kit::space(ui, Space::S20);
            kit::view().row().gap(Space::S48).show(ui, |ui| {
                kit::view().column().max_width(card_side).show(ui, |ui| {
                    let qr = model.qr.as_ref();
                    kit::qr_card(
                        ui,
                        qr_side,
                        qr.map_or(&[], |qr| qr.modules.as_slice()),
                        qr.map_or(0, |qr| qr.width),
                    );
                    kit::space(ui, Space::S24);
                    let offline = kit::button(&t("auth.continue_without_account"))
                        .autofocus()
                        .show(ui);
                    let enter = ui.input(|input| input.key_pressed(crate::gfx::Key::Enter));
                    if offline.clicked() || enter {
                        *request = Some(WelcomeRequest::Offline);
                    }
                });
                kit::view().column().max_width(text_width).show(ui, |ui| {
                    kit::text(t("auth.welcome_back"), Type::Headline).show(ui);
                    kit::space(ui, Space::S12);
                    kit::text(t("auth.scan_to_connect"), Type::Subtitle)
                        .role(Role::Secondary)
                        .show(ui);
                    kit::space(ui, Space::S16);
                    for (index, key) in [
                        "auth.tv.step_scan",
                        "auth.tv.step_login",
                        "auth.tv.step_return",
                    ]
                    .iter()
                    .enumerate()
                    {
                        kit::step(ui, index + 1, &t(key));
                        kit::space(ui, Space::S8);
                    }
                    if let Some((url, code)) = model.pair_code.as_ref() {
                        kit::text(
                            t("auth.tv.or_enter_code").replacen("%s", url, 1),
                            Type::Body,
                        )
                        .role(Role::Secondary)
                        .show(ui);
                        kit::space(ui, Space::S8);
                        kit::text(code.as_str(), Type::Headline).show(ui);
                        kit::space(ui, Space::S8);
                    }
                    let status = match model.error.as_deref() {
                        Some(key) => t(key),
                        None => t("auth.login_pending"),
                    };
                    kit::text(status, Type::Body).role(Role::Muted).show(ui);
                });
            });
        });
}
