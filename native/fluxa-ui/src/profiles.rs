use super::*;

#[derive(Clone, Debug, Default)]
pub struct ProfileEntry {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub locked: bool,
    pub uses_primary_addons: bool,
    pub uses_primary_plugins: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ProfileAvatar {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAvatarPack {
    pub id: String,
    pub repository_url: String,
    pub title: String,
    pub manifest_url: String,
    pub avatars: Vec<ProfileAvatar>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProfilePickerSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_url: Option<String>,
    pub avatar_packs: Vec<ProfileAvatarPack>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProfilesMode {
    #[default]
    Select,
    Form,
    Settings,
}

#[derive(Clone, Debug, Default)]
pub struct ProfileDraft {
    pub editing: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub pin: String,
    pub has_pin: bool,
    pub remove_pin: bool,
    pub uses_primary_addons: bool,
    pub uses_primary_plugins: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinPurpose {
    Enter,
    Edit,
    Delete,
}

#[derive(Clone, Debug)]
pub struct PinPrompt {
    pub profile_id: String,
    pub purpose: PinPurpose,
    pub pin: String,
    pub error: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ProfilesModel {
    pub profiles: Vec<ProfileEntry>,
    pub primary_id: Option<String>,
    pub mode: ProfilesMode,
    pub draft: ProfileDraft,
    pub pin_prompt: Option<PinPrompt>,
    pub confirm_delete: Option<String>,
    pub picker: ProfilePickerSettings,
    pub background_input: String,
    pub repository_input: String,
    pub busy: bool,
    pub notice: Option<String>,
    pub language: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProfilesRequest {
    Select(String),
    SubmitPin,
    Save,
    Delete(String),
    AddRepository,
    RefreshRepository(String),
    RefreshAllPacks,
    RemovePack(String),
    SaveBackground,
    PickAvatarImage,
    PickBackgroundImage,
}

impl ProfilesModel {
    pub fn profile(&self, id: &str) -> Option<&ProfileEntry> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn open_form(&mut self, profile: Option<&ProfileEntry>) {
        self.draft = match profile {
            Some(profile) => ProfileDraft {
                editing: Some(profile.id.clone()),
                name: profile.name.clone(),
                avatar_url: profile.avatar_url.clone(),
                has_pin: profile.locked,
                uses_primary_addons: profile.uses_primary_addons,
                uses_primary_plugins: profile.uses_primary_plugins,
                ..ProfileDraft::default()
            },
            None => ProfileDraft::default(),
        };
        self.mode = ProfilesMode::Form;
    }

    fn duplicate_name(&self) -> bool {
        let name = self.draft.name.trim().to_lowercase();
        !name.is_empty()
            && self.profiles.iter().any(|profile| {
                Some(&profile.id) != self.draft.editing.as_ref()
                    && profile.name.trim().to_lowercase() == name
            })
    }

    fn draft_is_primary(&self) -> bool {
        match &self.draft.editing {
            Some(id) => self.primary_id.as_ref() == Some(id),
            None => self.profiles.is_empty(),
        }
    }

    pub fn can_save(&self) -> bool {
        !self.draft.name.trim().is_empty()
            && !self.duplicate_name()
            && !self.busy
            && (self.draft.pin.is_empty() || self.draft.pin.len() == 4)
    }
}

pub fn draw_profiles(
    context: &egui::Context,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_backdrop(context, &painter, screen, assets);
    let metrics = metrics_for_assets(viewport, assets);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let blocked = model.pin_prompt.is_some() || model.confirm_delete.is_some();

    egui::Area::new(Id::new("fluxa-profiles"))
        .fixed_pos(Pos2::ZERO)
        .interactable(!blocked)
        .show(context, |ui| {
            ui.set_min_size(screen.size());
            ui.set_max_size(screen.size());
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match model.mode {
                    ProfilesMode::Select => {
                        draw_select(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                    ProfilesMode::Form => {
                        draw_form(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                    ProfilesMode::Settings => {
                        draw_picker_settings(ui, viewport, model, assets, metrics, &t, &mut request)
                    }
                });
        });

    if model.pin_prompt.is_some() {
        draw_pin_prompt(context, screen, model, assets, metrics, &t, &mut request);
    } else if let Some(id) = model.confirm_delete.clone() {
        let name = model
            .profile(&id)
            .map(|profile| profile.name.clone())
            .unwrap_or_default();
        modal(context, screen, "fluxa-profiles-confirm", |ui| {
            ui.label(
                RichText::new(t("profiles.delete_confirm_title"))
                    .size(22.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(t("profiles.delete_confirm_body").replacen("%s", &name, 1))
                    .size(14.0)
                    .color(Color32::from_white_alpha(170)),
            );
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if secondary_button(ui, &t("common.cancel"), 140.0).clicked() {
                    model.confirm_delete = None;
                }
                if danger_button(ui, &t("profiles.delete"), 140.0).clicked() {
                    model.confirm_delete = None;
                    request = Some(ProfilesRequest::Delete(id.clone()));
                }
            });
        });
    }

    if let Some(notice) = model.notice.clone() {
        egui::Area::new(Id::new("fluxa-profiles-notice"))
            .fixed_pos(Pos2::new(screen.right() - 376.0, 24.0))
            .order(egui::Order::Tooltip)
            .show(context, |ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(28, 28, 28))
                    .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(28)))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.set_width(320.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(notice)
                                        .size(14.0)
                                        .color(Color32::from_white_alpha(220)),
                                )
                                .wrap(),
                            );
                            if icon_button(ui, assets, "Close", 22.0, Color32::from_white_alpha(170), false, true)
                                .clicked()
                            {
                                model.notice = None;
                            }
                        });
                    });
            });
    }

    let _ = metrics;
    HomeLayout {
        profiles: request,
        ..HomeLayout::default()
    }
}

