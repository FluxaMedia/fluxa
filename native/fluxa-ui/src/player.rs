use super::*;

#[derive(Clone, Debug, Default)]
pub struct PlayerModel {
    pub title: String,
    pub video: Option<TextureId>,
    pub status: Option<(String, String)>,
    pub error: Option<String>,
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub controls_visible: bool,
    pub show_pause_info: bool,
    pub logo: Option<String>,
    pub episode_title: Option<String>,
    pub description: Option<String>,
    pub chapters: Vec<(f64, String)>,
    pub thumbnail: Option<(f64, TextureId)>,
    pub warnings: Vec<(String, String)>,
    pub warnings_elapsed: Option<f32>,
    pub language: String,
    pub upscaling: String,
    pub recommendations: Vec<HomeCard>,
}

const WARNING_BAR: f32 = 0.3;
const WARNING_STAGGER: f32 = 0.08;
const WARNING_FADE: f32 = 0.25;
const WARNING_HOLD: f32 = 5.0;

pub fn content_warning_duration(rows: usize) -> f32 {
    let rows = rows as f32 * WARNING_STAGGER + WARNING_FADE;
    WARNING_BAR * 2.0 + rows * 2.0 + WARNING_HOLD
}

impl PlayerModel {
    fn chapter_at(&self, time: f64) -> Option<&str> {
        self.chapters
            .iter()
            .rev()
            .find(|(start, _)| *start <= time)
            .map(|(_, title)| title.as_str())
            .filter(|title| !title.trim().is_empty())
    }
}

