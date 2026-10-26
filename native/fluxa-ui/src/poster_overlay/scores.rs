use super::*;
use crate::{Shade, shade, stroke, tone};

pub(super) fn bottom_vignette(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    fade: Fade,
    [r, g, b]: [u8; 3],
) {
    const STEPS: usize = 12;
    let solid = radius.min(rect.height() * 0.1);
    let top = rect.bottom() - rect.height() * fade.reach;
    let bottom = rect.bottom() - solid;
    let shade = |alpha: f32| Color32::from_rgba_unmultiplied(r, g, b, alpha as u8);
    let mut mesh = Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let y = top + (bottom - top) * t;
        let color = shade(fade.alpha * t.powf(1.5));
        mesh.colored_vertex(Pos2::new(rect.left(), y), color);
        mesh.colored_vertex(Pos2::new(rect.right(), y), color);
        if step > 0 {
            let base = (step as u32 - 1) * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
    }
    painter.add(Shape::mesh(mesh));
    let corner = solid as u8;
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(rect.left(), bottom), rect.max),
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: corner,
            se: corner,
        },
        shade(fade.alpha),
    );
}

pub(super) fn score_colors(score: f32) -> (Color32, Color32) {
    let (left, right) = match score {
        s if s < 5.0 => ([255, 80, 80], [160, 40, 40]),
        s if s < 7.0 => ([255, 210, 90], [200, 150, 40]),
        s if s < 8.5 => ([120, 255, 160], [40, 170, 90]),
        _ => ([190, 140, 255], [186, 85, 211]),
    };
    let soft = |[r, g, b]: [u8; 3]| {
        let mix = |c: u8| (c as f32 * 0.9 + 255.0 * 0.1) as u8;
        Color32::from_rgba_unmultiplied(mix(r), mix(g), mix(b), 220)
    };
    (soft(left), soft(right))
}

pub(super) fn score_bar(
    painter: &Painter,
    rect: Rect,
    rating: Option<f32>,
    caption: &str,
    scale: f32,
) {
    let width = rect.width();
    let bar_height = (rect.height() * 0.012 * scale).max(3.0);
    let side = width * 0.14;
    let bottom = rect.bottom() - rect.height() * 0.04;
    let track = Rect::from_min_max(
        Pos2::new(rect.left() + side, bottom - bar_height),
        Pos2::new(rect.right() - side, bottom),
    );
    if !caption.is_empty() {
        let size = (width * 0.08 * scale).clamp(8.0, 18.0);
        let galley = painter.layout_no_wrap(
            caption.to_owned(),
            FontId::proportional(size),
            Color32::from_gray(200),
        );
        let center = Pos2::new(rect.center().x, track.top() - size * 0.95);
        painter.galley(
            center - galley.size() * 0.5,
            galley,
            Color32::from_gray(200),
        );
    }
    let Some(score) = rating else {
        return;
    };
    let pill = bar_height * 0.5;
    painter.rect_filled(track, pill, ink(tone::strong()));
    let fill = Rect::from_min_size(
        track.min,
        Vec2::new(track.width() * score / 10.0, bar_height),
    );
    let (left, right) = score_colors(score);
    let fill_painter = painter.with_clip_rect(fill.intersect(painter.clip_rect()));
    let shape = Rect::from_min_size(fill.min, Vec2::new(fill.width() + pill, bar_height));
    fill_painter.rect_filled(shape, pill, left);
    let mut mesh = Mesh::default();
    let fade = |c: Color32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 0);
    mesh.colored_vertex(Pos2::new(fill.left() + pill, fill.top()), fade(right));
    mesh.colored_vertex(fill.right_top() + Vec2::X * pill, right);
    mesh.colored_vertex(fill.right_bottom() + Vec2::X * pill, right);
    mesh.colored_vertex(Pos2::new(fill.left() + pill, fill.bottom()), fade(right));
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    fill_painter.add(Shape::mesh(mesh));
    fill_painter.line_segment(
        [
            Pos2::new(fill.left() + pill, fill.top() + 1.0),
            Pos2::new(fill.right(), fill.top() + 1.0),
        ],
        Stroke::new(stroke::hairline(), ink(tone::strong())),
    );
}

pub(super) fn score_number(
    painter: &Painter,
    rect: Rect,
    rating: Option<f32>,
    caption: &str,
    scale: f32,
) -> f32 {
    let width = rect.width();
    let left = rect.left() + width * 0.07;
    let mut bottom = rect.bottom() - rect.height() * 0.04;
    if !caption.is_empty() {
        let size = (width * 0.07 * scale).clamp(8.0, 16.0);
        let galley = painter.layout_no_wrap(
            caption.to_owned(),
            FontId::proportional(size),
            Color32::from_gray(200),
        );
        bottom -= galley.size().y;
        painter.galley(Pos2::new(left, bottom), galley, Color32::from_gray(200));
    }
    if let Some(score) = rating {
        let size = (width * 0.2 * scale).clamp(16.0, 56.0);
        let galley = painter.layout_no_wrap(
            format!("{score:.1}"),
            FontId::proportional(size),
            score_colors(score).0,
        );
        bottom -= galley.size().y * 0.9;
        painter.galley(Pos2::new(left, bottom), galley, text_primary());
    }
    (rect.bottom() - bottom) / rect.height() + 0.02
}