fn paint_backdrop(
    context: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    paint_ambient(painter, screen, assets);
    let url = assets.custom_background_url().map(ToOwned::to_owned);
    let size = artwork_target_size(screen.size(), context.pixels_per_point());
    if components::artwork_image(
        painter,
        screen,
        url.as_deref(),
        size,
        ArtworkPriority::Hero,
        Color32::WHITE,
        assets,
    ) {
        painter.rect_filled(screen, 0.0, Color32::from_rgba_unmultiplied(12, 12, 12, 210));
    }
}

pub(crate) fn paint_custom_background(
    context: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    paint_backdrop(context, painter, screen, assets);
}

fn centered_column(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let available = ui.available_width();
    let inset = ((available - width) * 0.5).max(16.0);
    ui.horizontal(|ui| {
        ui.add_space(inset);
        ui.vertical(|ui| {
            ui.set_width(width.min(available - 32.0));
            add(ui);
        });
    });
}

fn heading(ui: &mut egui::Ui, eyebrow: &str, title: &str, subtitle: &str) {
    ui.label(
        RichText::new(eyebrow.to_uppercase())
            .size(11.0)
            .color(Color32::from_white_alpha(110)),
    );
    ui.add_space(10.0);
    ui.label(RichText::new(title).size(36.0).strong().color(Color32::WHITE));
    ui.add_space(8.0);
    ui.label(
        RichText::new(subtitle)
            .size(14.0)
            .color(Color32::from_white_alpha(120)),
    );
    ui.add_space(28.0);
}

