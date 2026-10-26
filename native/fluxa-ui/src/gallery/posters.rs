use super::*;
use crate::poster_overlay::{
    Enrichment, Fade, MetaOrder, NotchLook, Placement, PosterFacts, PosterOverlays, PosterStatus,
    RatingSource, SashPriority, SashSources, SashStyle, ScoreScale, Weights, set_poster_enrichment,
    set_poster_overlays,
};

struct Sample {
    caption: &'static str,
    id: &'static str,
    rating: Option<f32>,
    status: Option<PosterStatus>,
    year: &'static str,
    genre: &'static str,
    style: SashStyle,
    look: NotchLook,
    rating_style: Option<Placement>,
    priority: SashPriority,
    trending: bool,
}

const SAMPLES: [Sample; 8] = [
    Sample {
        caption: "diagonal status",
        id: "tt1",
        rating: Some(8.1),
        status: Some(PosterStatus::NewEpisode),
        year: "2024",
        genre: "Drama",
        style: SashStyle::Diagonal,
        look: NotchLook::Frosted,
        rating_style: Some(Placement::Bar),
        priority: SashPriority::StatusFirst,
        trending: false,
    },
    Sample {
        caption: "notch frosted award",
        id: "tt2",
        rating: Some(8.4),
        status: None,
        year: "2006",
        genre: "Fantasy",
        style: SashStyle::Notch,
        look: NotchLook::Frosted,
        rating_style: Some(Placement::Band),
        priority: SashPriority::AwardsFirst,
        trending: false,
    },
    Sample {
        caption: "notch gold",
        id: "tt2",
        rating: Some(6.6),
        status: Some(PosterStatus::Premiere),
        year: "2025",
        genre: "Sci-Fi",
        style: SashStyle::Notch,
        look: NotchLook::Gold,
        rating_style: Some(Placement::Minimal),
        priority: SashPriority::StatusFirst,
        trending: false,
    },
    Sample {
        caption: "chip tinted trending",
        id: "tt3",
        rating: Some(7.4),
        status: None,
        year: "2026",
        genre: "Horror",
        style: SashStyle::Chip,
        look: NotchLook::Tinted,
        rating_style: Some(Placement::Clean),
        priority: SashPriority::TrendingFirst,
        trending: true,
    },
    Sample {
        caption: "edge silver",
        id: "tt2",
        rating: Some(8.8),
        status: None,
        year: "1998",
        genre: "History",
        style: SashStyle::Edge,
        look: NotchLook::Silver,
        rating_style: Some(Placement::Number),
        priority: SashPriority::AwardsFirst,
        trending: false,
    },
    Sample {
        caption: "notch black finale",
        id: "tt4",
        rating: Some(9.0),
        status: Some(PosterStatus::NewEpisode),
        year: "2019",
        genre: "Crime",
        style: SashStyle::Notch,
        look: NotchLook::Black,
        rating_style: Some(Placement::TopLeft),
        priority: SashPriority::StatusFirst,
        trending: false,
    },
    Sample {
        caption: "upcoming chip",
        id: "tt5",
        rating: None,
        status: Some(PosterStatus::Upcoming),
        year: "2027",
        genre: "Comedy",
        style: SashStyle::Chip,
        look: NotchLook::Frosted,
        rating_style: None,
        priority: SashPriority::StatusFirst,
        trending: false,
    },
    Sample {
        caption: "mini series",
        id: "tt6",
        rating: Some(7.9),
        status: None,
        year: "2023",
        genre: "Thriller",
        style: SashStyle::Notch,
        look: NotchLook::Silver,
        rating_style: Some(Placement::Bar),
        priority: SashPriority::StatusFirst,
        trending: false,
    },
];

const ART: [([u8; 3], [u8; 3]); 4] = [
    ([32, 84, 70], [8, 20, 18]),
    ([120, 60, 40], [26, 12, 10]),
    ([40, 60, 120], [10, 12, 30]),
    ([96, 40, 110], [22, 8, 26]),
];

