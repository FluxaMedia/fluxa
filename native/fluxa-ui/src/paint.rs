use super::*;

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
    painter: &egui::Painter,
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

pub(crate) fn truncate_to_width(
    painter: &egui::Painter,
    text: &str,
    font: &FontId,
    max_width: f32,
) -> String {
    if text.is_empty() || max_width <= 0.0 {
        return String::new();
    }
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

pub(crate) fn paint_brand_ambient(painter: &egui::Painter, screen: Rect, assets: &impl HomeAssets) {
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

pub(crate) fn paint_ambient(painter: &egui::Painter, screen: Rect, assets: &impl HomeAssets) {
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    painter.image(assets.background(), screen, uv, Color32::WHITE);
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
    context: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    profiles::paint_custom_background(context, painter, screen, assets);
    let time = context.input(|input| input.time) as f32;
    let unit = (screen.width().min(screen.height()) / 900.0).clamp(0.7, 1.6);
    let breathe = 0.5 - 0.5 * (time * std::f32::consts::TAU / 1.9).cos();
    let alpha = (0.72 + 0.28 * breathe) * 255.0;
    let logo_height = 96.0 * unit;
    let center = screen.center() - Vec2::new(0.0, 24.0 * unit);
    components::brand_lockup(painter, center, logo_height, alpha as u8, assets);
    let radius = 12.0 * unit;
    let spinner_center = center + Vec2::new(0.0, logo_height * 0.5 + 36.0 * unit);
    let stroke = 3.0 * unit;
    painter.circle_stroke(
        spinner_center,
        radius,
        egui::Stroke::new(stroke, ink(tone::strong())),
    );
    let start = time * std::f32::consts::TAU / 0.8;
    let arc: Vec<Pos2> = (0..=24)
        .map(|step| {
            let angle = start + step as f32 / 24.0 * std::f32::consts::FRAC_PI_2;
            spinner_center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(egui::Shape::line(arc, egui::Stroke::new(stroke, accent())));
    context.request_repaint();
}

pub(crate) fn paint_hero_scrim(
    painter: &egui::Painter,
    rect: Rect,
    compact: bool,
    background: Color32,
) {
    if !compact {
        paint_horizontal_gradient(
            painter,
            Rect::from_min_max(
                rect.left_top(),
                Pos2::new(rect.left() + rect.width() * 0.72, rect.bottom()),
            ),
            background.gamma_multiply(0.94),
            Color32::TRANSPARENT,
        );
    }
    let fade_from = rect.top() + rect.height() * if compact { 0.1 } else { 0.3 };
    let dark_at = rect.top() + rect.height() * if compact { 0.6 } else { 0.8 };
    let scrim = background.gamma_multiply(0.92);
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(
            Pos2::new(rect.left(), fade_from),
            Pos2::new(rect.right(), dark_at),
        ),
        Color32::TRANSPARENT,
        scrim,
    );
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(Pos2::new(rect.left(), dark_at), rect.right_bottom()),
        scrim,
        background,
    );
}

pub(crate) fn paint_ambient_fade(
    painter: &egui::Painter,
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
    let mut mesh = egui::epaint::Mesh::with_texture(assets.background());
    for (pos, alpha) in [
        (rect.left_top(), 0),
        (rect.right_top(), 0),
        (rect.right_bottom(), 255),
        (rect.left_bottom(), 255),
    ] {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: uv_at(pos),
            color: Color32::from_white_alpha(alpha),
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
}

pub(crate) fn text_width(painter: &egui::Painter, text: &str, font: FontId) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    painter
        .layout_no_wrap(text.to_owned(), font, Color32::WHITE)
        .size()
        .x
}

pub(crate) fn fill_to(
    painter: &egui::Painter,
    rect: Rect,
    radius: f32,
    fraction: f32,
    fill: Color32,
) -> Option<egui::Painter> {
    if fraction <= 0.0 {
        return None;
    }
    let part = Rect::from_min_max(
        rect.min,
        Pos2::new(
            rect.left() + rect.width() * fraction.min(1.0),
            rect.bottom(),
        ),
    );
    let painter = painter.with_clip_rect(part.intersect(painter.clip_rect()));
    painter.rect_filled(rect, radius, fill);
    Some(painter)
}

pub(crate) fn paint_vertical_gradient(
    painter: &egui::Painter,
    rect: Rect,
    top: Color32,
    bottom: Color32,
) {
    paint_gradient(painter, rect, [top, top, bottom, bottom]);
}

pub(crate) fn paint_horizontal_gradient(
    painter: &egui::Painter,
    rect: Rect,
    left: Color32,
    right: Color32,
) {
    paint_gradient(painter, rect, [left, right, right, left]);
}

pub(crate) fn paint_gradient(painter: &egui::Painter, rect: Rect, colors: [Color32; 4]) {
    let mut mesh = egui::epaint::Mesh::default();
    for (position, color) in [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ]
    .into_iter()
    .zip(colors)
    {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: position,
            uv: Pos2::ZERO,
            color,
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
}

pub(crate) fn paint_circle_texture(
    painter: &egui::Painter,
    texture: TextureId,
    center: Pos2,
    radius: f32,
) {
    let segments = 40u32;
    let mut mesh = egui::epaint::Mesh::with_texture(texture);
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: Pos2::new(0.5, 0.5),
        color: Color32::WHITE,
    });
    for segment in 0..segments {
        let angle = std::f32::consts::TAU * segment as f32 / segments as f32;
        let (sin, cos) = angle.sin_cos();
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + Vec2::new(cos, sin) * radius,
            uv: Pos2::new(0.5 + cos * 0.5, 0.5 + sin * 0.5),
            color: Color32::WHITE,
        });
    }
    for segment in 0..segments {
        let current = segment + 1;
        let next = ((segment + 1) % segments) + 1;
        mesh.indices.extend([0, current, next]);
    }
    painter.add(egui::Shape::mesh(mesh));
}