fn draw_select(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    _metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    let radius = 65.0;
    let cell = Vec2::new(150.0, 222.0);
    let gap = 40.0;
    let count = model.profiles.len() + 1;
    let per_row = (((viewport.width - 80.0 + gap) / (cell.x + gap)).floor() as usize).clamp(1, count);
    let rows = count.div_ceil(per_row);
    let grid_height = rows as f32 * cell.y + (rows - 1) as f32 * gap;
    let content_height = 60.0 + 24.0 + 46.0 + 52.0 + grid_height;
    let top = ((viewport.height - content_height) * 0.5).max(48.0);
    ui.add_space(top);

    let width = ui.available_width();
    let brand_font = FontId::proportional(32.0);
    let brand = ui
        .painter()
        .layout_no_wrap("fluxa".to_owned(), brand_font, Color32::WHITE);
    let mark = 40.0;
    let brand_width = mark + 10.0 + brand.size().x;
    let (brand_rect, _) = ui.allocate_exact_size(Vec2::new(width, 60.0), Sense::hover());
    let left = brand_rect.center().x - brand_width * 0.5;
    if let Some(texture) = assets.brand_mark() {
        ui.painter().image(
            texture,
            Rect::from_center_size(Pos2::new(left + mark * 0.5, brand_rect.center().y), Vec2::splat(mark)),
            full_uv(),
            Color32::WHITE,
        );
    }
    ui.painter().galley(
        Pos2::new(left + mark + 10.0, brand_rect.center().y - brand.size().y * 0.5),
        brand,
        Color32::WHITE,
    );
    ui.add_space(24.0);

    let title = ui.painter().layout_no_wrap(
        t("profiles.who_watching"),
        FontId::proportional(38.0),
        Color32::WHITE,
    );
    let (title_rect, _) = ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::hover());
    let title_pos = Pos2::new(
        title_rect.center().x - title.size().x * 0.5,
        title_rect.center().y - title.size().y * 0.5,
    );
    let title_right = title_pos.x + title.size().x;
    ui.painter().galley(title_pos, title, Color32::WHITE);
    let gear_rect = Rect::from_center_size(
        Pos2::new(title_right + 38.0, title_rect.center().y),
        Vec2::splat(44.0),
    );
    let gear = ui
        .interact(gear_rect, Id::new("fluxa-profiles-gear"), Sense::click())
        .on_hover_text(t("profiles.picker_settings"));
    if let Some(icon) = assets.icon("Settings") {
        ui.painter().image(
            icon,
            Rect::from_center_size(gear_rect.center(), Vec2::splat(26.0)),
            full_uv(),
            Color32::from_white_alpha(if gear.hovered() { 255 } else { 180 }),
        );
    }
    if gear.clicked() {
        model.background_input = model.picker.background_url.clone().unwrap_or_default();
        model.mode = ProfilesMode::Settings;
    }
    ui.add_space(52.0);

    let (grid_rect, _) = ui.allocate_exact_size(Vec2::new(width, grid_height), Sense::hover());
    let profiles = model.profiles.clone();
    for index in 0..count {
        let row = index / per_row;
        let column = index % per_row;
        let in_row = (count - row * per_row).min(per_row);
        let row_width = in_row as f32 * cell.x + (in_row - 1) as f32 * gap;
        let origin = Pos2::new(
            grid_rect.center().x - row_width * 0.5 + column as f32 * (cell.x + gap),
            grid_rect.top() + row as f32 * (cell.y + gap),
        );
        let avatar_center = Pos2::new(origin.x + cell.x * 0.5, origin.y + radius);
        let hit = Rect::from_center_size(avatar_center, Vec2::splat(radius * 2.0))
            .union(Rect::from_min_size(
                Pos2::new(origin.x, avatar_center.y + radius),
                Vec2::new(cell.x, 40.0),
            ));
        let Some(profile) = profiles.get(index) else {
            let response = ui.interact(hit, Id::new("fluxa-profiles-add"), Sense::click());
            let r = if response.hovered() { radius * 1.04 } else { radius };
            ui.painter().circle(
                avatar_center,
                r,
                Color32::from_white_alpha(if response.hovered() { 26 } else { 10 }),
                egui::Stroke::new(1.0, Color32::from_white_alpha(26)),
            );
            if let Some(icon) = assets.icon("Plus") {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(avatar_center, Vec2::splat(36.0)),
                    full_uv(),
                    Color32::from_white_alpha(140),
                );
            }
            ui.painter().text(
                Pos2::new(avatar_center.x, avatar_center.y + radius + 24.0),
                Align2::CENTER_CENTER,
                t("profiles.add_profile"),
                FontId::proportional(14.0),
                Color32::from_white_alpha(110),
            );
            if response.clicked() {
                model.open_form(None);
            }
            continue;
        };
        let response = ui.interact(hit, Id::new(("fluxa-profile", &profile.id)), Sense::click());
        let card_hovered = ui.rect_contains_pointer(Rect::from_min_size(origin, cell));
        let r = if response.hovered() { radius * 1.04 } else { radius };
        components::avatar(ui.painter(), assets, avatar_center, r, &profile.name, profile.avatar_url.as_deref());
        if !response.hovered() {
            ui.painter().circle_filled(avatar_center, r, Color32::from_black_alpha(30));
        }
        if profile.locked {
            let badge = avatar_center + Vec2::splat(radius * 0.72);
            ui.painter().circle(
                badge,
                17.0,
                Color32::from_rgba_unmultiplied(12, 12, 12, 235),
                egui::Stroke::new(1.0, Color32::from_white_alpha(38)),
            );
            if let Some(icon) = assets.icon("Lock") {
                ui.painter().image(
                    icon,
                    Rect::from_center_size(badge, Vec2::splat(16.0)),
                    full_uv(),
                    Color32::from_white_alpha(220),
                );
            }
        }
        let name_font = FontId::proportional(16.0);
        let name = truncate_to_width(ui.painter(), &profile.name, &name_font, cell.x);
        let name_y = avatar_center.y + radius + 24.0;
        ui.painter().text(
            Pos2::new(avatar_center.x, name_y),
            Align2::CENTER_CENTER,
            name,
            name_font,
            Color32::from_white_alpha(if response.hovered() { 255 } else { 215 }),
        );
        let mut actions_y = name_y + 24.0;
        if model.primary_id.as_ref() == Some(&profile.id) {
            let label = t("profiles.primary_badge").to_uppercase();
            let galley = ui.painter().layout_no_wrap(
                label,
                FontId::proportional(10.0),
                Color32::from_white_alpha(150),
            );
            let badge = Rect::from_center_size(
                Pos2::new(avatar_center.x, name_y + 22.0),
                galley.size() + Vec2::new(14.0, 6.0),
            );
            ui.painter().rect_filled(badge, 4.0, Color32::from_white_alpha(24));
            ui.painter().galley(badge.center() - galley.size() * 0.5, galley, Color32::WHITE);
            actions_y += 20.0;
        }
        let alpha = if card_hovered { 230 } else { 120 };
        for (offset, icon, label, is_delete) in [
            (-16.0, "Edit", t("profiles.edit"), false),
            (16.0, "Delete", t("profiles.delete"), true),
        ] {
            let rect = Rect::from_center_size(
                Pos2::new(avatar_center.x + offset, actions_y + 6.0),
                Vec2::splat(28.0),
            );
            let action = ui
                .interact(rect, Id::new(("fluxa-profile-action", &profile.id, is_delete)), Sense::click())
                .on_hover_text(label);
            if action.hovered() {
                ui.painter().rect_filled(rect, 6.0, Color32::from_white_alpha(18));
            }
            let color = Color32::from_white_alpha(if action.hovered() { 255 } else { alpha });
            if let Some(texture) = assets.icon(icon) {
                ui.painter().image(
                    texture,
                    Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                    full_uv(),
                    color,
                );
            }
            if action.clicked() {
                let purpose = if is_delete { PinPurpose::Delete } else { PinPurpose::Edit };
                if profile.locked {
                    model.pin_prompt = Some(PinPrompt {
                        profile_id: profile.id.clone(),
                        purpose,
                        pin: String::new(),
                        error: false,
                    });
                } else if is_delete {
                    model.confirm_delete = Some(profile.id.clone());
                } else {
                    model.open_form(Some(profile));
                }
            }
        }
        if response.clicked() {
            if profile.locked {
                model.pin_prompt = Some(PinPrompt {
                    profile_id: profile.id.clone(),
                    purpose: PinPurpose::Enter,
                    pin: String::new(),
                    error: false,
                });
            } else {
                *request = Some(ProfilesRequest::Select(profile.id.clone()));
            }
        }
    }
    ui.add_space(48.0);
}

