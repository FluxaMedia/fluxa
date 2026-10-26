use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Award {
    Oscar,
    Emmy,
    Globe,
    Festival,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SashKind {
    Award(Award, bool),
    Trending(u32),
    Status(PosterStatus),
    Foreign,
    MustSee,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SashStyle {
    Diagonal,
    Notch,
    Chip,
    Edge,
}

impl SashStyle {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("notch") => SashStyle::Notch,
            Some("chip") => SashStyle::Chip,
            Some("edge") => SashStyle::Edge,
            _ => SashStyle::Diagonal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotchLook {
    Frosted,
    Black,
    Silver,
    Gold,
    Tinted,
}

impl NotchLook {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("black") => NotchLook::Black,
            Some("silver") => NotchLook::Silver,
            Some("gold") => NotchLook::Gold,
            Some("tinted") => NotchLook::Tinted,
            _ => NotchLook::Frosted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SashPriority {
    StatusFirst,
    AwardsFirst,
    TrendingFirst,
}

impl SashPriority {
    pub(super) fn parse(value: Option<&str>) -> Self {
        match value {
            Some("awards_first") => SashPriority::AwardsFirst,
            Some("trending_first") => SashPriority::TrendingFirst,
            _ => SashPriority::StatusFirst,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SashSources {
    pub awards: bool,
    pub trending: bool,
    pub lifecycle: bool,
    pub foreign: bool,
    pub must_see: bool,
}

pub(crate) fn label_keys() -> [(&'static str, &'static str); 21] {
    [
        ("upcoming", "poster.badge.upcoming"),
        ("new_season", "poster.badge.new_season"),
        ("new_episode", "poster.badge.new_episode"),
        ("new_release", "poster.badge.new_release"),
        ("premiere", "poster.badge.premiere"),
        ("finale", "poster.badge.finale"),
        ("ended", "poster.badge.ended"),
        ("cancelled", "poster.badge.cancelled"),
        ("mini_series", "poster.badge.mini_series"),
        ("oscar_winner", "poster.badge.oscar_winner"),
        ("oscar_nominee", "poster.badge.oscar_nominee"),
        ("emmy_winner", "poster.badge.emmy_winner"),
        ("emmy_nominee", "poster.badge.emmy_nominee"),
        ("globe_winner", "poster.badge.globe_winner"),
        ("globe_nominee", "poster.badge.globe_nominee"),
        ("festival_winner", "poster.badge.festival_winner"),
        ("festival_nominee", "poster.badge.festival_nominee"),
        ("trending", "poster.badge.trending"),
        ("foreign", "poster.badge.foreign"),
        ("must_see", "poster.badge.must_see"),
        ("watchlist", "poster.badge.watchlist"),
    ]
}

impl SashKind {
    fn label_key(self) -> &'static str {
        match self {
            SashKind::Award(Award::Oscar, true) => "oscar_winner",
            SashKind::Award(Award::Oscar, false) => "oscar_nominee",
            SashKind::Award(Award::Emmy, true) => "emmy_winner",
            SashKind::Award(Award::Emmy, false) => "emmy_nominee",
            SashKind::Award(Award::Globe, true) => "globe_winner",
            SashKind::Award(Award::Globe, false) => "globe_nominee",
            SashKind::Award(Award::Festival, true) => "festival_winner",
            SashKind::Award(Award::Festival, false) => "festival_nominee",
            SashKind::Trending(_) => "trending",
            SashKind::Status(status) => status.slug(),
            SashKind::Foreign => "foreign",
            SashKind::MustSee => "must_see",
        }
    }

    pub(super) fn label(self, overlays: &PosterOverlays) -> String {
        let text = overlays.label(self.label_key());
        match self {
            SashKind::Trending(rank) => format!("#{rank} {text}"),
            _ => text,
        }
    }

    pub(super) fn colors(self) -> (Color32, Color32) {
        match self {
            SashKind::Status(status) => sash_colors(status),
            SashKind::Trending(_) => trending_colors(),
            SashKind::Award(_, won) => award_colors(won),
            SashKind::Foreign => (
                Color32::from_rgb(70, 90, 150),
                Color32::from_rgb(170, 190, 245),
            ),
            SashKind::MustSee => (
                Color32::from_rgb(30, 120, 80),
                Color32::from_rgb(110, 225, 170),
            ),
        }
    }
}

pub(super) fn pick(
    overlays: &PosterOverlays,
    facts: &PosterFacts,
    graded: Option<&Graded>,
    trending: Option<u32>,
) -> Option<SashKind> {
    let sources = overlays.sources;
    let mut awards = Vec::new();
    if sources.awards {
        if let Some(won) = graded.and_then(|graded| graded.oscar) {
            awards.push(SashKind::Award(Award::Oscar, won));
        }
        for (award, won) in graded.map(|graded| graded.awards.as_slice()).unwrap_or(&[]) {
            let kind = SashKind::Award(*award, *won);
            if !awards.contains(&kind) {
                awards.push(kind);
            }
        }
        awards.sort_by_key(|kind| match kind {
            SashKind::Award(_, won) => !won,
            _ => true,
        });
    }
    let trending = trending
        .filter(|_| sources.trending)
        .map(SashKind::Trending);
    let status = overlays
        .status
        .filter(|placement| *placement == Placement::Sash)
        .and(facts.status)
        .map(
            |status| match (status, graded.is_some_and(|graded| graded.is_ended())) {
                (PosterStatus::NewEpisode, true) => PosterStatus::Finale,
                (status, _) => status,
            },
        )
        .map(SashKind::Status);
    let lifecycle = sources
        .lifecycle
        .then(|| graded.and_then(|graded| graded.lifecycle(facts)))
        .flatten()
        .map(SashKind::Status);
    let foreign = (sources.foreign
        && graded.is_some_and(|graded| graded.is_foreign(&overlays.language)))
    .then_some(SashKind::Foreign);
    let must_see = (sources.must_see
        && graded.is_some_and(|graded| {
            graded
                .rating("metacritic")
                .is_some_and(|score| score >= 81.0)
        }))
    .then_some(SashKind::MustSee);

    let mut order: Vec<Option<SashKind>> = Vec::new();
    match overlays.priority {
        SashPriority::StatusFirst => {
            order.push(status);
            order.push(trending);
            order.extend(awards.into_iter().map(Some));
        }
        SashPriority::AwardsFirst => {
            order.extend(awards.into_iter().map(Some));
            order.push(trending);
            order.push(status);
        }
        SashPriority::TrendingFirst => {
            order.push(trending);
            order.extend(awards.into_iter().map(Some));
            order.push(status);
        }
    }
    order.extend([must_see, foreign, lifecycle]);
    order.into_iter().flatten().next()
}

impl Graded {
    pub(super) fn rating(&self, source: &str) -> Option<f32> {
        self.ratings
            .iter()
            .find(|(name, _)| *name == source)
            .map(|(_, score)| *score)
    }

    fn is_ended(&self) -> bool {
        self.status
            .as_deref()
            .is_some_and(|status| status.contains("ended") || status.contains("cancel"))
    }

    fn is_foreign(&self, language: &str) -> bool {
        let own = language.split(['-', '_']).next().unwrap_or(language);
        self.language
            .as_deref()
            .is_some_and(|original| !original.is_empty() && original != own)
    }

    fn lifecycle(&self, facts: &PosterFacts) -> Option<PosterStatus> {
        let status = self.status.as_deref()?;
        if status.contains("cancel") {
            Some(PosterStatus::Cancelled)
        } else if status.contains("ended") {
            let structure = facts.structure;
            let mini = facts.series
                && structure.seasons == 1
                && (1..=10).contains(&structure.episodes)
                && !structure.pending;
            Some(if mini {
                PosterStatus::MiniSeries
            } else {
                PosterStatus::Ended
            })
        } else {
            None
        }
    }
}

struct Look {
    fill: Color32,
    border: Option<Color32>,
    text: Color32,
}

fn look(kind: NotchLook, tint: Option<[u8; 3]>, accent: Color32) -> Look {
    let black = Color32::from_rgba_unmultiplied(10, 10, 12, 232);
    match kind {
        NotchLook::Frosted => Look {
            fill: Color32::from_rgba_unmultiplied(34, 34, 40, 205),
            border: Some(Color32::from_rgba_unmultiplied(255, 255, 255, 80)),
            text: Color32::WHITE,
        },
        NotchLook::Black => Look {
            fill: black,
            border: None,
            text: Color32::WHITE,
        },
        NotchLook::Silver => Look {
            fill: black,
            border: Some(Color32::from_rgb(192, 192, 200)),
            text: Color32::from_rgb(236, 236, 240),
        },
        NotchLook::Gold => Look {
            fill: black,
            border: Some(Color32::from_rgb(212, 175, 55)),
            text: Color32::from_rgb(244, 220, 140),
        },
        NotchLook::Tinted => {
            let [r, g, b] = tint.unwrap_or([accent.r(), accent.g(), accent.b()]);
            let dim = |c: u8| (c as f32 * 0.45) as u8;
            Look {
                fill: Color32::from_rgba_unmultiplied(dim(r), dim(g), dim(b), 238),
                border: Some(Color32::from_rgba_unmultiplied(r, g, b, 150)),
                text: Color32::WHITE,
            }
        }
    }
}

pub(super) fn draw(
    painter: &Painter,
    rect: Rect,
    overlays: &PosterOverlays,
    kind: SashKind,
    tint: Option<[u8; 3]>,
) -> Option<Rect> {
    let label = kind.label(overlays);
    match overlays.sash_style {
        SashStyle::Diagonal => {
            sash(painter, rect, kind.colors(), &label);
            None
        }
        style => {
            let look = look(overlays.notch_look, tint, kind.colors().1);
            Some(tab(painter, rect, &label, &look, style, overlays.scale))
        }
    }
}

fn tab(
    painter: &Painter,
    rect: Rect,
    label: &str,
    look: &Look,
    style: SashStyle,
    scale: f32,
) -> Rect {
    let size = (rect.width() * 0.062).clamp(8.0, 14.0) * scale;
    let galley = painter.layout_no_wrap(label.to_owned(), FontId::proportional(size), look.text);
    let max = rect.width() * if style == SashStyle::Edge { 0.8 } else { 0.78 };
    let text_width = galley.size().x.min(max - size * 1.6);
    let height = size * 2.1;
    let width = text_width + size * 1.6;
    let stroke = look.border.map_or(Stroke::NONE, |color| {
        Stroke::new(stroke::hairline().max(1.0), color)
    });
    let galley = if galley.size().x > text_width {
        painter.layout(
            label.to_owned(),
            FontId::proportional(size),
            look.text,
            text_width,
        )
    } else {
        galley
    };
    match style {
        SashStyle::Notch => {
            let body = Rect::from_min_size(
                Pos2::new(rect.center().x - width * 0.5, rect.top()),
                Vec2::new(width, height),
            );
            let round = (height * 0.5) as u8;
            let corners = egui::CornerRadius {
                nw: 0,
                ne: 0,
                sw: round,
                se: round,
            };
            painter.rect_filled(body, corners, look.fill);
            painter.rect_stroke(body, corners, stroke, egui::StrokeKind::Inside);
            painter.galley(body.center() - galley.size() * 0.5, galley, look.text);
            body
        }
        SashStyle::Chip => {
            let inset = (rect.width() * 0.04).max(4.0);
            let body = Rect::from_min_size(
                Pos2::new(rect.center().x - width * 0.5, rect.top() + inset),
                Vec2::new(width, height),
            );
            let round = (height * 0.5) as u8;
            painter.rect_filled(body, round, look.fill);
            painter.rect_stroke(body, round, stroke, egui::StrokeKind::Inside);
            painter.galley(body.center() - galley.size() * 0.5, galley, look.text);
            body
        }
        _ => {
            let center = Pos2::new(rect.left() + height * 0.5, rect.center().y);
            let body = Rect::from_center_size(center, Vec2::new(height, width));
            let round = (height * 0.5) as u8;
            let corners = egui::CornerRadius {
                nw: 0,
                sw: 0,
                ne: round,
                se: round,
            };
            painter.rect_filled(body, corners, look.fill);
            painter.rect_stroke(body, corners, stroke, egui::StrokeKind::Inside);
            let pos = center - galley.size() * 0.5;
            painter.add(
                TextShape::new(pos, galley, look.text)
                    .with_angle_and_anchor(-std::f32::consts::FRAC_PI_2, Align2::CENTER_CENTER),
            );
            body
        }
    }
}
