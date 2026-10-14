use super::*;

#[derive(Clone, Debug, Default)]
pub struct WatchMemberView {
    pub name: String,
    pub avatar: Option<String>,
    pub ready: bool,
    pub buffering: bool,
    pub host: bool,
    pub you: bool,
}

#[derive(Clone, Debug, Default)]
pub struct WatchTogetherModel {
    pub language: String,
    pub code_input: String,
    pub room_code: Option<String>,
    pub members: Vec<WatchMemberView>,
    pub connecting: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchTogetherRequest {
    Create,
    Join(String),
    Leave,
    Close,
    CopyCode,
}

pub fn waiting_member(members: &[WatchMemberView]) -> Option<&WatchMemberView> {
    members.iter().find(|member| member.buffering)
}

pub fn draw_watch_together(
    context: &egui::Context,
    viewport: Viewport,
    model: &mut WatchTogetherModel,
    assets: &mut impl HomeAssets,
) -> Option<WatchTogetherRequest> {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let metrics = metrics_for_assets(viewport, assets);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    components::modal(
        context,
        screen,
        "fluxa-watch-together",
        |ui| {
            ui.label(
                RichText::new(t("watch.title"))
                    .size(metrics.text.subtitle)
                    .strong()
                    .color(text_primary()),
            );
            ui.add_space(14.0);
            let width = ui.available_width();
            match model.room_code.clone() {
                Some(code) => draw_room(ui, model, &code, width, assets, metrics, &t, &mut request),
                None => draw_entry(ui, model, width, metrics, &t, &mut request),
            }
            if let Some(error) = model.error.as_ref() {
                components::note(ui, &t(error), metrics);
            }
            ui.add_space(14.0);
            if components::secondary_button(ui, &t("common.close"), width).clicked()
                || ui.input(|input| input.key_pressed(egui::Key::Escape))
            {
                request = Some(WatchTogetherRequest::Close);
            }
        },
        metrics,
    );
    request
}

fn draw_entry(
    ui: &mut egui::Ui,
    model: &mut WatchTogetherModel,
    width: f32,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    if model.connecting {
        components::note(ui, &t("watch.connecting"), metrics);
        return;
    }
    if components::primary_button(ui, &t("watch.start"), width, true).clicked() {
        *request = Some(WatchTogetherRequest::Create);
    }
    ui.add_space(16.0);
    components::note(ui, &t("watch.join_hint"), metrics);
    ui.add_space(8.0);
    let field = components::text_input(
        ui,
        &mut model.code_input,
        &t("watch.code_hint"),
        width,
        components::InputKind::Text,
        metrics,
    );
    model.code_input = model
        .code_input
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(6)
        .collect::<String>()
        .to_uppercase();
    ui.add_space(8.0);
    let complete = model.code_input.len() == 6;
    let submit =
        complete && field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    if components::secondary_button(ui, &t("watch.join"), width).clicked() && complete || submit {
        *request = Some(WatchTogetherRequest::Join(model.code_input.clone()));
    }
}

fn draw_room(
    ui: &mut egui::Ui,
    model: &WatchTogetherModel,
    code: &str,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(code)
                .size(metrics.text.subtitle)
                .strong()
                .color(accent()),
        );
        if components::text_button(ui, &t("watch.copy_code"), metrics.text.meta, 170).clicked() {
            *request = Some(WatchTogetherRequest::CopyCode);
        }
    });
    let ready = model.members.iter().filter(|member| member.ready).count();
    components::note(
        ui,
        &t("watch.ready_count")
            .replace("{ready}", &ready.to_string())
            .replace("{total}", &model.members.len().to_string()),
        metrics,
    );
    ui.add_space(12.0);
    for member in &model.members {
        draw_member(ui, member, width, assets, metrics, t);
    }
    if let Some(member) = waiting_member(&model.members) {
        ui.add_space(8.0);
        ui.label(
            RichText::new(t("watch.waiting_for").replace("{name}", &member.name))
                .size(metrics.text.meta)
                .color(accent()),
        );
    } else {
        components::note(ui, &t("watch.hint"), metrics);
    }
    ui.add_space(14.0);
    if components::secondary_button(ui, &t("watch.leave"), width).clicked() {
        *request = Some(WatchTogetherRequest::Leave);
    }
}

fn draw_member(
    ui: &mut egui::Ui,
    member: &WatchMemberView,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 44.0), Sense::hover());
    components::avatar(
        ui.painter(),
        assets,
        Pos2::new(rect.left() + 18.0, rect.center().y),
        18.0,
        &member.name,
        member.avatar.as_deref(),
        metrics,
    );
    let mut name = member.name.clone();
    if member.host {
        name = format!("{name} · {}", t("watch.host"));
    }
    if member.you {
        name = format!("{name} · {}", t("watch.you"));
    }
    ui.painter().text(
        Pos2::new(rect.left() + 46.0, rect.center().y),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(metrics.text.meta),
        text_primary(),
    );
    let (status, color) = if member.buffering {
        (t("watch.buffering"), accent())
    } else if member.ready {
        (t("watch.ready"), accent())
    } else {
        (t("watch.waiting"), metrics.text_muted)
    };
    let dot = Pos2::new(rect.right() - 6.0, rect.center().y);
    if member.ready && !member.buffering {
        ui.painter().circle_filled(dot, 5.0, color);
    } else {
        ui.painter()
            .circle_stroke(dot, 4.5, egui::Stroke::new(1.5, color));
    }
    ui.painter().text(
        Pos2::new(rect.right() - 20.0, rect.center().y),
        Align2::RIGHT_CENTER,
        status,
        FontId::proportional(metrics.text.label),
        color,
    );
}
