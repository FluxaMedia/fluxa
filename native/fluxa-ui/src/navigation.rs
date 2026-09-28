use super::*;

pub(crate) const NAV_BAR_TOP: f32 = 14.0;
pub(crate) const NAV_ITEM_HEIGHT: f32 = 40.0;
pub(crate) const NAV_BAR_PADDING: f32 = 6.0;
pub(crate) const NAV_ITEM_GAP: f32 = 2.0;
pub(crate) const NAV_AVATAR_RADIUS: f32 = 13.5;

pub(crate) fn estimated_navigation_text_width(text: &str, size: f32) -> f32 {
    text.chars()
        .map(|character| match character {
            'i' | 'l' | 'I' | '!' | '.' => 0.30,
            'm' | 'w' | 'M' | 'W' => 0.82,
            ' ' => 0.32,
            _ => 0.54,
        })
        .sum::<f32>()
        * size
        + 4.0
}

/// Draws the navigation shared by every native screen. The active route is an
/// index into Home/Library/Discover/Calendar and the returned node id is
/// consumed by the platform event adapters.
pub fn draw_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &impl HomeAssets,
) -> Option<u64> {
    draw_navigation_bar_with_profile(
        context,
        viewport,
        active_route,
        assets,
        assets.active_profile_avatar_url(),
        assets.active_profile_name().unwrap_or("Profile"),
    )
}

pub(crate) fn draw_navigation_bar_with_profile(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    if compact {
        return draw_mobile_navigation_bar(
            context,
            viewport,
            active_route,
            metrics,
            assets,
            profile_avatar_url,
        );
    }
    let label_size = metrics.navigation_label_size(tv);
    let icon_size = metrics.navigation_icon_size(tv);
    let profile_width = metrics.navigation_profile_width(profile_name, tv);
    let bar_width = metrics.navigation_bar_width(viewport, profile_name) + NAV_BAR_PADDING * 2.0;
    let bar_x = (viewport.width - bar_width).max(12.0) * 0.5;
    let font = FontId::proportional(label_size);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-top-bar"))
        .fixed_pos(Pos2::new(bar_x, NAV_BAR_TOP))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            egui::Frame::NONE
                .fill(Color32::from_black_alpha(190))
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(22)))
                .corner_radius(NAV_ITEM_HEIGHT * 0.5 + NAV_BAR_PADDING)
                .inner_margin(egui::Margin::same(NAV_BAR_PADDING as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = NAV_ITEM_GAP;
                        for (index, label) in ["Home", "Library", "Discover", "Calendar"]
                            .into_iter()
                            .enumerate()
                        {
                            let active = index == active_route;
                            let width = metrics.navigation_item_width(label, tv);
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(width, NAV_ITEM_HEIGHT),
                                Sense::click(),
                            );
                            let fill = if active {
                                Color32::from_white_alpha(28)
                            } else if response.hovered() {
                                Color32::from_white_alpha(12)
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(rect, NAV_ITEM_HEIGHT * 0.5, fill);
                            let color = if active || response.hovered() {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(160)
                            };
                            let galley =
                                ui.painter()
                                    .layout_no_wrap(label.to_owned(), font.clone(), color);
                            let content = icon_size + 8.0 + galley.size().x;
                            let left = rect.center().x - content * 0.5;
                            if let Some(icon_id) = assets.icon(label) {
                                ui.painter().image(
                                    icon_id,
                                    Rect::from_min_size(
                                        Pos2::new(left, rect.center().y - icon_size * 0.5),
                                        Vec2::splat(icon_size),
                                    ),
                                    full_uv(),
                                    color,
                                );
                            }
                            ui.painter().galley(
                                Pos2::new(
                                    left + icon_size + 8.0,
                                    rect.center().y - galley.size().y * 0.5,
                                ),
                                galley,
                                color,
                            );
                            if response.clicked() {
                                activated = Some(NODE_HOME + index as u64);
                            }
                        }
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(profile_width, NAV_ITEM_HEIGHT),
                            Sense::click(),
                        );
                        let profile_active = active_route == 4;
                        if profile_active || response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                NAV_ITEM_HEIGHT * 0.5,
                                Color32::from_white_alpha(if profile_active { 28 } else { 12 }),
                            );
                        }
                        let center = rect.left_center() + Vec2::new(8.0 + NAV_AVATAR_RADIUS, 0.0);
                        if let Some(texture) = assets.cached_texture(profile_avatar_url) {
                            paint_circle_texture(ui.painter(), texture, center, NAV_AVATAR_RADIUS);
                        } else {
                            ui.painter().circle_filled(
                                center,
                                NAV_AVATAR_RADIUS,
                                Color32::from_white_alpha(30),
                            );
                            if let Some(icon) = assets.icon("Account") {
                                ui.painter().image(
                                    icon,
                                    Rect::from_center_size(center, Vec2::splat(16.0)),
                                    full_uv(),
                                    Color32::from_white_alpha(215),
                                );
                            }
                        }
                        let label_left = center.x + NAV_AVATAR_RADIUS + 8.0;
                        ui.painter().text(
                            Pos2::new(label_left, rect.center().y),
                            Align2::LEFT_CENTER,
                            truncate_to_width(
                                ui.painter(),
                                profile_name,
                                &font,
                                rect.right() - 12.0 - label_left,
                            ),
                            font.clone(),
                            if profile_active || response.hovered() {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(200)
                            },
                        );
                        if response.clicked() {
                            activated = Some(NODE_PROFILE);
                        }
                    });
                });
        });
    activated
}

