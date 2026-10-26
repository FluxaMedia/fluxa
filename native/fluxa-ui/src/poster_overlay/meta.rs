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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingSource {
    Imdb,
    Mdblist,
    Weighted,
}

impl RatingSource {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("mdblist") => RatingSource::Mdblist,
            Some("weighted") => RatingSource::Weighted,
            _ => RatingSource::Imdb,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weights {
    Balanced,
    Critics,
    Audience,
}

impl Weights {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("critics") => Weights::Critics,
            Some("audience") => Weights::Audience,
            _ => Weights::Balanced,
        }
    }

    fn weight(self, source: &str) -> f32 {
        match (self, source) {
            (Weights::Balanced, "imdb") => 30.0,
            (Weights::Balanced, "letterboxd") => 25.0,
            (Weights::Balanced, "tomatoes" | "trakt" | "metacritic") => 15.0,
            (Weights::Balanced, "myanimelist") => 30.0,
            (Weights::Balanced, "anilist") => 10.0,
            (Weights::Balanced, "kitsu") => 5.0,
            (Weights::Critics, "tomatoes" | "metacritic") => 35.0,
            (Weights::Critics, "letterboxd") => 20.0,
            (Weights::Critics, "rogerebert") => 10.0,
            (Weights::Audience, "imdb") => 35.0,
            (Weights::Audience, "letterboxd" | "myanimelist") => 30.0,
            (Weights::Audience, "trakt") => 20.0,
            (Weights::Audience, "popcorn") => 15.0,
            (Weights::Audience, "tmdb") => 10.0,
            _ => 0.0,
        }
    }
}

impl Graded {
    pub(super) fn weighted(&self, weights: Weights) -> Option<f32> {
        let (sum, total) = self
            .ratings
            .iter()
            .map(|(source, score)| (weights.weight(source), *score))
            .filter(|(weight, _)| *weight > 0.0)
            .fold((0.0, 0.0), |(sum, total), (weight, score)| {
                (sum + weight * score, total + weight)
            });
        (total > 0.0).then(|| sum / total / 10.0)
    }
}

pub(super) fn resolve_rating(
    overlays: &PosterOverlays,
    facts: &PosterFacts,
    graded: Option<&Graded>,
) -> (Option<f32>, &'static str) {
    let imdb = (facts.rating, "imdb");
    match overlays.rating_source {
        RatingSource::Imdb => imdb,
        RatingSource::Mdblist => graded
            .and_then(|graded| graded.score)
            .map_or(imdb, |score| (Some(score), "mdblist")),
        RatingSource::Weighted => graded
            .and_then(|graded| graded.weighted(overlays.weights))
            .map(|score| (Some(score), ""))
            .or_else(|| {
                graded
                    .and_then(|graded| graded.score)
                    .map(|score| (Some(score), "mdblist"))
            })
            .unwrap_or(imdb),
    }
}

const BADGE_ORDER: [&str; 5] = ["imdb", "tomatoes", "letterboxd", "metacritic", "trakt"];

pub(super) struct RatingBadge {
    pub(super) logo: &'static str,
    pub(super) short: &'static str,
    pub(super) text: String,
}

impl RatingBadge {
    fn label(&self, has_logo: bool) -> String {
        if has_logo {
            self.text.clone()
        } else {
            format!("{} {}", self.short, self.text)
        }
    }
}

fn rating_badges(graded: &Graded, count: usize) -> Vec<RatingBadge> {
    BADGE_ORDER
        .iter()
        .filter_map(|source| {
            let score = graded.rating(source)?;
            let (logo, short, text) = match *source {
                "imdb" => ("imdb", "IMDb", format!("{:.1}", score / 10.0)),
                "tomatoes" => (
                    if score >= 60.0 {
                        "rt-tomato-fresh"
                    } else {
                        "rt-tomato-rotten"
                    },
                    "RT",
                    format!("{score:.0}%"),
                ),
                "letterboxd" => ("letterboxd", "LB", format!("{:.1}", score / 20.0)),
                "metacritic" => ("metacritic", "MC", format!("{score:.0}")),
                _ => ("trakt", "Trakt", format!("{score:.0}%")),
            };
            Some(RatingBadge { logo, short, text })
        })
        .take(count)
        .collect()
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
    pub(super) badges: Vec<RatingBadge>,
}

