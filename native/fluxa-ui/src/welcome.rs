use super::*;
use crate::components::InputKind;
use crate::kit::{self, Modifier, Role, Type, UiKit};

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
    context: &egui::Context,
    viewport: Viewport,
    model: &mut WelcomeModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, background());
    let metrics = metrics_for_assets(viewport, assets);
    kit::provide(context, metrics);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let wordmark = assets.logo("wordmark");
    let eye = assets.icon(if model.show_password { "EyeOff" } else { "Eye" });

    egui::Area::new(Id::new("fluxa-welcome"))
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
                        draw_pairing(ui, viewport, model, wordmark, metrics, &t, &mut request);
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

fn paint_wordmark(
    ui: &mut egui::Ui,
    wordmark: Option<(egui::TextureId, Vec2)>,
    width: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let Some((texture, aspect)) = wordmark else {
        return;
    };
    let size = Vec2::new(height * aspect.x, height);
    ui.painter().image(
        texture,
        Rect::from_center_size(rect.center(), size),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn centered_text(ui: &mut egui::Ui, text: String, size: f32, color: Color32, strong: bool) {
    let mut label = RichText::new(text).size(size).color(color);
    if strong {
        label = label.strong();
    }
    ui.vertical_centered(|ui| ui.add(egui::Label::new(label).wrap()));
}

fn draw_login(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut WelcomeModel,
    wordmark: Option<(egui::TextureId, Vec2)>,
    eye: Option<egui::TextureId>,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) {
    let theme = ui.tokens();
    ui.add_space(((viewport.height - 620.0) * 0.5).max(32.0));
    let layout = Modifier::new()
        .max_width(theme.welcome_max_width)
        .margin(Space::S16)
        .center_x();
    kit::column(ui, layout, |ui| {
        let width = ui.available_width();
        paint_wordmark(ui, wordmark, width, 52.0);
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
                options.input_options.surrender_focus_on = egui::SurrenderFocusOn::Never
            });
        }
        let password = kit::input(&mut model.password, &t("auth.placeholder.password_login"))
            .kind(kind)
            .trailing(56.0)
            .show(ui);
        if eye_clicked {
            ui.ctx().options_mut(|options| {
                options.input_options.surrender_focus_on = egui::SurrenderFocusOn::Clicks
            });
        }
        keep_in_view(ui, &password, viewport.height);
        let toggled = ui.interact(hit, Id::new("welcome-show-password"), egui::Sense::CLICK);
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
            .on_hover_cursor(egui::CursorIcon::PointingHand)
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
            password.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
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

fn keep_in_view(ui: &egui::Ui, field: &egui::Response, height: f32) {
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

fn field_label(ui: &mut egui::Ui, text: &str) {
    kit::text(text, Type::Label).role(Role::Secondary).show(ui);
    kit::space(ui, Space::S8);
}

fn secondary(ui: &mut egui::Ui, label: &str, width: f32, metrics: UiMetrics) -> egui::Response {
    components::button_with_text_size(
        ui,
        label,
        width,
        button_height(),
        components::ButtonKind::Secondary,
        size::BUTTON_TEXT,
        metrics,
    )
}

fn draw_pairing(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut WelcomeModel,
    wordmark: Option<(egui::TextureId, Vec2)>,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) {
    let qr_side = metrics.welcome_qr.of(viewport.height);
    let card_side = qr_side + 64.0;
    let text_width = metrics.welcome_text.of(viewport.width);
    let gap = 96.0;
    let total = card_side + gap + text_width;
    let inset = ((ui.available_width() - total) * 0.5).max(48.0);
    let wordmark_height = metrics.welcome_wordmark.of(viewport.height);
    ui.add_space(metrics.gutter * 0.5);
    ui.horizontal(|ui| {
        ui.add_space(inset);
        ui.vertical(|ui| {
            ui.set_width(total);
            paint_wordmark(ui, wordmark, total, wordmark_height);
            ui.add_space(20.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                ui.vertical(|ui| {
                    ui.set_width(card_side);
                    let (card, _) = ui.allocate_exact_size(Vec2::splat(card_side), Sense::hover());
                    ui.painter().rect_filled(card, 28.0, Color32::WHITE);
                    if let Some(qr) = model.qr.as_ref().filter(|qr| qr.width > 0) {
                        paint_qr(ui.painter(), card.shrink(32.0), qr);
                    }
                    ui.add_space(28.0);
                    let response =
                        secondary(ui, &t("auth.continue_without_account"), card_side, metrics);
                    if !response.has_focus() {
                        response.request_focus();
                    }
                    ui.painter().rect_stroke(
                        response.rect.expand(metrics.focus_ring_expand),
                        response.rect.height() * 0.5 + metrics.focus_ring_expand,
                        egui::Stroke::new(metrics.focus_ring_width, accent()),
                        egui::StrokeKind::Outside,
                    );
                    let activated =
                        response.clicked() || ui.input(|input| input.key_pressed(egui::Key::Enter));
                    if activated {
                        *request = Some(WelcomeRequest::Offline);
                    }
                });
                ui.vertical(|ui| {
                    ui.set_width(text_width);
                    ui.label(
                        RichText::new(t("auth.welcome_back"))
                            .size(metrics.text.title * 1.6)
                            .strong()
                            .color(text_primary()),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(t("auth.scan_to_connect"))
                            .size(metrics.text.subtitle)
                            .color(metrics.text_secondary),
                    );
                    ui.add_space(16.0);
                    for (index, key) in [
                        "auth.tv.step_scan",
                        "auth.tv.step_login",
                        "auth.tv.step_return",
                    ]
                    .iter()
                    .enumerate()
                    {
                        step(ui, index + 1, &t(key), metrics);
                        ui.add_space(8.0);
                    }
                    if let Some((url, code)) = model.pair_code.as_ref() {
                        ui.label(
                            RichText::new(t("auth.tv.or_enter_code").replacen("%s", url, 1))
                                .size(metrics.text.body)
                                .color(metrics.text_secondary),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(code)
                                .size(metrics.text.title * 1.3)
                                .strong()
                                .color(text_primary()),
                        );
                        ui.add_space(8.0);
                    }
                    let status = match model.error.as_deref() {
                        Some(key) => t(key),
                        None => t("auth.login_pending"),
                    };
                    ui.label(
                        RichText::new(status)
                            .size(metrics.text.body)
                            .color(metrics.text_muted),
                    );
                });
            });
        });
    });
}

fn step(ui: &mut egui::Ui, number: usize, text: &str, metrics: UiMetrics) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        let badge = metrics.text.subtitle * 1.6;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(badge), Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), badge * 0.5, ink(tone::soft()));
        draw_text(
            ui.painter(),
            rect.center(),
            Align2::CENTER_CENTER,
            number.to_string(),
            FontId::proportional(metrics.text.body),
            text_primary(),
            None,
        );
        ui.label(
            RichText::new(text)
                .size(metrics.text.subtitle)
                .color(text_primary()),
        );
    });
}

fn paint_qr(painter: &egui::Painter, rect: Rect, qr: &WelcomeQr) {
    let module = rect.width() / qr.width as f32;
    for (index, dark) in qr.modules.iter().enumerate() {
        if !dark {
            continue;
        }
        let (x, y) = ((index % qr.width) as f32, (index / qr.width) as f32);
        painter.rect_filled(
            Rect::from_min_size(
                rect.min + Vec2::new(x * module, y * module),
                Vec2::splat(module.ceil()),
            ),
            0.0,
            Color32::BLACK,
        );
    }
}
