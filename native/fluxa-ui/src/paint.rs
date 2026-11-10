use super::*;
use crate::Space;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

pub(crate) fn full_uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))
}

pub(crate) fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let mut output = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        output.pop();
        output.push('…');
    }
    output
}

pub(crate) fn today_iso() -> String {
    let days = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

pub(crate) fn draw_text(
    painter: &crate::gfx::Painter,
    pos: Pos2,
    align: Align2,
    text: impl AsRef<str>,
    font: FontId,
    color: Color32,
    max_width: Option<f32>,
) -> Rect {
    let text = text.as_ref();
    let text = match max_width {
        Some(width) => truncate_to_width(painter, text, &font, width),
        None => text.to_owned(),
    };
    painter.text(pos, align, text, font, color)
}

const TRUNCATION_CACHE_LIMIT: usize = 4096;

thread_local! {
    static TRUNCATIONS: std::cell::RefCell<std::collections::HashMap<u64, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

pub(crate) fn truncate_to_width(
    painter: &crate::gfx::Painter,
    text: &str,
    font: &FontId,
    max_width: f32,
) -> String {
    use std::hash::{Hash, Hasher};
    if text.is_empty() || max_width <= 0.0 {
        return String::new();
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    font.size.to_bits().hash(&mut hasher);
    font.family.hash(&mut hasher);
    max_width.to_bits().hash(&mut hasher);
    painter.ctx().pixels_per_point().to_bits().hash(&mut hasher);
    crate::fonts::epoch().hash(&mut hasher);
    let key = hasher.finish();
    if let Some(cached) = TRUNCATIONS.with(|cache| cache.borrow().get(&key).cloned()) {
        return cached;
    }
    let truncated = truncate_uncached(painter, text, font, max_width);
    TRUNCATIONS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= TRUNCATION_CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(key, truncated.clone());
    });
    truncated
}

fn truncate_uncached(
    painter: &crate::gfx::Painter,
    text: &str,
    font: &FontId,
    max_width: f32,
) -> String {
    let full_width = painter
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    if full_width <= max_width {
        return text.to_owned();
    }
    let ellipsis = "…";
    let boundaries = text
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect::<Vec<_>>();
    let mut lower = 0usize;
    let mut upper = boundaries.len();
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        let prefix = &text[..boundaries[middle]];
        let candidate = format!("{prefix}{ellipsis}");
        let width = painter
            .layout_no_wrap(candidate, font.clone(), Color32::WHITE)
            .size()
            .x;
        if width <= max_width {
            lower = middle + 1;
        } else {
            upper = middle;
        }
    }
    let prefix_len = lower.saturating_sub(1);
    if prefix_len == 0 {
        ellipsis.to_owned()
    } else {
        format!("{}{ellipsis}", &text[..boundaries[prefix_len]])
    }
}

pub(crate) fn accent_from_value(value: &serde_json::Value) -> Option<Color32> {
    if let Some(text) = value.as_str() {
        return settings::parse_settings_color(text);
    }
    let argb = value.as_i64()? as u32;
    Some(Color32::from_rgb(
        (argb >> 16) as u8,
        (argb >> 8) as u8,
        argb as u8,
    ))
}

pub(crate) fn accent_is_neutral(assets: &impl HomeAssets) -> bool {
    assets.accent_color().is_none_or(|accent| {
        let [r, g, b, _] = accent.to_array();
        r.max(g).max(b) - r.min(g).min(b) < 24
    })
}

pub(crate) fn paint_brand_ambient(
    painter: &crate::gfx::Painter,
    screen: Rect,
    assets: &impl HomeAssets,
) {
    paint_ambient(painter, screen, assets);
    if !accent_is_neutral(assets) {
        return;
    }
    let (Some(glow), Some([from, to])) = (assets.ambient_glow(), assets.brand_colors()) else {
        return;
    };
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    painter.image(glow, screen, uv, from.gamma_multiply_u8(70));
    let flipped = Rect::from_min_max(Pos2::new(1.0, 1.0), Pos2::ZERO);
    painter.image(glow, screen, flipped, to.gamma_multiply_u8(50));
}