fn close_button(
    ui: &mut egui::Ui,
    model: &mut ProfilesModel,
    assets: &impl HomeAssets,
    t: &dyn Fn(&str) -> String,
) {
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        let response = icon_button(ui, assets, "Close", 36.0, Color32::from_white_alpha(190), true, true)
            .on_hover_text(t("common.close"));
        if response.clicked() {
            model.mode = ProfilesMode::Select;
        }
    });
    ui.add_space(24.0);
}

fn icon_button(
    ui: &mut egui::Ui,
    assets: &impl HomeAssets,
    icon: &str,
    size: f32,
    color: Color32,
    framed: bool,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::splat(size),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    if framed {
        ui.painter().rect(
            rect,
            8.0,
            Color32::from_white_alpha(if response.hovered() { 24 } else { 14 }),
            egui::Stroke::new(1.0, Color32::from_white_alpha(26)),
            egui::StrokeKind::Inside,
        );
    } else if response.hovered() {
        ui.painter().rect_filled(rect, 6.0, Color32::from_white_alpha(18));
    }
    let tint = if !enabled {
        color.gamma_multiply(0.4)
    } else if response.hovered() {
        Color32::WHITE
    } else {
        color
    };
    if let Some(texture) = assets.icon(icon) {
        ui.painter().image(
            texture,
            Rect::from_center_size(rect.center(), Vec2::splat((size * 0.45).clamp(14.0, 22.0))),
            full_uv(),
            tint,
        );
    }
    response
}

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.0)
            .color(Color32::from_white_alpha(100)),
    );
    ui.add_space(6.0);
}