pub fn draw_player(
    context: &egui::Context,
    viewport: Viewport,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let mut layout = HomeLayout::default();
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    match player.video {
        Some(texture) => {
            painter.rect_filled(rect, 0.0, Color32::BLACK);
            painter.image(texture, rect, full_uv(), Color32::WHITE);
        }
        None => {
            painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 8, 10));
        }
    }
    if player.video.is_none() {
        draw_status(&painter, rect, player);
    }
    if player.show_pause_info {
        draw_pause_info(context, &painter, rect, player, assets);
    }
    let compact = viewport.is_compact();
    let margin = (rect.width() * 0.035).clamp(if compact { 14.0 } else { 22.0 }, 64.0);
    let header_y = rect.top() + 30.0 + if compact { 8.0 } else { 0.0 };
    let close_rect = Rect::from_center_size(Pos2::new(margin + 20.0, header_y), Vec2::splat(42.0));
    if let Some(elapsed) = player.warnings_elapsed {
        let top = if player.controls_visible { 72.0 } else { 24.0 };
        draw_warnings(context, Pos2::new(margin, top), player, elapsed);
    }
    egui::Area::new(Id::new("fluxa-player-controls"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            if player.video.is_none() || !player.controls_visible {
                let close = control(ui, &painter, close_rect, "close", false);
                layout.focusable.push((NODE_PLAYER_CLOSE, close.rect));
                if close.clicked() {
                    layout.activated = Some(NODE_PLAYER_CLOSE);
                }
                if player.video.is_some() {
                    let surface = ui.interact(rect, Id::new("fluxa-player-surface"), Sense::click());
                    if surface.clicked() {
                        layout.activated = Some(NODE_PLAYER_TOGGLE);
                    }
                }
                return;
            }
            scrims(&painter, rect);
            let close = control(ui, &painter, close_rect, "close", false);
            layout.focusable.push((NODE_PLAYER_CLOSE, close.rect));
            if close.clicked() {
                layout.activated = Some(NODE_PLAYER_CLOSE);
            }
            painter.text(
                Pos2::new(close_rect.right() + 10.0, header_y),
                Align2::LEFT_CENTER,
                truncate_to_width(
                    &painter,
                    &player.title,
                    &FontId::proportional(19.0),
                    rect.width() - close_rect.right() - margin - 10.0,
                ),
                FontId::proportional(19.0),
                Color32::WHITE,
            );

            let bar_width = (rect.width() - margin * 2.0).max(80.0);
            let track = Rect::from_min_size(
                Pos2::new(margin, rect.bottom() - 93.0 - viewport.safe_bottom),
                Vec2::new(bar_width, 16.0),
            );
            let seek = ui.interact(
                track.expand2(Vec2::new(0.0, 8.0)),
                Id::new("fluxa-player-seek"),
                Sense::click_and_drag(),
            );
            layout.focusable.push((NODE_PLAYER_SEEK, track));
            let duration = player.duration.max(0.0);
            let mut position = player.position;
            if (seek.clicked() || seek.dragged())
                && let Some(pointer) = seek.interact_pointer_pos()
            {
                let ratio = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
                position = duration * ratio as f64;
                if seek.clicked() || seek.drag_stopped() || seek.dragged() {
                    layout.seek_to = Some(position);
                }
            }
            painter.rect_filled(
                Rect::from_center_size(track.center(), Vec2::new(track.width(), 3.0)),
                2.0,
                Color32::from_white_alpha(95),
            );
            if duration > 0.0 {
                for (start, _) in &player.chapters {
                    if *start <= 0.0 || *start >= duration {
                        continue;
                    }
                    let x = track.left() + track.width() * (*start / duration) as f32;
                    painter.rect_filled(
                        Rect::from_center_size(Pos2::new(x, track.center().y), Vec2::new(2.0, 5.0)),
                        0.0,
                        Color32::BLACK,
                    );
                }
                let played = track.width() * (position / duration).clamp(0.0, 1.0) as f32;
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(track.left(), track.center().y - 1.75),
                        Vec2::new(played, 3.5),
                    ),
                    2.0,
                    Color32::WHITE,
                );
                if seek.hovered() || seek.dragged() || focused == Some(NODE_PLAYER_SEEK) {
                    painter.circle_filled(
                        Pos2::new(track.left() + played, track.center().y),
                        6.0,
                        Color32::WHITE,
                    );
                }
            }

            if duration > 0.0
                && let Some(pointer) = seek.hover_pos()
            {
                let ratio = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
                let time = duration * ratio as f64;
                layout.seek_hover = Some(time);
                draw_seek_preview(context, track, pointer.x, time, player);
            }

            let controls_y = rect.bottom() - 48.0 - viewport.safe_bottom;
            let mut x = margin + 1.0;
            let play = control(
                ui,
                &painter,
                Rect::from_center_size(Pos2::new(x + 22.0, controls_y), Vec2::splat(48.0)),
                if player.paused { "play" } else { "pause" },
                true,
            );
            layout.focusable.push((NODE_PLAYER_TOGGLE, play.rect));
            if play.clicked() {
                layout.activated = Some(NODE_PLAYER_TOGGLE);
            }
            x += 62.0;
            for (icon, node) in [
                ("back", NODE_PLAYER_REWIND),
                ("forward", NODE_PLAYER_FORWARD),
            ] {
                let button = control(
                    ui,
                    &painter,
                    Rect::from_center_size(Pos2::new(x + 21.0, controls_y), Vec2::splat(42.0)),
                    icon,
                    false,
                );
                layout.focusable.push((node, button.rect));
                if button.clicked() {
                    layout.activated = Some(node);
                }
                x += 48.0;
            }
            painter.text(
                Pos2::new(x + 2.0, controls_y),
                Align2::LEFT_CENTER,
                format!(
                    "{}  /  {}",
                    format_time(position),
                    format_time(duration)
                ),
                FontId::proportional(13.0),
                Color32::from_white_alpha(218),
            );

            let right = rect.right() - margin - 20.0;
            let fullscreen = control(
                ui,
                &painter,
                Rect::from_center_size(Pos2::new(right, controls_y), Vec2::splat(42.0)),
                "fullscreen",
                false,
            );
            layout.focusable.push((NODE_PLAYER_FULLSCREEN, fullscreen.rect));
            if fullscreen.clicked() {
                layout.activated = Some(NODE_PLAYER_FULLSCREEN);
            }
            let volume_x = right - 52.0;
            let mute = control(
                ui,
                &painter,
                Rect::from_center_size(Pos2::new(volume_x, controls_y), Vec2::splat(42.0)),
                if player.muted { "mute" } else { "volume" },
                false,
            );
            layout.focusable.push((NODE_PLAYER_MUTE, mute.rect));
            if mute.clicked() {
                layout.activated = Some(NODE_PLAYER_MUTE);
            }
            painter.text(
                Pos2::new(volume_x - 18.0, controls_y),
                Align2::RIGHT_CENTER,
                if player.muted {
                    localized("player.muted", &player.language)
                } else {
                    format!("{}%", player.volume.round() as i32)
                },
                FontId::proportional(12.0),
                Color32::from_white_alpha(170),
            );
            let upscaling_label = format!(
                "{}  {}",
                localized("player.anime4k", &player.language),
                match player.upscaling.as_str() {
                    "off" | "" => localized("player.off", &player.language),
                    mode => localized(&format!("player.anime4k_mode_{mode}"), &player.language),
                }
            );
            let upscaling_rect = Rect::from_center_size(
                Pos2::new(volume_x - 138.0, controls_y),
                Vec2::new(150.0, 34.0),
            );
            let upscaling = ui.interact(
                upscaling_rect,
                Id::new("fluxa-player-upscaling"),
                Sense::click(),
            );
            painter.rect_filled(
                upscaling_rect,
                17.0,
                Color32::from_white_alpha(if upscaling.hovered() { 40 } else { 22 }),
            );
            painter.text(
                upscaling_rect.center(),
                Align2::CENTER_CENTER,
                upscaling_label,
                FontId::proportional(13.0),
                Color32::from_white_alpha(225),
            );
            layout.focusable.push((NODE_PLAYER_UPSCALING, upscaling_rect));
            if upscaling.clicked() {
                layout.activated = Some(NODE_PLAYER_UPSCALING);
            }
            let surface = ui.interact(
                Rect::from_min_max(
                    Pos2::new(rect.left(), close_rect.bottom() + 8.0),
                    Pos2::new(rect.right(), track.top() - 8.0),
                ),
                Id::new("fluxa-player-surface"),
                Sense::click(),
            );
            if surface.clicked() {
                layout.activated = Some(NODE_PLAYER_TOGGLE);
            }
        });
    if !player.recommendations.is_empty() {
        let bottom = if player.video.is_some() && player.controls_visible {
            rect.bottom() - 116.0
        } else {
            rect.bottom() - 32.0
        } - viewport.safe_bottom;
        draw_recommendations(context, rect, margin, bottom, compact, player, assets, &mut layout);
    }
    if let Some((_, rect)) = layout.focusable.iter().find(|(id, _)| Some(*id) == focused) {
        context
            .layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                Id::new("fluxa-player-focus"),
            ))
            .rect_stroke(
                rect.expand(3.0),
                rect.height() * 0.5,
                egui::Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
    }
    layout
}

