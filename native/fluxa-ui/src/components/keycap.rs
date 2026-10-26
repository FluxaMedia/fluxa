use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::draw_text;

const CAP_GAP: f32 = 4.0;
const ALT_GAP: f32 = 10.0;
const DEPTH: f32 = 3.0;

#[derive(Clone, Copy, PartialEq)]
enum Pad {
    A,
    B,
    X,
    Y,
    DPad(Option<u8>),
    Stick,
}

fn pad(token: &str) -> Option<Pad> {
    match token {
        "A" => Some(Pad::A),
        "B" => Some(Pad::B),
        "X" => Some(Pad::X),
        "Y" => Some(Pad::Y),
        "D-pad" => Some(Pad::DPad(None)),
        "D-pad Up" => Some(Pad::DPad(Some(0))),
        "D-pad Down" => Some(Pad::DPad(Some(1))),
        "D-pad Left" => Some(Pad::DPad(Some(2))),
        "D-pad Right" => Some(Pad::DPad(Some(3))),
        "left stick" => Some(Pad::Stick),
        _ => None,
    }
}

fn alternatives(label: &str) -> Vec<Vec<&str>> {
    label
        .split(" / ")
        .map(|alt| alt.split(" + ").collect())
        .collect()
}

fn cap_width(painter: &Painter, key: &str, size: f32) -> f32 {
    if pad(key).is_some() {
        return size * 1.7;
    }
    let text = painter
        .layout_no_wrap(key.to_owned(), FontId::proportional(size), Color32::WHITE)
        .size()
        .x;
    (text + size * 1.1).max(size * 1.9)
}

pub(crate) fn keys_width(painter: &Painter, label: &str, size: f32) -> f32 {
    alternatives(label)
        .iter()
        .map(|alt| {
            alt.iter()
                .map(|key| cap_width(painter, key, size))
                .sum::<f32>()
                + CAP_GAP * alt.len().saturating_sub(1) as f32
        })
        .sum::<f32>()
        + ALT_GAP * 2.0 * label.matches(" / ").count() as f32
}

pub(crate) fn draw_keys(
    painter: &Painter,
    rect: Rect,
    label: &str,
    size: f32,
    lit: bool,
    text: Color32,
) {
    let cap_height = (size * 2.0).min(rect.height());
    let mut x = rect.right() - keys_width(painter, label, size);
    let alts = alternatives(label);
    let last = alts.len().saturating_sub(1);
    for (position, alt) in alts.iter().enumerate() {
        for (index, key) in alt.iter().enumerate() {
            let width = cap_width(painter, key, size);
            let cap = Rect::from_min_size(
                Pos2::new(x, rect.center().y - cap_height * 0.5 - DEPTH * 0.5),
                Vec2::new(width, cap_height),
            );
            match pad(key) {
                Some(kind) => draw_pad(painter, cap, kind),
                None => draw_cap(painter, cap, key, size, lit, text),
            }
            x += width;
            if index + 1 < alt.len() {
                x += CAP_GAP;
            }
        }
        if position < last {
            x += ALT_GAP;
            draw_text(
                painter,
                Pos2::new(x, rect.center().y),
                Align2::CENTER_CENTER,
                "/",
                FontId::proportional(size),
                text.gamma_multiply(0.45),
                None,
            );
            x += ALT_GAP;
        }
    }
}

fn draw_cap(painter: &Painter, cap: Rect, key: &str, size: f32, lit: bool, text: Color32) {
    let radius = 6.0;
    let base = Color32::from_rgb(10, 10, 13);
    let face = if lit {
        Color32::from_rgb(64, 64, 74)
    } else {
        Color32::from_rgb(46, 46, 54)
    };
    let side = cap.translate(Vec2::new(0.0, DEPTH));
    painter.rect_filled(side, radius, base);
    painter.rect_stroke(
        side,
        radius,
        Stroke::new(1.0, Color32::from_white_alpha(14)),
        StrokeKind::Inside,
    );
    painter.rect_filled(cap, radius, face);
    painter.rect_stroke(
        cap,
        radius,
        Stroke::new(1.0, Color32::from_white_alpha(38)),
        StrokeKind::Inside,
    );
    let inner = cap.shrink2(Vec2::new(5.0, 3.0));
    painter.rect_stroke(
        inner,
        radius - 2.0,
        Stroke::new(1.0, Color32::from_white_alpha(10)),
        StrokeKind::Inside,
    );
    draw_text(
        painter,
        cap.center(),
        Align2::CENTER_CENTER,
        key,
        FontId::proportional(size),
        text,
        None,
    );
}

fn draw_pad(painter: &Painter, cap: Rect, kind: Pad) {
    let center = cap.center() + Vec2::new(0.0, DEPTH * 0.5);
    let radius = cap.height() * 0.5;
    let body = Color32::from_rgb(34, 34, 40);
    match kind {
        Pad::A | Pad::B | Pad::X | Pad::Y => {
            let (letter, tint) = match kind {
                Pad::A => ("A", Color32::from_rgb(108, 214, 128)),
                Pad::B => ("B", Color32::from_rgb(240, 98, 98)),
                Pad::X => ("X", Color32::from_rgb(96, 160, 255)),
                _ => ("Y", Color32::from_rgb(244, 200, 80)),
            };
            painter.circle_filled(
                center + Vec2::new(0.0, 2.0),
                radius,
                Color32::from_rgb(10, 10, 13),
            );
            painter.circle_filled(center, radius, body);
            painter.circle_stroke(center, radius, Stroke::new(1.5, tint));
            draw_text(
                painter,
                center,
                Align2::CENTER_CENTER,
                letter,
                FontId::proportional(radius * 1.15),
                tint,
                None,
            );
        }
        Pad::DPad(lit) => {
            let arm = radius * 1.05;
            let thick = radius * 0.55;
            let dim = Color32::from_rgb(150, 150, 160);
            let hot = Color32::from_rgb(240, 240, 246);
            let tint = |direction: u8| match lit {
                Some(active) if active != direction => dim.gamma_multiply(0.55),
                Some(_) => hot,
                None => hot,
            };
            let half = arm * 0.5;
            let arms = [
                (0, Vec2::new(0.0, -half), Vec2::new(thick, arm)),
                (1, Vec2::new(0.0, half), Vec2::new(thick, arm)),
                (2, Vec2::new(-half, 0.0), Vec2::new(arm, thick)),
                (3, Vec2::new(half, 0.0), Vec2::new(arm, thick)),
            ];
            for (direction, offset, size) in arms {
                painter.rect_filled(
                    Rect::from_center_size(center + offset, size),
                    2.0,
                    tint(direction),
                );
            }
        }
        Pad::Stick => {
            painter.circle_filled(
                center + Vec2::new(0.0, 2.0),
                radius,
                Color32::from_rgb(10, 10, 13),
            );
            painter.circle_filled(center, radius, body);
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(1.0, Color32::from_white_alpha(60)),
            );
            painter.circle_filled(center, radius * 0.55, Color32::from_rgb(70, 70, 80));
            painter.circle_stroke(
                center,
                radius * 0.55,
                Stroke::new(1.0, Color32::from_white_alpha(90)),
            );
        }
    }
}