fn note(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).size(12.0).color(Color32::from_white_alpha(170)));
}

fn text_field(ui: &mut egui::Ui, value: &mut String, hint: &str, width: f32, password: bool) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 44.0), Sense::hover());
    let response = ui.put(
        rect.shrink2(Vec2::new(13.0, 0.0)),
        egui::TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .password(password)
            .font(FontId::proportional(14.0))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(Color32::from_white_alpha(90)))
            .text_color(Color32::WHITE)
            .margin(Vec2::ZERO),
    );
    ui.painter().rect_filled(rect, 8.0, Color32::from_white_alpha(10));
    ui.painter().rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(
            1.0,
            Color32::from_white_alpha(if response.has_focus() { 110 } else { 26 }),
        ),
        egui::StrokeKind::Inside,
    );
    response
}

fn secondary_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    ui.add_sized(
        [width, 44.0],
        egui::Button::new(RichText::new(label).size(13.0).color(Color32::from_white_alpha(200)))
            .fill(Color32::from_white_alpha(14))
            .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(26)))
            .corner_radius(8.0),
    )
}

fn primary_button(ui: &mut egui::Ui, label: &str, width: f32, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(
            RichText::new(label)
                .size(13.0)
                .strong()
                .color(if enabled { Color32::BLACK } else { Color32::from_white_alpha(80) }),
        )
        .fill(if enabled { Color32::WHITE } else { Color32::from_white_alpha(24) })
        .corner_radius(8.0)
        .min_size(Vec2::new(width, 44.0)),
    )
}

fn danger_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    ui.add_sized(
        [width, 44.0],
        egui::Button::new(RichText::new(label).size(13.0).strong().color(Color32::WHITE))
            .fill(Color32::from_rgb(170, 48, 48))
            .corner_radius(8.0),
    )
}

fn panel(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(20, 20, 20, 235))
        .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(20)))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(22))
        .show(ui, add);
}

fn draw_form(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    _metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    close_button(ui, model, assets, t);
    let width = (viewport.width - 56.0).min(1100.0);
    let stacked = width < 760.0;
    centered_column(ui, width, |ui| {
        heading(
            ui,
            &t("profiles.settings"),
            &if model.draft.editing.is_some() {
                t("profiles.edit")
            } else {
                t("profiles.create_new")
            },
            &t("profiles.form_subtitle"),
        );
        let left_width = if stacked { width } else { (width * 0.38).max(280.0) };
        let right_width = if stacked { width } else { width - left_width - 14.0 };
        if stacked {
            form_details(ui, model, assets, t, request, left_width);
            ui.add_space(14.0);
            form_images(ui, model, assets, t, request, right_width);
        } else {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                ui.vertical(|ui| form_details(ui, model, assets, t, request, left_width));
                ui.vertical(|ui| form_images(ui, model, assets, t, request, right_width));
            });
        }
        ui.add_space(48.0);
    });
}