#[allow(clippy::too_many_arguments)]
fn draw_recommendations(
    context: &egui::Context,
    rect: Rect,
    margin: f32,
    bottom: f32,
    compact: bool,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let poster = if compact { Vec2::new(92.0, 138.0) } else { Vec2::new(128.0, 192.0) };
    let label = 30.0;
    let header = 40.0;
    let top = bottom - poster.y - label - header;
    let ppp = context.pixels_per_point();
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        Id::new("fluxa-player-recommendations-scrim"),
    ));
    let fade_top = top - 80.0;
    let bands = 20;
    let band = (rect.bottom() - fade_top) / bands as f32;
    for index in 0..bands {
        let strength = (index as f32 + 1.0) / bands as f32;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), fade_top + index as f32 * band),
                Pos2::new(rect.right(), fade_top + (index + 1) as f32 * band),
            ),
            0.0,
            Color32::from_black_alpha((200.0 * strength.powf(1.4)).round() as u8),
        );
    }
    egui::Area::new(Id::new("fluxa-player-recommendations"))
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::new(margin, top))
        .show(context, |ui| {
            let width = rect.width() - margin * 2.0;
            ui.set_max_width(width);
            let painter = ui.painter().clone();
            painter.text(
                Pos2::new(margin, top + header * 0.5),
                Align2::LEFT_CENTER,
                localized("player.recommendations", &player.language),
                FontId::proportional(if compact { 17.0 } else { 20.0 }),
                Color32::WHITE,
            );
            let close = control(
                ui,
                &painter,
                Rect::from_center_size(
                    Pos2::new(margin + width - 18.0, top + header * 0.5),
                    Vec2::splat(36.0),
                ),
                "close",
                false,
            );
            layout.focusable.push((NODE_PLAYER_RECOMMENDATIONS_CLOSE, close.rect));
            if close.clicked() {
                layout.activated = Some(NODE_PLAYER_RECOMMENDATIONS_CLOSE);
            }
            ui.add_space(header);
            egui::ScrollArea::horizontal()
                .id_salt("fluxa-player-recommendations-scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 14.0;
                        for (index, card) in player.recommendations.iter().enumerate() {
                            let (card_rect, response) = ui.allocate_exact_size(
                                Vec2::new(poster.x, poster.y + label),
                                Sense::click(),
                            );
                            let node = NODE_PLAYER_RECOMMENDATION_BASE + index as u64;
                            layout.focusable.push((node, card_rect));
                            if response.clicked() {
                                layout.activated = Some(node);
                            }
                            if !ui.is_rect_visible(card_rect) {
                                continue;
                            }
                            let painter = ui.painter();
                            let lift = context.animate_bool_with_time(
                                Id::new(("fluxa-player-recommendation-hover", index)),
                                response.hovered(),
                                0.15,
                            );
                            let image = Rect::from_min_size(card_rect.min, poster).expand(4.0 * lift);
                            painter.rect_filled(image, 10.0, Color32::from_white_alpha(14));
                            super::detail::rounded_art(
                                painter,
                                image,
                                10.0,
                                card.artwork_url.as_deref(),
                                ppp,
                                assets,
                            );
                            let font = FontId::proportional(13.0);
                            painter.text(
                                Pos2::new(card_rect.left(), card_rect.top() + poster.y + 9.0),
                                Align2::LEFT_TOP,
                                truncate_to_width(painter, &card.title, &font, poster.x),
                                font,
                                Color32::from_white_alpha(210),
                            );
                        }
                    });
                });
        });
}