fn overlays(sample: &Sample, language: &str) -> PosterOverlays {
    PosterOverlays {
        rating: sample.rating_style,
        status: Some(Placement::Sash),
        scale: 1.0,
        labels: crate::poster_overlay::label_keys()
            .map(|(slug, key)| (slug, localized(key, language)))
            .to_vec(),
        language: language.to_owned(),
        sources: SashSources {
            awards: true,
            trending: sample.trending,
            lifecycle: true,
            ..Default::default()
        },
        priority: sample.priority,
        sash_style: sample.style,
        notch_look: sample.look,
        meta_order: MetaOrder::YearGenreScore,
        score_scale: ScoreScale::Hundred,
        top_fade: 0.7,
        mute_unreleased: true,
        release_disc: true,
        band_light: false,
        watched: false,
        progress: false,
        saved: false,
        fade: Fade {
            tint: true,
            reach: 0.5,
            alpha: 225.0,
        },
        textless: false,
        age_country: None,
        quality: false,
        age: true,
        rating_source: RatingSource::Weighted,
        weights: Weights::Balanced,
        badge_count: 2,
        template: None,
    }
}

pub(super) fn draw(context: &egui::Context, viewport: Viewport, metrics: UiMetrics) {
    let mut enrichment = Enrichment::default();
    enrichment.add_mdblist(&[
        serde_json::json!({
            "ids": {"imdb": "tt2"},
            "keywords": [{"name": "emmy-winner"}, {"name": "best-picture-nominated"}],
            "certification": "PG-13",
            "ratings": [
                {"source": "imdb", "score": 84},
                {"source": "tomatoes", "score": 91},
                {"source": "letterboxd", "score": 86},
            ],
        }),
        serde_json::json!({
            "ids": {"imdb": "tt3"},
            "ratings": [{"source": "imdb", "score": 74}, {"source": "metacritic", "score": 68}],
        }),
        serde_json::json!({"ids": {"imdb": "tt6"}, "status": "Ended", "certification": "TV-MA"}),
        serde_json::json!({"ids": {"imdb": "tt4"}, "status": "Ended"}),
    ]);
    enrichment.set_trending(&[
        serde_json::json!({"id": 3, "title": "Obsession", "release_date": "2026-01-01"}),
    ]);
    set_poster_enrichment(context, std::sync::Arc::new(enrichment));
    let columns = if viewport.width < 600.0 { 2 } else { 4 };
    let gap = Space::S16.px(&metrics);
    let width = ((viewport.width - gap * (columns as f32 + 1.0)) / columns as f32).min(220.0);
    let height = width * 1.5;
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-gallery-posters"),
    ));
    let radius = Space::S12.px(&metrics);
    for (index, sample) in SAMPLES.iter().enumerate() {
        let column = (index % columns) as f32;
        let row = (index / columns) as f32;
        let origin = Pos2::new(
            gap + column * (width + gap),
            gap + row * (height + gap + 22.0),
        );
        let rect = Rect::from_min_size(origin, Vec2::new(width, height));
        let (top, bottom) = ART[index % ART.len()];
        crate::paint_vertical_gradient(
            &painter,
            rect,
            Color32::from_rgb(top[0], top[1], top[2]),
            Color32::from_rgb(bottom[0], bottom[1], bottom[2]),
        );
        set_poster_overlays(context, Some(overlays(sample, "en")));
        let card = HomeCard {
            id: Some(sample.id.to_owned()),
            overlay: PosterFacts {
                rating: sample.rating,
                status: sample.status,
                caption: format!("{} · {}", sample.genre, sample.year),
                genre: sample.genre.to_owned(),
                year: sample.year.to_owned(),
                imdb: Some(sample.id.to_owned()),
                tmdb: Some(3),
                title_key: "obsession:2026".to_owned(),
                series: sample.status != Some(PosterStatus::Upcoming),
                release: (sample.status == Some(PosterStatus::Upcoming)).then_some((11, 27)),
                ..Default::default()
            },
            ..Default::default()
        };
        crate::poster_overlay::paint(
            &painter,
            rect,
            &card,
            radius,
            Some([top, [top[0], top[1], top[2]]]),
        );
        painter.text(
            Pos2::new(rect.left(), rect.bottom() + 4.0),
            Align2::LEFT_TOP,
            sample.caption,
            FontId::proportional(12.0),
            text_primary(),
        );
    }
    set_poster_overlays(context, None);
}