fn form_details(
    ui: &mut egui::Ui,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
    width: f32,
) {
            panel(ui, |ui| {
                ui.set_width(width - 44.0);
                ui.vertical_centered(|ui| {
                    let (rect, response) = ui.allocate_exact_size(Vec2::splat(128.0), Sense::click());
                    let name = if model.draft.name.trim().is_empty() {
                        t("auto.profile")
                    } else {
                        model.draft.name.trim().to_owned()
                    };
                    components::avatar(
                        ui.painter(),
                        assets,
                        rect.center(),
                        64.0,
                        &name,
                        model.draft.avatar_url.as_deref(),
                    );
                    if response.hovered() {
                        ui.painter().circle_filled(rect.center(), 64.0, Color32::from_black_alpha(90));
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            t("profiles.change_image"),
                            FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
                    if response.on_hover_text(t("profiles.choose_image")).clicked() {
                        *request = Some(ProfilesRequest::PickAvatarImage);
                    }
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&name).size(18.0).strong().color(Color32::WHITE));
                        if model.draft_is_primary() {
                            ui.label(
                                RichText::new(t("profiles.primary_badge").to_uppercase())
                                    .size(10.0)
                                    .color(Color32::from_white_alpha(160))
                                    .background_color(Color32::from_white_alpha(24)),
                            );
                        }
                    });
                    if model.draft.avatar_url.is_some()
                        && ui
                            .add(
                                egui::Button::new(
                                    RichText::new(t("profiles.use_initials"))
                                        .size(12.0)
                                        .color(Color32::from_white_alpha(110)),
                                )
                                .frame(false),
                            )
                            .clicked()
                    {
                        model.draft.avatar_url = None;
                    }
                });
                ui.add_space(20.0);
                let field_width = ui.available_width();
                field_label(ui, &t("profiles.name"));
                text_field(ui, &mut model.draft.name, &t("profiles.name_placeholder"), field_width, false);
                if model.duplicate_name() {
                    note(ui, &t("profiles.duplicate_name"));
                }
                ui.add_space(14.0);
                field_label(ui, &t("profiles.pin_lock"));
                let hint = if model.draft.has_pin && !model.draft.remove_pin {
                    t("profiles.pin_set_placeholder")
                } else {
                    t("profiles.pin_placeholder")
                };
                if text_field(ui, &mut model.draft.pin, &hint, field_width, true).changed() {
                    model.draft.pin.retain(|character| character.is_ascii_digit());
                    model.draft.pin.truncate(4);
                    model.draft.remove_pin = false;
                }
                if !model.draft.pin.is_empty() && model.draft.pin.len() != 4 {
                    note(ui, &t("profiles.pin_invalid"));
                }
                if model.draft.has_pin && !model.draft.remove_pin {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(t("profiles.remove_pin"))
                                    .size(12.0)
                                    .color(Color32::from_white_alpha(120)),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        model.draft.remove_pin = true;
                        model.draft.pin.clear();
                    }
                }
                if model.draft.remove_pin {
                    note(ui, &t("profiles.pin_will_be_removed"));
                }
                if !model.draft_is_primary() {
                    ui.add_space(14.0);
                    field_label(ui, &t("profiles.sharing"));
                    ui.checkbox(
                        &mut model.draft.uses_primary_addons,
                        RichText::new(t("profiles.use_primary_addons")).size(13.0),
                    );
                    ui.checkbox(
                        &mut model.draft.uses_primary_plugins,
                        RichText::new(t("profiles.use_primary_plugins")).size(13.0),
                    );
                }
                ui.add_space(20.0);
                let half = (field_width - 8.0) * 0.5;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if secondary_button(ui, &t("common.cancel"), half).clicked() {
                        model.mode = ProfilesMode::Select;
                    }
                    let label = if model.busy {
                        t("common.saving")
                    } else if model.draft.editing.is_some() {
                        t("profiles.save")
                    } else {
                        t("profiles.create")
                    };
                    if primary_button(ui, &label, half, model.can_save()).clicked() {
                        *request = Some(ProfilesRequest::Save);
                    }
                });
            });
        }

fn form_images(
    ui: &mut egui::Ui,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
    width: f32,
) {
            panel(ui, |ui| {
                ui.set_width(width - 44.0);
                let inner = ui.available_width();
                if secondary_button(ui, &t("profiles.choose_image"), inner).clicked() {
                    *request = Some(ProfilesRequest::PickAvatarImage);
                }
                if model.picker.avatar_packs.is_empty() {
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new(t("profiles.no_avatar_packs"))
                            .size(13.0)
                            .color(Color32::from_white_alpha(110)),
                    );
                    return;
                }
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    field_label(ui, &t("profiles.avatar_packs"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let refresh = icon_button(ui, assets, "Refresh", 24.0, Color32::from_white_alpha(150), false, !model.busy)
                            .on_hover_text(t("profiles.refresh_pack"));
                        if refresh.clicked() {
                            *request = Some(ProfilesRequest::RefreshAllPacks);
                        }
                    });
                });
                let tile = 64.0;
                let gap = 10.0;
                let per_row = (((inner + gap) / (tile + gap)).floor() as usize).max(1);
                for pack in model.picker.avatar_packs.clone() {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(&pack.title)
                            .size(13.0)
                            .strong()
                            .color(Color32::from_white_alpha(210)),
                    );
                    ui.add_space(8.0);
                    for chunk in pack.avatars.chunks(per_row) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            for avatar in chunk {
                                let (rect, response) =
                                    ui.allocate_exact_size(Vec2::new(tile, tile + 20.0), Sense::click());
                                let center = Pos2::new(rect.center().x, rect.top() + tile * 0.5);
                                components::avatar(ui.painter(), assets, center, tile * 0.5, &avatar.name, Some(&avatar.url));
                                let selected = model.draft.avatar_url.as_deref() == Some(avatar.url.as_str());
                                if selected || response.hovered() {
                                    ui.painter().circle_stroke(
                                        center,
                                        tile * 0.5 + 2.0,
                                        egui::Stroke::new(
                                            2.0,
                                            Color32::from_white_alpha(if selected { 255 } else { 90 }),
                                        ),
                                    );
                                }
                                let font = FontId::proportional(11.0);
                                ui.painter().text(
                                    Pos2::new(center.x, rect.bottom() - 6.0),
                                    Align2::CENTER_CENTER,
                                    truncate_to_width(ui.painter(), &avatar.name, &font, tile),
                                    font,
                                    Color32::from_white_alpha(170),
                                );
                                if response.on_hover_text(&avatar.name).clicked() {
                                    model.draft.avatar_url = Some(avatar.url.clone());
                                }
                            }
                        });
                        ui.add_space(gap);
                    }
                }
            });
        }

