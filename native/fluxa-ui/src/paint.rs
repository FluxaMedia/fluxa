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
    let tint = Color32::from_white_alpha(alpha as u8);
    let mark_size = 64.0 * unit;
    let gap = 14.0 * unit;
    let galley =
        painter.layout_no_wrap("fluxa".to_owned(), FontId::proportional(52.0 * unit), tint);
    let brand_width = mark_size + gap + galley.size().x;
    let center = screen.center() - Vec2::new(0.0, 24.0 * unit);
    let left = center.x - brand_width * 0.5;
    if let Some(mark) = assets.brand_mark() {
        painter.image(
            mark,
            Rect::from_center_size(
                Pos2::new(left + mark_size * 0.5, center.y),
                Vec2::splat(mark_size),
            ),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            tint,
        );
    }
    let text_pos = Pos2::new(left + mark_size + gap, center.y - galley.size().y * 0.5);
    painter.galley(text_pos, galley, tint);
    let radius = 12.0 * unit;
    let spinner_center = center + Vec2::new(0.0, mark_size * 0.5 + 44.0 * unit);
    let stroke = 3.0 * unit;
    painter.circle_stroke(
        spinner_center,
        radius,
        egui::Stroke::new(stroke, Color32::from_white_alpha(50)),
    );
    let start = time * std::f32::consts::TAU / 0.8;
    let arc: Vec<Pos2> = (0..=24)
        .map(|step| {
            let angle = start + step as f32 / 24.0 * std::f32::consts::FRAC_PI_2;
            spinner_center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(egui::Shape::line(
        arc,
        egui::Stroke::new(stroke, Color32::from_white_alpha(220)),
    ));
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
        Rect::from_min_max(Pos2::new(rect.left(), fade_from), Pos2::new(rect.right(), dark_at)),
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
    let fade =
        context.animate_bool_with_time(Id::new(("fluxa-trailer-fade", key, texture)), true, 0.6);
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