pub(crate) fn paint_ambient(painter: &crate::gfx::Painter, screen: Rect, assets: &impl HomeAssets) {
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    match assets.background_image_size() {
        Some(size) => {
            painter.image(
                assets.background(),
                screen,
                cover_uv(size, screen),
                Color32::WHITE,
            );
            let dim = crate::metrics::active().background_dim;
            painter.rect_filled(screen, 0.0, Color32::from_black_alpha(dim));
        }
        None => {
            painter.image(assets.background(), screen, uv, Color32::WHITE);
        }
    }
    let Some(accent) = assets.accent_color() else {
        return;
    };
    let [r, g, b, _] = accent.to_array();
    let chroma = r.max(g).max(b) - r.min(g).min(b);
    if chroma < 24 {
        return;
    }
    if let Some(glow) = assets.ambient_glow() {
        painter.image(
            glow,
            screen,
            uv,
            Color32::from_rgba_unmultiplied(r, g, b, 70),
        );
    }
}

pub(crate) fn draw_loading_screen(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    profiles::paint_custom_background(context, painter, screen, assets);
    let time = context.input(|input| input.time) as f32;
    let metrics = crate::metrics::active();
    let unit = (screen.width().min(screen.height()) / metrics.control_skeleton_unit_base).clamp(
        metrics.control_skeleton_unit_min,
        metrics.control_skeleton_unit_max,
    );
    let breathe = 0.5 - 0.5 * (time * std::f32::consts::TAU / 1.9).cos();
    let alpha = (0.72 + 0.28 * breathe) * 255.0;
    let logo_height = Dp::D96.px() * unit;
    let logo = flex::stack(Align2::CENTER_CENTER, [flex::leaf(Vec2::ZERO).id(0)])
        .padding(Insets {
            bottom: Dp::D24.px() * unit * 2.0,
            ..Insets::default()
        })
        .solve(screen)
        .find(0)
        .unwrap_or(screen);
    let center = logo.center();
    components::brand_lockup(painter, center, logo_height, alpha as u8, assets);
    let radius = Dp::D12.px() * unit;
    let spinner_center = center + Vec2::new(0.0, logo_height * 0.5 + Dp::D36.px() * unit);
    let stroke = Dp::D3.px() * unit;
    painter.circle_stroke(
        spinner_center,
        radius,
        crate::gfx::Stroke::new(stroke, ink(tone::strong())),
    );
    let start = time * std::f32::consts::TAU / 0.8;
    let arc: Vec<Pos2> = (0..=24)
        .map(|step| {
            let angle = start + step as f32 / 24.0 * std::f32::consts::FRAC_PI_2;
            spinner_center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(crate::gfx::Shape::line(
        arc,
        crate::gfx::Stroke::new(stroke, accent()),
    ));
    context.request_repaint();
}

const CARD_BASE: usize = 1000;

pub(crate) fn draw_home_skeleton(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    screen: Rect,
    metrics: UiMetrics,
    margin: f32,
    assets: &mut impl HomeAssets,
) {
    profiles::paint_custom_background(context, painter, screen, assets);
    let time = context.input(|input| input.time) as f32;
    let pulse = 0.5 - 0.5 * (time * std::f32::consts::TAU / 1.6).cos();
    let low = f32::from(tone::raised());
    let high = f32::from(tone::strong());
    let fill = ink((low + (high - low) * pulse) as u8);
    let unit = (screen.width().min(screen.height()) / metrics.control_skeleton_unit_base).clamp(
        metrics.control_skeleton_unit_min,
        metrics.control_skeleton_unit_max,
    );
    let bar = |id: usize, width: f32, height: f32| {
        flex::leaf(Vec2::new(width, height) * unit)
            .width(width * unit)
            .height(height * unit)
            .id(id)
    };
    let gap = metrics.section_gap;
    let card = Vec2::new(metrics.poster_card_width, metrics.poster_card_height);
    let card_gap = gap.max(Dp::D12.px());
    let spacer = |height: f32| flex::leaf(Vec2::new(0.0, height)).height(height);
    let mut rows = vec![
        bar(0, metrics.control_skeleton_title_w, Dp::D32.px()),
        spacer(gap),
        bar(1, metrics.control_skeleton_sub_w, Dp::D14.px()),
        spacer(gap),
        bar(2, metrics.control_skeleton_button_w, Dp::D40.px()),
        spacer(gap * 2.0),
    ];
    let top = screen.top() + screen.height() * metrics.control_skeleton_top;
    let header = flex::column(rows.clone()).outer_size().y;
    let row_height = Dp::D20.px() * unit + card_gap + card.y + card_gap * 2.0;
    let shelves = ((screen.bottom() - top - header) / row_height)
        .ceil()
        .max(0.0) as usize;
    let per_row = ((screen.right() - margin - screen.left() - margin) / (card.x + card_gap))
        .ceil()
        .max(0.0) as usize;
    for shelf in 0..shelves {
        let base = CARD_BASE + shelf * (per_row + 1);
        rows.push(bar(base, metrics.control_skeleton_row_w, Dp::D20.px()));
        rows.push(spacer(card_gap));
        rows.push(
            flex::row((0..per_row).map(|slot| {
                flex::leaf(card)
                    .width(card.x)
                    .height(card.y)
                    .id(base + 1 + slot)
            }))
            .gap(card_gap),
        );
        rows.push(spacer(card_gap * 2.0));
    }
    let column = flex::column(rows);
    let size = column.outer_size();
    let placed = column.solve(Rect::from_min_size(
        Pos2::new(screen.left() + margin, top),
        size,
    ));
    for (id, rect) in placed.tagged() {
        if id < CARD_BASE {
            painter.rect_filled(rect, rect.height() * 0.5, fill);
        } else if (id - CARD_BASE) % (per_row + 1) == 0 {
            painter.rect_filled(rect, rect.height() * 0.5, fill);
        } else {
            painter.rect_filled(rect, Dp::D12.px(), fill);
        }
    }
    context.request_repaint_after(crate::motion::HALF_FRAME_RATE);
}

pub(crate) fn paint_hero_scrim(painter: &crate::gfx::Painter, rect: Rect, metrics: UiMetrics) {
    let (compact, background) = (metrics.compact, metrics.background);
    let placed = flex::column([
        flex::leaf(Vec2::new(
            rect.width(),
            rect.height() * metrics.paint_hero_fade_from,
        )),
        flex::leaf(Vec2::new(
            rect.width(),
            rect.height() * (metrics.paint_hero_dark_at - metrics.paint_hero_fade_from),
        ))
        .id(0),
        flex::fill().id(1),
    ])
    .align(Align::Stretch)
    .solve(rect);
    if !compact {
        paint_horizontal_gradient(
            painter,
            flex::anchored(
                rect,
                Align2::LEFT_TOP,
                Vec2::new(
                    rect.width() * metrics.control_hero_scrim_span,
                    rect.height(),
                ),
                0.0,
            ),
            background.gamma_multiply(0.94),
            Color32::TRANSPARENT,
        );
    }
    let scrim = background.gamma_multiply(0.92);
    paint_vertical_gradient(
        painter,
        placed.find(0).unwrap_or(rect),
        Color32::TRANSPARENT,
        scrim,
    );
    paint_vertical_gradient(painter, placed.find(1).unwrap_or(rect), scrim, background);
}

pub(crate) fn paint_ambient_fade(
    painter: &crate::gfx::Painter,
    rect: Rect,
    screen: Rect,
    assets: &impl HomeAssets,
) {
    let uv_at = |pos: Pos2| {
        Pos2::new(
            ((pos.x - screen.left()) / screen.width()).clamp(0.0, 1.0),
            ((pos.y - screen.top()) / screen.height()).clamp(0.0, 1.0),
        )
    };
    let mut mesh = crate::gfx::epaint::Mesh::with_texture(assets.background());
    for (pos, alpha) in [
        (rect.left_top(), 0),
        (rect.right_top(), 0),
        (rect.right_bottom(), 255),
        (rect.left_bottom(), 255),
    ] {
        mesh.vertices.push(crate::gfx::epaint::Vertex {
            pos,
            uv: uv_at(pos),
            color: Color32::from_white_alpha(alpha),
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(crate::gfx::Shape::mesh(mesh));
}

pub(crate) fn text_width(painter: &crate::gfx::Painter, text: &str, font: FontId) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    painter
        .layout_no_wrap(text.to_owned(), font, Color32::WHITE)
        .size()
        .x
}

pub(crate) fn fill_to(
    painter: &crate::gfx::Painter,
    rect: Rect,
    radius: f32,
    fraction: f32,
    fill: Color32,
) -> Option<crate::gfx::Painter> {
    if fraction <= 0.0 {
        return None;
    }
    let part = flex::anchored(
        rect,
        Align2::LEFT_TOP,
        Vec2::new(rect.width() * fraction.min(1.0), rect.height()),
        0.0,
    );
    let painter = painter.with_clip_rect(part.intersect(painter.clip_rect()));
    painter.rect_filled(rect, radius, fill);
    Some(painter)
}

pub(crate) fn paint_vertical_gradient(
    painter: &crate::gfx::Painter,
    rect: Rect,
    top: Color32,
    bottom: Color32,
) {
    paint_gradient(painter, rect, [top, top, bottom, bottom]);
}

pub(crate) fn paint_horizontal_gradient(
    painter: &crate::gfx::Painter,
    rect: Rect,
    left: Color32,
    right: Color32,
) {
    paint_gradient(painter, rect, [left, right, right, left]);
}

pub(crate) fn paint_gradient(painter: &crate::gfx::Painter, rect: Rect, colors: [Color32; 4]) {
    let mut mesh = crate::gfx::epaint::Mesh::default();
    for (position, color) in [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ]
    .into_iter()
    .zip(colors)
    {
        mesh.vertices.push(crate::gfx::epaint::Vertex {
            pos: position,
            uv: Pos2::ZERO,
            color,
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(crate::gfx::Shape::mesh(mesh));
}

pub(crate) fn paint_circle_texture(
    painter: &crate::gfx::Painter,
    texture: TextureId,
    center: Pos2,
    radius: f32,
) {
    painter.add(
        crate::gfx::epaint::RectShape::filled(
            Rect::from_center_size(center, Vec2::splat(radius * 2.0)),
            radius,
            Color32::WHITE,
        )
        .with_texture(texture, full_uv()),
    );
}

pub(crate) fn paint_trailer(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    texture: crate::gfx::TextureId,
    rect: Rect,
    key: Option<&str>,
    alpha: u8,
) {
    let fade = crate::motion::appear(
        context,
        Id::new(("fluxa-trailer-fade", key)),
        crate::motion::slow() * 2.0,
        crate::motion::Ease::Linear,
    );
    if texture == crate::DOM_TRAILER {
        crate::components::dom_video(&painter.with_clip_rect(rect), rect);
        return;
    }
    painter.with_clip_rect(rect).image(
        texture,
        rect,
        cover_uv([1920, 1080], rect),
        Color32::from_white_alpha((f32::from(alpha) * fade) as u8),
    );
}

pub(crate) fn cover_uv(size: [u32; 2], destination: Rect) -> Rect {
    cover_uv_focus(size, destination, [0.5, 0.5])
}

pub(crate) fn framed_focus(size: [u32; 2], destination: Rect, profile: Option<&[f32]>) -> [f32; 2] {
    let image_aspect = size[0] as f32 / size[1].max(1) as f32;
    let destination_aspect = destination.width() / destination.height().max(1.0);
    match profile {
        Some(profile) if image_aspect > destination_aspect => [
            fluxa_artwork::framing_center(profile, destination_aspect / image_aspect),
            0.5,
        ],
        _ => [0.5, 0.5],
    }
}

pub(crate) fn cover_uv_focus(size: [u32; 2], destination: Rect, focus: [f32; 2]) -> Rect {
    let image_aspect = size[0] as f32 / size[1].max(1) as f32;
    let destination_aspect = destination.width() / destination.height().max(1.0);
    if image_aspect > destination_aspect {
        let visible_width = destination_aspect / image_aspect;
        let left = (focus[0] - visible_width * 0.5).clamp(0.0, 1.0 - visible_width);
        Rect::from_min_max(Pos2::new(left, 0.0), Pos2::new(left + visible_width, 1.0))
    } else {
        let visible_height = image_aspect / destination_aspect.max(f32::EPSILON);
        let top = (focus[1] - visible_height * 0.5).clamp(0.0, 1.0 - visible_height);
        Rect::from_min_max(Pos2::new(0.0, top), Pos2::new(1.0, top + visible_height))
    }
}

pub(crate) fn rating_logo(source: &str, score: &str) -> Option<&'static str> {
    let percent = score.trim_end_matches('%').trim().parse::<f32>().ok();
    Some(match source {
        "IMDb" => "imdb",
        "TMDB" => "tmdb",
        "Trakt" => "trakt",
        "Letterboxd" => "letterboxd",
        "MyAnimeList" => "mal",
        "MDBList" => "mdblist",
        "Metacritic" | "Metacritic Users" => "metacritic",
        "Rotten Tomatoes" => match percent {
            Some(value) if value >= 60.0 => "rt-tomato-fresh",
            Some(_) => "rt-tomato-rotten",
            None => "rt-tomato-empty",
        },
        "Audience" => match percent {
            Some(value) if value < 60.0 => "rt-popcorn-spilled",
            _ => "rt-popcorn-full",
        },
        _ => return None,
    })
}

pub(crate) fn paint_trailer_subtitle(context: &crate::gfx::Context, rect: Rect, text: &str) {
    let painter = context.layer_painter(crate::gfx::LayerId::new(
        crate::gfx::Order::Foreground,
        Id::new("fluxa-trailer-subtitle"),
    ));
    let metrics = crate::metrics::active();
    let size = metrics.paint_text_width.of(rect.width());
    let galley = painter.layout(
        text.to_owned(),
        crate::gfx::FontId::proportional(size),
        text_primary(),
        rect.width() * metrics.control_subtitle_wrap_ratio,
    );
    let placed = flex::stack(
        Align2::CENTER_BOTTOM,
        [
            flex::stack(Align2::CENTER_CENTER, [flex::leaf(galley.size()).id(1)])
                .padding(Insets::xy(Space::S12.px(&metrics), Space::S6.px(&metrics)))
                .id(0),
        ],
    )
    .padding(Insets {
        bottom: rect.height() * metrics.control_subtitle_bottom_ratio,
        ..Insets::default()
    })
    .solve(rect);
    let backdrop = placed.find(0).unwrap_or(rect);
    painter.rect_filled(backdrop, radius::control(), shade(Shade::Veil));
    painter.galley(
        placed.find(1).unwrap_or(backdrop).min,
        galley,
        text_primary(),
    );
}

pub(crate) fn paint_subtitle_cue(painter: &crate::gfx::Painter, rect: Rect, text: &str, lift: f32) {
    let metrics = crate::metrics::active();
    let size = metrics.paint_text_height.of(rect.height());
    let wrap = rect.width() * metrics.control_cue_wrap_ratio;
    let galley = painter.layout(
        text.to_owned(),
        FontId::proportional(size),
        Color32::WHITE,
        wrap,
    );
    let pos = flex::stack(Align2::CENTER_BOTTOM, [flex::leaf(galley.size()).id(0)])
        .padding(Insets {
            bottom: lift,
            ..Insets::default()
        })
        .solve(rect)
        .find(0)
        .unwrap_or(rect)
        .min;
    let shadow = painter.layout(
        text.to_owned(),
        FontId::proportional(size),
        Color32::from_black_alpha(220),
        wrap,
    );
    for offset in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0), (0.0, 3.0)] {
        painter.galley(
            pos + Vec2::new(offset.0, offset.1),
            shadow.clone(),
            Color32::BLACK,
        );
    }
    painter.galley(pos, galley, Color32::WHITE);
}