fn draw_picker_settings(
    ui: &mut egui::Ui,
    viewport: Viewport,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    _metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    close_button(ui, model, assets, t);
    let width = (viewport.width - 56.0).min(1024.0);
    centered_column(ui, width, |ui| {
        heading(
            ui,
            &t("profiles.settings"),
            &t("profiles.picker_settings"),
            &t("profiles.picker_background_desc"),
        );
        panel(ui, |ui| {
            ui.set_width(width - 44.0);
            ui.label(RichText::new(t("profiles.picker_background")).size(19.0).strong().color(Color32::WHITE));
            ui.add_space(16.0);
            let has_background = model.picker.background_url.is_some();
            let buttons = if has_background { 2.0 } else { 1.0 };
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let field_width = ui.available_width() - buttons * 54.0;
                let field = text_field(
                    ui,
                    &mut model.background_input,
                    &t("profiles.picker_background_placeholder"),
                    field_width,
                    false,
                );
                if field.lost_focus() {
                    *request = Some(ProfilesRequest::SaveBackground);
                }
                if icon_button(ui, assets, "ImagePlus", 44.0, Color32::from_white_alpha(200), true, true)
                    .on_hover_text(t("profiles.choose_image"))
                    .clicked()
                {
                    *request = Some(ProfilesRequest::PickBackgroundImage);
                }
                if has_background
                    && icon_button(ui, assets, "Delete", 44.0, Color32::from_white_alpha(200), true, true)
                        .on_hover_text(t("profiles.clear_background"))
                        .clicked()
                {
                    model.background_input.clear();
                    *request = Some(ProfilesRequest::SaveBackground);
                }
            });
        });
        ui.add_space(20.0);
        panel(ui, |ui| {
            ui.set_width(width - 44.0);
            ui.label(RichText::new(t("profiles.avatar_packs")).size(19.0).strong().color(Color32::WHITE));
            ui.add_space(6.0);
            ui.label(
                RichText::new(t("profiles.avatar_packs_desc"))
                    .size(14.0)
                    .color(Color32::from_white_alpha(120)),
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let field_width = ui.available_width() - 150.0;
                let field = text_field(
                    ui,
                    &mut model.repository_input,
                    &t("profiles.avatar_pack_repository_placeholder"),
                    field_width,
                    false,
                );
                let submit = field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                let label = if model.busy { t("common.loading") } else { t("profiles.add_pack") };
                let can_add = !model.repository_input.trim().is_empty() && !model.busy;
                if (primary_button(ui, &label, 140.0, can_add).clicked() || submit) && can_add {
                    *request = Some(ProfilesRequest::AddRepository);
                }
            });
            ui.add_space(18.0);
            if model.picker.avatar_packs.is_empty() {
                ui.label(
                    RichText::new(t("profiles.no_avatar_packs"))
                        .size(14.0)
                        .color(Color32::from_white_alpha(110)),
                );
            }
            for pack in model.picker.avatar_packs.clone() {
                egui::Frame::NONE
                    .fill(Color32::from_white_alpha(8))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(14, 12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let previews = pack.avatars.len().min(6);
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(24.0 + previews as f32 * 32.0, 48.0),
                                Sense::hover(),
                            );
                            for (index, avatar) in pack.avatars.iter().take(previews).enumerate() {
                                let center = Pos2::new(rect.left() + 24.0 + index as f32 * 32.0, rect.center().y);
                                ui.painter().circle_filled(center, 24.5, Color32::from_rgb(20, 20, 20));
                                components::avatar(ui.painter(), assets, center, 23.0, &avatar.name, Some(&avatar.url));
                            }
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    pack.title,
                                    t("profiles.avatar_pack_count").replacen("%s", &pack.avatars.len().to_string(), 1)
                                ))
                                .size(14.0)
                                .color(Color32::from_white_alpha(220)),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if icon_button(ui, assets, "Delete", 32.0, Color32::from_rgb(211, 74, 74), false, true)
                                    .on_hover_text(t("profiles.remove_pack"))
                                    .clicked()
                                {
                                    *request = Some(ProfilesRequest::RemovePack(pack.id.clone()));
                                }
                                if icon_button(ui, assets, "Refresh", 32.0, Color32::from_white_alpha(170), false, !model.busy)
                                    .on_hover_text(t("profiles.refresh_pack"))
                                    .clicked()
                                {
                                    *request = Some(ProfilesRequest::RefreshRepository(pack.repository_url.clone()));
                                }
                            });
                        });
                    });
                ui.add_space(10.0);
            }
        });
        ui.add_space(16.0);
        if ui
            .add(
                egui::Button::new(RichText::new(format!("‹  {}", t("common.back"))).size(14.0).color(Color32::WHITE))
                    .frame(false),
            )
            .clicked()
        {
            model.mode = ProfilesMode::Select;
        }
        ui.add_space(48.0);
    });
}