pub(super) fn score_minimal(
    painter: &Painter,
    rect: Rect,
    mut meta: meta::Meta,
    scale: f32,
) -> f32 {
    if meta.prefix.is_empty() && meta.score.is_none() {
        return 0.0;
    }
    let size = meta::fit(
        painter,
        &mut meta,
        (rect.width() * 0.075 * scale).clamp(9.0, 18.0),
        rect.width() * 0.88,
    );
    let center_y = rect.bottom() - rect.height() * 0.05 - size * 0.5;
    meta::meta_row(
        painter,
        rect.left() + rect.width() * 0.06,
        center_y,
        size,
        &meta,
        Color32::from_gray(235),
    );
    (rect.bottom() - center_y + size) / rect.height()
}

pub(super) fn score_frosted(
    painter: &Painter,
    rect: Rect,
    rating: Option<f32>,
    caption: &str,
    scale: f32,
    radius: f32,
) -> f32 {
    let height = (rect.height() * 0.09 * scale).max(18.0);
    let bar = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - height), rect.max);
    let r = radius as u8;
    painter.rect_filled(
        bar,
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: r,
            se: r,
        },
        shade(Shade::Scrim),
    );
    painter.line_segment(
        [bar.left_top(), bar.right_top()],
        Stroke::new(stroke::hairline(), ink(tone::raised())),
    );
    let size = height * 0.45;
    let pad = rect.width() * 0.05;
    if let Some(score) = rating {
        meta::star(
            painter,
            Pos2::new(bar.left() + pad + size * 0.45, bar.center().y),
            size * 0.47,
            score_colors(score).0,
        );
        let galley = painter.layout_no_wrap(
            format!("{score:.1}"),
            FontId::proportional(size),
            text_primary(),
        );
        painter.galley(
            Pos2::new(
                bar.left() + pad + size * 1.25,
                bar.center().y - galley.size().y * 0.5,
            ),
            galley,
            text_primary(),
        );
    }
    if !caption.is_empty() {
        let galley = painter.layout_no_wrap(
            caption.to_owned(),
            FontId::proportional(size * 0.8),
            Color32::from_gray(190),
        );
        let pos = Pos2::new(
            bar.right() - pad - galley.size().x,
            bar.center().y - galley.size().y * 0.5,
        );
        painter.galley(pos, galley, Color32::from_gray(190));
    }
    height / rect.height()
}

pub(super) fn sash_colors(status: PosterStatus) -> (Color32, Color32) {
    let ([r, g, b], [br, bg, bb]) = match status {
        PosterStatus::NewSeason => ([30, 130, 120], [100, 220, 210]),
        PosterStatus::NewEpisode => ([50, 110, 190], [160, 220, 255]),
        PosterStatus::Upcoming => ([170, 100, 20], [255, 190, 90]),
        PosterStatus::NewRelease => ([160, 130, 40], [212, 175, 55]),
        PosterStatus::Premiere => ([120, 60, 170], [200, 150, 255]),
        PosterStatus::Finale => ([150, 50, 90], [240, 130, 180]),
        PosterStatus::Ended => ([70, 70, 75], [190, 190, 195]),
        PosterStatus::Cancelled => ([110, 40, 40], [210, 110, 110]),
        PosterStatus::MiniSeries => ([40, 100, 140], [130, 200, 240]),
    };
    (Color32::from_rgb(r, g, b), Color32::from_rgb(br, bg, bb))
}

pub(super) fn trending_colors() -> (Color32, Color32) {
    (
        Color32::from_rgb(150, 30, 40),
        Color32::from_rgb(235, 90, 90),
    )
}

pub(super) fn award_colors(won: bool) -> (Color32, Color32) {
    if won {
        (
            Color32::from_rgb(150, 115, 30),
            Color32::from_rgb(235, 200, 90),
        )
    } else {
        (
            Color32::from_rgb(70, 70, 75),
            Color32::from_rgb(190, 190, 195),
        )
    }
}

pub(super) fn sash(
    painter: &Painter,
    rect: Rect,
    (inner, border): (Color32, Color32),
    label: &str,
) {
    let width = rect.width();
    let length = width * 1.15;
    let height = width * 0.12;
    let center = Pos2::new(rect.right() - width * 0.162, rect.top() + width * 0.162);
    let along = Vec2::new(1.0, 1.0).normalized();
    let across = along.rot90();
    let band = |inset: f32| {
        let half = height * 0.5 - inset;
        vec![
            center - along * length * 0.5 - across * half,
            center + along * length * 0.5 - across * half,
            center + along * length * 0.5 + across * half,
            center - along * length * 0.5 + across * half,
        ]
    };
    let shadow: Vec<Pos2> = band(0.0)
        .into_iter()
        .map(|p| p + Vec2::splat(height * 0.08))
        .collect();
    painter.add(Shape::convex_polygon(
        shadow,
        shade(Shade::Faint),
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(band(0.0), border, Stroke::NONE));
    painter.add(Shape::convex_polygon(
        band((height / 18.0).max(1.0)),
        inner,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        band(height * 0.12),
        Color32::from_rgba_unmultiplied(8, 8, 8, 245),
        Stroke::NONE,
    ));

    let size = (height * 0.4).min(height * 0.85 / (label.chars().count() as f32).powf(0.35));
    let text = label.to_uppercase();
    let font = FontId::proportional(size);
    let angle = std::f32::consts::FRAC_PI_4;
    for (offset, color) in [
        (Vec2::splat(size * 0.08), shade(Shade::Scrim)),
        (Vec2::ZERO, Color32::from_gray(225)),
    ] {
        let galley = painter.layout_no_wrap(text.clone(), font.clone(), color);
        let pos = center + offset - galley.size() * 0.5;
        painter.add(
            TextShape::new(pos, galley, color).with_angle_and_anchor(angle, Align2::CENTER_CENTER),
        );
    }
}