fn draw_seek_preview(
    context: &egui::Context,
    track: Rect,
    pointer_x: f32,
    time: f64,
    player: &PlayerModel,
) {
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-seek-preview"),
    ));
    let chapter = player.chapter_at(time);
    let thumbnail = player
        .thumbnail
        .filter(|(shown, _)| (shown - time).abs() <= 30.0)
        .map(|(_, texture)| texture);
    let width = if thumbnail.is_some() { 240.0 } else { 160.0 };
    let x = pointer_x.clamp(track.left() + width * 0.5, track.right() - width * 0.5);
    let mut bottom = track.top() - 10.0;
    let label_font = FontId::proportional(13.0);
    painter.text(
        Pos2::new(x, bottom),
        Align2::CENTER_BOTTOM,
        format_time(time),
        label_font.clone(),
        Color32::WHITE,
    );
    bottom -= 18.0;
    if let Some(chapter) = chapter {
        painter.text(
            Pos2::new(x, bottom),
            Align2::CENTER_BOTTOM,
            truncate_to_width(&painter, chapter, &label_font, width),
            label_font,
            Color32::from_white_alpha(200),
        );
        bottom -= 20.0;
    }
    if let Some(texture) = thumbnail {
        let frame = Rect::from_min_max(
            Pos2::new(x - width * 0.5, bottom - width * 9.0 / 16.0),
            Pos2::new(x + width * 0.5, bottom),
        );
        painter.rect_filled(frame.expand(1.0), 4.0, Color32::from_white_alpha(40));
        painter.image(texture, frame, full_uv(), Color32::WHITE);
    }
}