fn modal(context: &egui::Context, screen: Rect, id: &str, add: impl FnOnce(&mut egui::Ui)) {
    context
        .layer_painter(egui::LayerId::new(egui::Order::Middle, Id::new((id, "scrim"))))
        .rect_filled(screen, 0.0, Color32::from_black_alpha(170));
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(context, |ui| {
            egui::Frame::NONE
                .fill(Color32::from_rgb(22, 22, 22))
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(26)))
                .corner_radius(14.0)
                .inner_margin(egui::Margin::same(26))
                .show(ui, |ui| {
                    ui.set_width(340.0);
                    add(ui);
                });
        });
}

fn draw_pin_prompt(
    context: &egui::Context,
    screen: Rect,
    model: &mut ProfilesModel,
    assets: &mut impl HomeAssets,
    _metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<ProfilesRequest>,
) {
    let Some(prompt) = model.pin_prompt.clone() else {
        return;
    };
    let profile = model.profile(&prompt.profile_id).cloned().unwrap_or_default();
    let mut pin = prompt.pin.clone();
    let mut cancel = false;
    modal(context, screen, "fluxa-profiles-pin", |ui| {
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(72.0), Sense::hover());
            components::avatar(ui.painter(), assets, rect.center(), 36.0, &profile.name, profile.avatar_url.as_deref());
            ui.add_space(12.0);
            ui.label(RichText::new(&profile.name).size(18.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            ui.label(
                RichText::new(t("profiles.pin_prompt"))
                    .size(13.0)
                    .color(Color32::from_white_alpha(140)),
            );
        });
        ui.add_space(18.0);
        let width = ui.available_width();
        let field = text_field(ui, &mut pin, &t("profiles.enter_pin"), width, true);
        if !field.has_focus() && pin.is_empty() {
            field.request_focus();
        }
        pin.retain(|character| character.is_ascii_digit());
        pin.truncate(4);
        if prompt.error {
            note(ui, &t("profiles.pin_error"));
        }
        ui.add_space(18.0);
        let submit = pin.len() == 4
            && (field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let half = (width - 8.0) * 0.5;
            if secondary_button(ui, &t("common.cancel"), half).clicked()
                || ui.input(|input| input.key_pressed(egui::Key::Escape))
            {
                cancel = true;
            }
            if primary_button(ui, &t("profiles.unlock"), half, pin.len() == 4).clicked() || submit {
                *request = Some(ProfilesRequest::SubmitPin);
            }
        });
    });
    if cancel {
        model.pin_prompt = None;
    } else if let Some(current) = model.pin_prompt.as_mut() {
        if current.pin != pin {
            current.error = false;
        }
        current.pin = pin;
    }
}