pub(crate) fn draw_mobile_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
) -> Option<u64> {
    let height = 64.0;
    let slot_width = ((viewport.width - 16.0) / 5.0).max(1.0);
    let y = (viewport.height - viewport.safe_bottom - height).max(0.0);
    let bar = Rect::from_min_max(
        Pos2::new(0.0, y),
        Pos2::new(viewport.width, viewport.height),
    );
    let [r, g, b, _] = metrics.background.to_array();
    let solid = Color32::from_rgb(r, g, b);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-bottom-bar"))
        .fixed_pos(Pos2::new(8.0, y))
        .order(egui::Order::Tooltip)
        .show(context, |ui| {
            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            paint_vertical_gradient(
                &painter,
                Rect::from_min_max(Pos2::new(0.0, y - 48.0), Pos2::new(viewport.width, y + 8.0)),
                Color32::TRANSPARENT,
                solid,
            );
            painter.rect_filled(Rect::from_min_max(Pos2::new(0.0, y + 8.0), bar.max), 0.0, solid);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (index, (label, icon, node)) in [
                    ("Home", "Home", NODE_HOME),
                    ("Library", "Library", NODE_LIBRARY),
                    ("Discover", "Discover", NODE_DISCOVER),
                    ("Calendar", "Calendar", NODE_CALENDAR),
                    ("Profile", "Account", NODE_PROFILE),
                ]
                .into_iter()
                .enumerate()
                {
                    let active = index == active_route;
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(slot_width, height), Sense::click());
                    let color = if active {
                        Color32::WHITE
                    } else {
                        Color32::from_white_alpha(120)
                    };
                    let center = rect.center() - Vec2::new(0.0, 9.0);
                    let avatar = (index == 4)
                        .then(|| assets.cached_texture(profile_avatar_url))
                        .flatten();
                    if let Some(avatar) = avatar {
                        paint_circle_texture(ui.painter(), avatar, center, 12.0);
                        if active {
                            ui.painter().circle_stroke(
                                center,
                                13.5,
                                egui::Stroke::new(1.5, Color32::WHITE),
                            );
                        }
                    } else if let Some(icon) = assets.icon(icon) {
                        ui.painter().image(
                            icon,
                            Rect::from_center_size(center, Vec2::splat(22.0)),
                            full_uv(),
                            color,
                        );
                    }
                    ui.painter().text(
                        Pos2::new(rect.center().x, rect.center().y + 14.0),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::proportional(metrics.text.label - 1.0),
                        color,
                    );
                    if response.clicked() {
                        activated = Some(node);
                    }
                }
            });
        });
    activated
}

pub fn navigation_focus_rects(viewport: Viewport, metrics: UiMetrics) -> Vec<(u64, Rect)> {
    if viewport.is_compact() {
        let height = 64.0;
        let y = (viewport.height - viewport.safe_bottom - height).max(0.0);
        let slot = ((viewport.width - 16.0) / 5.0).max(1.0);
        [
            NODE_HOME,
            NODE_LIBRARY,
            NODE_DISCOVER,
            NODE_CALENDAR,
            NODE_PROFILE,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            (
                node,
                Rect::from_min_size(
                    Pos2::new(8.0 + slot * index as f32, y + 4.0),
                    Vec2::new(slot, height - 8.0),
                ),
            )
        })
        .collect()
    } else {
        let tv = viewport.is_tv();
        let widths = ["Home", "Library", "Discover", "Calendar"]
            .map(|label| metrics.navigation_item_width(label, tv));
        let bar_width = metrics.navigation_bar_width(viewport, "Profile") + NAV_BAR_PADDING * 2.0;
        let x = (viewport.width - bar_width).max(12.0) * 0.5 + NAV_BAR_PADDING;
        let y = NAV_BAR_TOP + NAV_BAR_PADDING;
        let mut left = x;
        let mut rects = Vec::with_capacity(5);
        for (index, node) in [NODE_HOME, NODE_LIBRARY, NODE_DISCOVER, NODE_CALENDAR]
            .into_iter()
            .enumerate()
        {
            let item_width = widths[index];
            rects.push((
                node,
                Rect::from_min_size(Pos2::new(left, y), Vec2::new(item_width, NAV_ITEM_HEIGHT)),
            ));
            left += item_width + NAV_ITEM_GAP;
        }
        let profile_width = metrics.navigation_profile_width("Profile", tv);
        rects.push((
            NODE_PROFILE,
            Rect::from_min_size(
                Pos2::new(left, y),
                Vec2::new(profile_width, NAV_ITEM_HEIGHT),
            ),
        ));
        rects
    }
}

pub(crate) fn is_navigation_node(node: u64) -> bool {
    matches!(
        node,
        NODE_HOME | NODE_LIBRARY | NODE_DISCOVER | NODE_CALENDAR | NODE_PROFILE
    )
}