fn draw_pause_info(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
) {
    let band = rect.width() / 24.0;
    for index in 0..24 {
        let fade = 1.0 - index as f32 / 24.0;
        let left = rect.left() + index as f32 * band;
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(left, rect.top()), Pos2::new(left + band, rect.bottom())),
            0.0,
            Color32::from_black_alpha((200.0 * fade.powf(1.4)).round() as u8),
        );
    }
    let left = rect.left() + 64.0;
    let width = (rect.width() * 0.42).clamp(260.0, 620.0);
    let mut blocks: Vec<(f32, Box<dyn Fn(&egui::Painter, f32)>)> = Vec::new();
    let muted = Color32::from_white_alpha(190);

    let label = localized("player.youre_watching", &player.language);
    blocks.push((
        28.0,
        Box::new(move |painter, y| {
            painter.text(
                Pos2::new(left, y),
                Align2::LEFT_TOP,
                &label,
                FontId::proportional(16.0),
                muted,
            );
        }),
    ));
    let max_logo = Vec2::new(width.min(420.0), 96.0);
    let logo = player.logo.as_deref().and_then(|url| {
        let texture = assets.texture_for(
            Some(url),
            artwork_target_size(max_logo, context.pixels_per_point()),
            ArtworkPriority::Hero,
        )?;
        let size = assets
            .texture_size(Some(url))
            .map(|size| contain_size(size, max_logo))
            .unwrap_or(max_logo);
        Some((texture, size))
    });
    match logo {
        Some((texture, size)) => blocks.push((
            size.y + 16.0,
            Box::new(move |painter, y| {
                painter.image(
                    texture,
                    Rect::from_min_size(Pos2::new(left, y), size),
                    full_uv(),
                    Color32::WHITE,
                );
            }),
        )),
        None => {
            let galley = wrapped(painter, &player.title, 40.0, Color32::WHITE, width, 2);
            blocks.push((
                galley.size().y + 12.0,
                Box::new(move |painter, y| {
                    painter.galley(Pos2::new(left, y), galley.clone(), Color32::WHITE)
                }),
            ));
        }
    }
    let chapter = player
        .chapter_at(player.position)
        .map(|title| format!("{}: {title}", localized("player.chapter", &player.language)));
    for (text, size) in [
        (player.episode_title.clone(), 20.0),
        (chapter, 15.0),
        (player.description.clone(), 16.0),
    ] {
        let Some(text) = text else {
            continue;
        };
        let rows = if size == 16.0 { 5 } else { 1 };
        let galley = wrapped(painter, &text, size, muted, width, rows);
        blocks.push((
            galley.size().y + 10.0,
            Box::new(move |painter, y| painter.galley(Pos2::new(left, y), galley.clone(), muted)),
        ));
    }
    let height: f32 = blocks.iter().map(|(height, _)| height).sum();
    let mut y = rect.bottom() - 120.0 - height;
    for (height, draw) in blocks {
        draw(painter, y);
        y += height;
    }
}

fn wrapped(
    painter: &egui::Painter,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), FontId::proportional(size), color, width);
    job.wrap.max_rows = rows;
    painter.layout_job(job)
}

fn draw_warnings(context: &egui::Context, origin: Pos2, player: &PlayerModel, elapsed: f32) {
    let rows = player.warnings.len();
    if rows == 0 {
        return;
    }
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-warnings"),
    ));
    let total = content_warning_duration(rows);
    let ramp = |value: f32| value.clamp(0.0, 1.0);
    let box_alpha = ramp(elapsed / WARNING_BAR).min(ramp((total - elapsed) / WARNING_BAR));
    let font = FontId::proportional(14.0);
    let row_height = 20.0;
    let lines = player
        .warnings
        .iter()
        .map(|(label, severity)| format!("{label} · {severity}"))
        .collect::<Vec<_>>();
    let text_width = lines
        .iter()
        .map(|line| {
            painter
                .layout_no_wrap(line.clone(), font.clone(), Color32::WHITE)
                .size()
                .x
        })
        .fold(0.0, f32::max);
    let panel = Rect::from_min_size(
        origin,
        Vec2::new(text_width + 34.0, rows as f32 * row_height + 20.0),
    );
    painter.rect_filled(panel, 6.0, Color32::from_black_alpha((173.0 * box_alpha) as u8));
    let bar = panel.height() - 20.0;
    painter.rect_filled(
        Rect::from_min_size(
            panel.min + Vec2::new(12.0, 10.0),
            Vec2::new(3.0, bar * ramp(elapsed / WARNING_BAR)),
        ),
        1.5,
        Color32::from_white_alpha((230.0 * box_alpha) as u8),
    );
    let fade_out_start = total - WARNING_BAR - rows as f32 * WARNING_STAGGER - WARNING_FADE;
    for (index, line) in lines.into_iter().enumerate() {
        let appear = ramp((elapsed - WARNING_BAR - index as f32 * WARNING_STAGGER) / WARNING_FADE);
        let leave = fade_out_start + (rows - 1 - index) as f32 * WARNING_STAGGER;
        let alpha = appear.min(1.0 - ramp((elapsed - leave) / WARNING_FADE));
        painter.text(
            panel.min + Vec2::new(24.0, 10.0 + index as f32 * row_height + row_height * 0.5),
            Align2::LEFT_CENTER,
            line,
            font.clone(),
            Color32::from_white_alpha((255.0 * alpha) as u8),
        );
    }
}