pub(super) fn meta(
    facts: &PosterFacts,
    rating: Option<f32>,
    overlays: &PosterOverlays,
    graded: Option<&Graded>,
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
        score: rating.map(|score| overlays.score_scale.format(score)),
        score_color: rating.map_or(Color32::WHITE, |score| score_colors(score).0),
        badges: graded
            .filter(|_| overlays.badge_count > 0)
            .map(|graded| rating_badges(graded, overlays.badge_count))
            .unwrap_or_default(),
    }
}

fn badge_gap(size: f32) -> f32 {
    size * 0.75
}

fn badge_width(painter: &Painter, logo: Option<(TextureId, Vec2)>, text: &str, size: f32) -> f32 {
    let text = painter
        .layout_no_wrap(text.to_owned(), crate::fonts::bold(size), Color32::WHITE)
        .size()
        .x;
    logo.map_or(0.0, |(_, aspect)| aspect.x * size * 1.1 + size * 0.3) + text
}

pub(super) fn meta_width(painter: &Painter, meta: &Meta, size: f32) -> f32 {
    let measure = |text: &str| {
        painter
            .layout_no_wrap(text.to_owned(), crate::fonts::bold(size), Color32::WHITE)
            .size()
            .x
    };
    let mut width = if meta.prefix.is_empty() {
        0.0
    } else {
        measure(&meta.prefix)
    };
    if !meta.badges.is_empty() {
        if width > 0.0 {
            width += size * 0.55;
        }
        for (index, badge) in meta.badges.iter().enumerate() {
            if index > 0 {
                width += badge_gap(size);
            }
            let logo = rating_logo(painter.ctx(), badge.logo);
            width += badge_width(painter, logo, &badge.label(logo.is_some()), size);
        }
    } else if let Some(score) = &meta.score {
        if width > 0.0 {
            width += size * 0.55;
        }
        width += size * 0.95 + size * 0.3 + measure(score);
    }
    width
}

pub(super) fn fit(painter: &Painter, meta: &mut Meta, size: f32, max_width: f32) -> f32 {
    while meta.badges.len() > 1 && meta_width(painter, meta, size) > max_width {
        meta.badges.pop();
    }
    let width = meta_width(painter, meta, size);
    if width > max_width {
        (size * max_width / width).max(7.0)
    } else {
        size
    }
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
        let galley = painter.layout_no_wrap(meta.prefix.clone(), crate::fonts::bold(size), color);
        let width = galley.size().x;
        painter.galley(
            Pos2::new(x, center_y - galley.size().y * 0.5),
            galley,
            color,
        );
        x += width + size * 0.55;
    }
    if !meta.badges.is_empty() {
        for badge in &meta.badges {
            let logo = rating_logo(painter.ctx(), badge.logo);
            let drawn = draw_logo(painter, logo, Pos2::new(x, center_y), size * 1.1);
            x += if drawn > 0.0 { drawn + size * 0.3 } else { 0.0 };
            let galley = painter.layout_no_wrap(
                badge.label(logo.is_some()),
                crate::fonts::bold(size),
                color,
            );
            let width = galley.size().x;
            painter.galley(
                Pos2::new(x, center_y - galley.size().y * 0.5),
                galley,
                color,
            );
            x += width + badge_gap(size);
        }
    } else if let Some(score) = &meta.score {
        star(
            painter,
            Pos2::new(x + size * 0.45, center_y - size * 0.02),
            size * 0.47,
            meta.score_color,
        );
        x += size * 0.95 + size * 0.3;
        let galley = painter.layout_no_wrap(score.clone(), crate::fonts::bold(size), color);
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
    mut meta: Meta,
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
    let size = fit(
        painter,
        &mut meta,
        (height * 0.5).clamp(11.0, 28.0),
        band.width() * 0.94,
    );
    let width = meta_width(painter, &meta, size);
    meta_row(
        painter,
        band.center().x - width * 0.5,
        band.center().y + size * 0.08,
        size,
        &meta,
        Color32::from_gray(235),
    );
    height / rect.height()
}

pub(super) fn score_clean(
    painter: &Painter,
    rect: Rect,
    mut meta: Meta,
    overlays: &PosterOverlays,
) -> f32 {
    let size = fit(
        painter,
        &mut meta,
        (rect.width() * 0.1 * overlays.scale).clamp(11.0, 30.0),
        rect.width() * 0.92,
    );
    let width = meta_width(painter, &meta, size);
    let center_y = rect.bottom() - rect.height() * 0.06 - size * 0.5;
    meta_row(
        painter,
        rect.center().x - width * 0.5,
        center_y,
        size,
        &meta,
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
