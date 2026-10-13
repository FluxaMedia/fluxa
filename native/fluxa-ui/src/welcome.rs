use super::*;

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
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if viewport.is_tv() {
                        draw_pairing(ui, viewport, model, wordmark, metrics, &t, &mut request);
                    } else {
                        draw_login(
                            ui,
                            viewport,
                            model,
                            wordmark,
                            eye,
                            metrics,
                            &t,
                            &mut request,
                        );
                    }
                });
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

const FIELD_HEIGHT: f32 = 52.0;

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
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WelcomeRequest>,
) {
    let max_width: f32 = if viewport.is_compact() { 420.0 } else { 400.0 };
    let width = max_width.min(viewport.width - 32.0);
    let inset = ((ui.available_width() - width) * 0.5).max(16.0);
    ui.add_space(((viewport.height - 620.0) * 0.5).max(32.0));
    ui.horizontal(|ui| {
        ui.add_space(inset);
        ui.vertical(|ui| {
            ui.set_width(width);
            paint_wordmark(ui, wordmark, width, 52.0);
            ui.add_space(20.0);
            centered_text(ui, t("auth.welcome_back"), 26.0, text_primary(), true);
            ui.add_space(8.0);
            centered_text(
                ui,
                t("auth.nuvio.subtitle"),
                14.0,
                metrics.text_secondary,
                false,
            );
            ui.add_space(28.0);

            label(ui, &t("auth.field.email"), metrics);
            components::text_field(
                ui,
                &mut model.email,
                &t("auth.placeholder.email"),
                Vec2::new(width, FIELD_HEIGHT),
                14.0,
                components::InputKind::Text,
                metrics,
            );
            ui.add_space(28.0);
            label(ui, &t("auth.field.password"), metrics);
            let kind = if model.show_password {
                components::InputKind::Text
            } else {
                components::InputKind::Password
            };
            let field_top = ui.cursor().min;
            let password = components::text_field(
                ui,
                &mut model.password,
                &t("auth.placeholder.password_login"),
                Vec2::new(width, FIELD_HEIGHT),
                14.0,
                kind,
                metrics,
            );
            let field = Rect::from_min_size(field_top, Vec2::new(width, FIELD_HEIGHT));
            let hit = Rect::from_center_size(
                Pos2::new(field.right() - 28.0, field.center().y),
                Vec2::splat(40.0),
            );
            let toggled = ui.interact(hit, Id::new("welcome-show-password"), egui::Sense::click());
            if let Some(texture) = eye {
                let tint = if toggled.hovered() {
                    metrics.text_primary
                } else {
                    metrics.text_secondary
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
            ui.add_space(4.0);
            if let Some(key) = model.error.as_deref() {
                components::note(ui, &t(key), metrics);
            }
            ui.add_space(43.0);
            let submitted =
                password.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            let label = if model.busy {
                t("auth.nuvio.signing_in")
            } else {
                t("auth.log_in")
            };
            if components::primary_button(ui, &label, width, !model.busy).clicked()
                || (submitted && !model.busy)
            {
                *request = Some(WelcomeRequest::SignIn);
            }
            ui.add_space(20.0);
            divider(ui, width, &t("auth.or"), metrics);
            ui.add_space(20.0);
            if secondary(ui, &t("auth.continue_without_account"), width, metrics).clicked() {
                *request = Some(WelcomeRequest::Offline);
            }
            ui.add_space(48.0);
        });
    });
}

fn label(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.label(
        RichText::new(text)
            .size(13.0)
            .strong()
            .color(metrics.text_secondary),
    );
    ui.add_space(8.0);
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

fn divider(ui: &mut egui::Ui, width: f32, label: &str, metrics: UiMetrics) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 16.0), Sense::hover());
    let painter = ui.painter();
    let galley = painter.layout_no_wrap(
        label.to_owned(),
        FontId::proportional(metrics.text.label),
        metrics.text_muted,
    );
    let half = galley.size().x * 0.5 + 12.0;
    let line = egui::Stroke::new(stroke::HAIRLINE, metrics.border);
    painter.hline(rect.left()..=rect.center().x - half, rect.center().y, line);
    painter.hline(rect.center().x + half..=rect.right(), rect.center().y, line);
    painter.galley(
        rect.center() - galley.size() * 0.5,
        galley,
        metrics.text_muted,
    );
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
    let qr_side = (viewport.height * 0.34).clamp(240.0, 420.0);
    let card_side = qr_side + 64.0;
    let text_width = (viewport.width * 0.32).clamp(420.0, 680.0);
    let gap = 96.0;
    let total = card_side + gap + text_width;
    let inset = ((ui.available_width() - total) * 0.5).max(48.0);
    let block_height = card_side + 260.0;
    ui.add_space(((viewport.height - block_height) * 0.5).max(48.0));
    ui.horizontal(|ui| {
        ui.add_space(inset);
        ui.vertical(|ui| {
            ui.set_width(total);
            paint_wordmark(ui, wordmark, total, 88.0);
            ui.add_space(48.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let (card, _) = ui.allocate_exact_size(Vec2::splat(card_side), Sense::hover());
                ui.painter().rect_filled(card, 28.0, Color32::WHITE);
                if let Some(qr) = model.qr.as_ref().filter(|qr| qr.width > 0) {
                    paint_qr(ui.painter(), card.shrink(32.0), qr);
                }
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
                    ui.add_space(28.0);
                    for (index, key) in [
                        "auth.tv.step_scan",
                        "auth.tv.step_login",
                        "auth.tv.step_return",
                    ]
                    .iter()
                    .enumerate()
                    {
                        step(ui, index + 1, &t(key), metrics);
                        ui.add_space(16.0);
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
                        ui.add_space(20.0);
                    }
                    ui.add_space(8.0);
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
            ui.add_space(40.0);
            ui.horizontal(|ui| {
                let button = (total * 0.4).clamp(320.0, 520.0);
                ui.add_space((total - button) * 0.5);
                let response = secondary(ui, &t("auth.continue_without_account"), button, metrics);
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
        });
    });
}

fn step(ui: &mut egui::Ui, number: usize, text: &str, metrics: UiMetrics) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        let badge = metrics.text.subtitle * 1.6;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(badge), Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), badge * 0.5, ink(tone::SOFT));
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