fn draw_status(painter: &egui::Painter, rect: Rect, player: &PlayerModel) {
    let center = rect.center();
    painter.text(
        center - Vec2::new(0.0, 28.0),
        Align2::CENTER_CENTER,
        &player.title,
        FontId::proportional(26.0),
        Color32::WHITE,
    );
    let (headline, detail) = player.status.clone().unwrap_or_else(|| {
        (
            "Preparing player…".to_owned(),
            "Waiting for the first video frame".to_owned(),
        )
    });
    painter.text(
        center + Vec2::new(0.0, 14.0),
        Align2::CENTER_CENTER,
        player.error.as_deref().unwrap_or(&headline),
        FontId::proportional(15.0),
        if player.error.is_some() {
            Color32::from_rgb(255, 130, 110)
        } else {
            Color32::from_white_alpha(180)
        },
    );
    if player.error.is_none() {
        painter.text(
            center + Vec2::new(0.0, 42.0),
            Align2::CENTER_CENTER,
            detail,
            FontId::proportional(13.0),
            Color32::from_white_alpha(145),
        );
    }
}

fn scrims(painter: &egui::Painter, rect: Rect) {
    let band = 18.0;
    for index in 0..16 {
        let fade = 1.0 - index as f32 / 16.0;
        let top = rect.top() + index as f32 * band;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), top),
                Pos2::new(rect.right(), (top + band).min(rect.bottom())),
            ),
            0.0,
            Color32::from_black_alpha((152.0 * fade.powf(1.7)).round() as u8),
        );
        let bottom = rect.bottom() - index as f32 * band;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), (bottom - band).max(rect.top())),
                Pos2::new(rect.right(), bottom),
            ),
            0.0,
            Color32::from_black_alpha((218.0 * fade.powf(1.45)).round() as u8),
        );
    }
}

fn control(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    icon: &str,
    prominent: bool,
) -> egui::Response {
    let response = ui.interact(rect, Id::new(("fluxa-player-button", icon)), Sense::click());
    if prominent || response.hovered() || response.is_pointer_button_down_on() {
        painter.circle_filled(
            rect.center(),
            rect.width() * 0.5,
            if prominent {
                Color32::from_white_alpha(30)
            } else {
                Color32::from_black_alpha(75)
            },
        );
    }
    let color = if response.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(235)
    };
    let c = rect.center();
    let stroke = egui::Stroke::new(2.0, color);
    match icon {
        "play" => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + Vec2::new(-5.0, -8.0),
                    c + Vec2::new(8.0, 0.0),
                    c + Vec2::new(-5.0, 8.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
        }
        "pause" => {
            for offset in [-4.0, 4.0] {
                painter.rect_filled(
                    Rect::from_center_size(c + Vec2::new(offset, 0.0), Vec2::new(4.0, 17.0)),
                    1.0,
                    color,
                );
            }
        }
        "close" => {
            painter.line_segment([c + Vec2::new(-6.0, -6.0), c + Vec2::new(6.0, 6.0)], stroke);
            painter.line_segment([c + Vec2::new(6.0, -6.0), c + Vec2::new(-6.0, 6.0)], stroke);
        }
        "back" | "forward" => {
            let sign = if icon == "back" { -1.0 } else { 1.0 };
            let center = c + Vec2::new(-4.0 * sign, -2.0);
            painter.add(egui::Shape::line(
                (0..=20)
                    .map(|step| {
                        let angle = 0.25 + (5.2 - 0.25) * step as f32 / 20.0;
                        center + Vec2::new(angle.cos() * 8.0, angle.sin() * 8.0)
                    })
                    .collect(),
                egui::Stroke::new(1.8, color),
            ));
            painter.text(
                c + Vec2::new(0.0, 7.0),
                Align2::CENTER_CENTER,
                "10",
                FontId::proportional(9.0),
                color,
            );
        }
        "volume" | "mute" => {
            painter.rect_filled(
                Rect::from_min_max(c + Vec2::new(-9.0, -4.0), c + Vec2::new(-5.0, 4.0)),
                0.5,
                color,
            );
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + Vec2::new(-5.0, -5.0),
                    c + Vec2::new(2.0, -10.0),
                    c + Vec2::new(2.0, 10.0),
                    c + Vec2::new(-5.0, 5.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
            if icon == "mute" {
                let thin = egui::Stroke::new(1.8, color);
                painter.line_segment([c + Vec2::new(5.0, -5.0), c + Vec2::new(11.0, 5.0)], thin);
                painter.line_segment([c + Vec2::new(11.0, -5.0), c + Vec2::new(5.0, 5.0)], thin);
            } else {
                painter.add(egui::Shape::line(
                    (0..=10)
                        .map(|step| {
                            let angle = -0.75 + 1.5 * step as f32 / 10.0;
                            c + Vec2::new(angle.cos() * 10.0, angle.sin() * 10.0)
                        })
                        .collect(),
                    egui::Stroke::new(1.5, color),
                ));
            }
        }
        "fullscreen" => {
            for (a, b) in [
                ((-8.0, -3.0), (-8.0, -8.0)),
                ((-8.0, -8.0), (-3.0, -8.0)),
                ((8.0, -3.0), (8.0, -8.0)),
                ((8.0, -8.0), (3.0, -8.0)),
                ((-8.0, 3.0), (-8.0, 8.0)),
                ((-8.0, 8.0), (-3.0, 8.0)),
                ((8.0, 3.0), (8.0, 8.0)),
                ((8.0, 8.0), (3.0, 8.0)),
            ] {
                painter.line_segment([c + Vec2::new(a.0, a.1), c + Vec2::new(b.0, b.1)], stroke);
            }
        }
        _ => {}
    }
    response
}