pub(crate) fn paint_trailer(
    context: &egui::Context,
    painter: &egui::Painter,
    texture: egui::TextureId,
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
    let image_aspect = size[0] as f32 / size[1].max(1) as f32;
    let destination_aspect = destination.width() / destination.height().max(1.0);
    if image_aspect > destination_aspect {
        let visible_width = destination_aspect / image_aspect;
        let crop = (1.0 - visible_width) * 0.5;
        Rect::from_min_max(Pos2::new(crop, 0.0), Pos2::new(1.0 - crop, 1.0))
    } else {
        let visible_height = image_aspect / destination_aspect.max(f32::EPSILON);
        let top = (1.0 - visible_height) * 0.5;
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

pub(crate) fn paint_trailer_subtitle(context: &egui::Context, rect: Rect, text: &str) {
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-trailer-subtitle"),
    ));
    let size = (rect.width() * 0.022).clamp(15.0, 26.0);
    let galley = painter.layout(
        text.to_owned(),
        egui::FontId::proportional(size),
        text_primary(),
        rect.width() * 0.6,
    );
    let size = galley.size() + Vec2::new(24.0, 12.0);
    let bottom = rect.bottom() - rect.height() * 0.06;
    let backdrop = Rect::from_center_size(Pos2::new(rect.center().x, bottom - size.y * 0.5), size);
    painter.rect_filled(backdrop, radius::control(), shade(Shade::Veil));
    painter.galley(
        backdrop.center() - galley.size() * 0.5,
        galley,
        text_primary(),
    );
}

pub(crate) fn paint_subtitle_cue(painter: &egui::Painter, rect: Rect, text: &str, lift: f32) {
    let size = (rect.height() * 0.052).clamp(18.0, 60.0);
    let galley = painter.layout(
        text.to_owned(),
        FontId::proportional(size),
        Color32::WHITE,
        rect.width() * 0.8,
    );
    let pos = Pos2::new(
        rect.center().x - galley.size().x / 2.0,
        rect.bottom() - lift - galley.size().y,
    );
    let shadow = painter.layout(
        text.to_owned(),
        FontId::proportional(size),
        Color32::from_black_alpha(220),
        rect.width() * 0.8,
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
