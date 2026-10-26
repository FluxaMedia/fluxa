use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaOrder {
    YearGenreScore,
    GenreYearScore,
}

impl MetaOrder {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("genre_year_score") => MetaOrder::GenreYearScore,
            _ => MetaOrder::YearGenreScore,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreScale {
    Ten,
    Hundred,
}

impl ScoreScale {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("hundred") => ScoreScale::Hundred,
            _ => ScoreScale::Ten,
        }
    }

    pub(super) fn format(self, score: f32) -> String {
        match self {
            ScoreScale::Ten => format!("{score:.1}"),
            ScoreScale::Hundred => format!("{:.0}", score * 10.0),
        }
    }
}

pub(super) fn star(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let points = (0..10)
        .map(|index| {
            let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::PI / 5.0;
            let length = if index % 2 == 0 {
                radius
            } else {
                radius * 0.42
            };
            center + Vec2::angled(angle) * length
        })
        .collect::<Vec<_>>();
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub(super) struct Meta {
    pub(super) prefix: String,
    pub(super) score: Option<String>,
    pub(super) score_color: Color32,
}

pub(super) fn meta(
    facts: &PosterFacts,
    rating: Option<f32>,
    overlays: &PosterOverlays,
    hidden_score: bool,
) -> Meta {
    let (year, genre) = (facts.year.as_str(), facts.genre.as_str());
    let parts: [&str; 2] = match overlays.meta_order {
        MetaOrder::YearGenreScore => [year, genre],
        MetaOrder::GenreYearScore => [genre, year],
    };
    Meta {
        prefix: parts
            .iter()
            .filter(|part| !part.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" · "),
        score: rating
            .filter(|_| !hidden_score)
            .map(|score| overlays.score_scale.format(score)),
        score_color: rating.map_or(Color32::WHITE, |score| score_colors(score).0),
    }
}

pub(super) fn meta_width(painter: &Painter, meta: &Meta, size: f32) -> f32 {
    let measure = |text: &str| {
        painter
            .layout_no_wrap(text.to_owned(), FontId::proportional(size), Color32::WHITE)
            .size()
            .x
    };
    let mut width = if meta.prefix.is_empty() {
        0.0
    } else {
        measure(&meta.prefix)
    };
    if let Some(score) = &meta.score {
        if width > 0.0 {
            width += size * 0.55;
        }
        width += size * 0.95 + size * 0.3 + measure(score);
    }
    width
}

pub(super) fn meta_row(
    painter: &Painter,
    left: f32,
    center_y: f32,
    size: f32,
    meta: &Meta,
    color: Color32,
) {
    let mut x = left;
    if !meta.prefix.is_empty() {
        let galley = painter.layout_no_wrap(meta.prefix.clone(), FontId::proportional(size), color);
        let width = galley.size().x;
        painter.galley(
            Pos2::new(x, center_y - galley.size().y * 0.5),
            galley,
            color,
        );
        x += width + size * 0.55;
    }
    if let Some(score) = &meta.score {
        star(
            painter,
            Pos2::new(x + size * 0.45, center_y - size * 0.02),
            size * 0.47,
            meta.score_color,
        );
        x += size * 0.95 + size * 0.3;
        let galley = painter.layout_no_wrap(score.clone(), FontId::proportional(size), color);
        painter.galley(
            Pos2::new(x, center_y - galley.size().y * 0.5),
            galley,
            color,
        );
    }
}

pub(super) fn score_band(
    painter: &Painter,
    rect: Rect,
    meta: &Meta,
    rating: Option<f32>,
    overlays: &PosterOverlays,
    tint: Option<[u8; 3]>,
    radius: f32,
) -> f32 {
    let height = (rect.height() * 0.095 * overlays.scale).max(24.0);
    let band = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - height), rect.max);
    let fill = match tint {
        Some([r, g, b]) => {
            let dim = |c: u8| (c as f32 * 0.42) as u8;
            Color32::from_rgba_unmultiplied(dim(r), dim(g), dim(b), 244)
        }
        None => Color32::from_rgba_unmultiplied(12, 12, 14, 240),
    };
    let corner = radius as u8;
    painter.rect_filled(
        band,
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: corner,
            se: corner,
        },
        fill,
    );
    if let Some(score) = rating {
        let line = (rect.height() * 0.006).max(2.0);
        let track = Rect::from_min_size(band.min, Vec2::new(band.width(), line));
        painter.rect_filled(
            track,
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 36),
        );
        painter.rect_filled(
            Rect::from_min_size(
                band.min,
                Vec2::new(band.width() * (score / 10.0).clamp(0.0, 1.0), line),
            ),
            0.0,
            score_colors(score).0,
        );
    }
    let size = (height * 0.46).clamp(10.0, 24.0);
    let width = meta_width(painter, meta, size);
    meta_row(
        painter,
        band.center().x - width * 0.5,
        band.center().y + size * 0.08,
        size,
        meta,
        Color32::from_gray(235),
    );
    height / rect.height()
}

pub(super) fn score_clean(
    painter: &Painter,
    rect: Rect,
    meta: &Meta,
    overlays: &PosterOverlays,
) -> f32 {
    let size = (rect.width() * 0.095 * overlays.scale).clamp(10.0, 26.0);
    let width = meta_width(painter, meta, size);
    let center_y = rect.bottom() - rect.height() * 0.06 - size * 0.5;
    meta_row(
        painter,
        rect.center().x - width * 0.5,
        center_y,
        size,
        meta,
        Color32::from_gray(238),
    );
    (rect.bottom() - center_y + size) / rect.height()
}

pub(super) fn top_vignette(painter: &Painter, rect: Rect, strength: f32) {
    const STEPS: usize = 10;
    let bottom = rect.top() + rect.height() * (0.18 + strength * 0.2);
    let mut mesh = Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let y = rect.top() + (bottom - rect.top()) * t;
        let alpha = 215.0 * strength * (1.0 - t).powf(1.6);
        let color = Color32::from_rgba_unmultiplied(0, 0, 0, alpha as u8);
        mesh.colored_vertex(Pos2::new(rect.left(), y), color);
        mesh.colored_vertex(Pos2::new(rect.right(), y), color);
        if step > 0 {
            let base = (step as u32 - 1) * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
    }
    painter.add(Shape::mesh(mesh));
}