pub fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (total / 3600, (total / 60) % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub fn torrent_status_lines(status: Option<&serde_json::Value>) -> (String, String) {
    let Some(status) = status else {
        return (
            "Resolving torrent metadata…".to_owned(),
            "Connecting to trackers and discovering peers".to_owned(),
        );
    };
    let text = |key: &str| status.get(key).and_then(serde_json::Value::as_str);
    let number = |key: &str| status.get(key).and_then(serde_json::Value::as_u64).unwrap_or(0);
    if status.get("stat").and_then(serde_json::Value::as_i64) == Some(-1) {
        return (
            "Torrent stream reported an error".to_owned(),
            text("error")
                .filter(|error| !error.trim().is_empty())
                .unwrap_or("Still waiting for the selected torrent")
                .to_owned(),
        );
    }
    let phase = text("phase")
        .or_else(|| text("stat_string"))
        .unwrap_or("resolving_metadata");
    if phase == "initializing"
        && status.get("resolving").and_then(serde_json::Value::as_bool) == Some(false)
        && text("hash").is_none_or(str::is_empty)
    {
        return (
            "Torrent metadata has not resolved yet".to_owned(),
            "The selected source is still waiting for a metadata response".to_owned(),
        );
    }
    let peers = number("active_peers");
    let peer_total = number("total_peers");
    let percent = status
        .get("preload")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0)
        .round()
        .clamp(0.0, 100.0) as u64;
    let downloaded = number("loaded_size");
    let speed = status
        .get("download_speed")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let headline = match phase {
        "resolving_metadata" | "initializing" | "resolving" => "Fetching torrent metadata…",
        "connecting_peers" => "Discovering peers…",
        "buffering_startup" => "Downloading startup buffer…",
        "rebuffering" => "Refilling playback buffer…",
        "seeking" => "Preparing the requested playback position…",
        "streaming" => "Torrent buffer ready · starting video…",
        "stalled" => "Torrent transfer is paused",
        "error" => "Torrent stream reported an error",
        "status_unavailable" => "Checking torrent engine status…",
        _ if peers == 0 => "Discovering peers…",
        _ => "Downloading torrent data…",
    };
    if phase == "status_unavailable" {
        return (
            headline.to_owned(),
            text("error")
                .unwrap_or("No response from the torrent status endpoint")
                .to_owned(),
        );
    }
    let peers = if peer_total > 0 {
        format!("{peers}/{peer_total} peers")
    } else {
        format!("{peers} peers")
    };
    let mib = 1024.0 * 1024.0;
    let downloaded = if downloaded as f64 >= mib {
        format!("{:.1} MiB downloaded", downloaded as f64 / mib)
    } else {
        format!("{} KiB downloaded", downloaded / 1024)
    };
    let speed = if speed >= mib {
        format!("{:.1} MiB/s", speed / mib)
    } else if speed >= 1024.0 {
        format!("{:.0} KiB/s", speed / 1024.0)
    } else {
        format!("{speed:.0} B/s")
    };
    (
        headline.to_owned(),
        format!("{peers} · {percent}% buffer · {downloaded} · {speed}"),
    )
}
