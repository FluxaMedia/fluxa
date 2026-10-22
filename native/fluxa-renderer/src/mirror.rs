use std::sync::Arc;

use egui::epaint::{ClippedShape, CornerRadiusF32, Shape};
use egui::{Pos2, Rect, TextureId, pos2};

pub fn mirror_shapes(shapes: &mut [ClippedShape], width: f32) {
    let mut last_text: Option<(Rect, f32)> = None;
    for clipped in shapes {
        let caret = last_text.and_then(|(span, shift)| caret_x(&clipped.shape, span, shift));
        if let Shape::Text(text) = &clipped.shape {
            let extent = text.galley.rect.translate(text.pos.to_vec2());
            let moved = width - text.pos.x - text.galley.rect.max.x - text.galley.rect.min.x;
            last_text = Some((extent, moved - text.pos.x));
        }
        clipped.clip_rect = mirror_rect(clipped.clip_rect, width);
        mirror_shape(&mut clipped.shape, width);
        if let (Some(x), Shape::LineSegment { points, .. }) = (caret, &mut clipped.shape) {
            points.iter_mut().for_each(|point| point.x = x);
        }
    }
}

fn caret_x(shape: &Shape, span: Rect, shift: f32) -> Option<f32> {
    let Shape::LineSegment { points, .. } = shape else {
        return None;
    };
    let [from, to] = *points;
    let vertical = (from.x - to.x).abs() < 0.01;
    let inside = span.x_range().contains(from.x)
        && span.y_range().contains(from.y.min(to.y))
        && span.y_range().contains(from.y.max(to.y));
    (vertical && inside).then_some(from.x + shift)
}

fn mirror_x(x: f32, width: f32) -> f32 {
    width - x
}

fn mirror_pos(pos: Pos2, width: f32) -> Pos2 {
    pos2(mirror_x(pos.x, width), pos.y)
}

pub fn mirror_rect(rect: Rect, width: f32) -> Rect {
    Rect::from_min_max(
        pos2(mirror_x(rect.max.x, width), rect.min.y),
        pos2(mirror_x(rect.min.x, width), rect.max.y),
    )
}

fn mirror_shape(shape: &mut Shape, width: f32) {
    match shape {
        Shape::Noop | Shape::Band(_) => {}
        Shape::Vec(shapes) => shapes
            .iter_mut()
            .for_each(|shape| mirror_shape(shape, width)),
        Shape::Circle(circle) => circle.center = mirror_pos(circle.center, width),
        Shape::Ellipse(ellipse) => ellipse.center = mirror_pos(ellipse.center, width),
        Shape::LineSegment { points, .. } => points
            .iter_mut()
            .for_each(|point| *point = mirror_pos(*point, width)),
        Shape::Path(path) => path
            .points
            .iter_mut()
            .for_each(|point| *point = mirror_pos(*point, width)),
        Shape::Rect(rect) => {
            rect.rect = mirror_rect(rect.rect, width);
            let radius = CornerRadiusF32::from(rect.corner_radius);
            rect.corner_radius = CornerRadiusF32 {
                nw: radius.ne,
                ne: radius.nw,
                sw: radius.se,
                se: radius.sw,
            }
            .into();
        }
        Shape::Text(text) => {
            let extent = text.galley.rect;
            if text.galley.rows.len() > 1 && text.galley.job.halign == egui::Align::Min {
                let galley = Arc::make_mut(&mut text.galley);
                for placed in &mut galley.rows {
                    placed.pos.x = extent.max.x - placed.row.size.x;
                }
            }
            text.pos.x = width - text.pos.x - extent.max.x - extent.min.x;
        }
        Shape::Mesh(mesh) if mesh.texture_id == TextureId::Managed(0) => Arc::make_mut(mesh)
            .vertices
            .iter_mut()
            .for_each(|vertex| vertex.pos.x = mirror_x(vertex.pos.x, width)),
        Shape::Mesh(mesh) => {
            let Some((min, max)) = mesh.vertices.iter().fold(None, |bounds, vertex| {
                let x = vertex.pos.x;
                Some(bounds.map_or((x, x), |(min, max): (f32, f32)| (min.min(x), max.max(x))))
            }) else {
                return;
            };
            let shift = width - max - min;
            Arc::make_mut(mesh)
                .vertices
                .iter_mut()
                .for_each(|vertex| vertex.pos.x += shift);
        }
        Shape::QuadraticBezier(curve) => curve
            .points
            .iter_mut()
            .for_each(|point| *point = mirror_pos(*point, width)),
        Shape::CubicBezier(curve) => curve
            .points
            .iter_mut()
            .for_each(|point| *point = mirror_pos(*point, width)),
        Shape::Callback(callback) => callback.rect = mirror_rect(callback.rect, width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::epaint::{CornerRadius, RectShape, Vertex};
    use egui::{Color32, Mesh, TextureId};

    fn clipped(shape: Shape) -> ClippedShape {
        ClippedShape {
            clip_rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0)),
            shape,
            transform_after_tessellation: egui::emath::TSTransform::IDENTITY,
        }
    }

    #[test]
    fn rect_moves_to_the_opposite_edge_and_swaps_corners() {
        let mut rect = RectShape::filled(
            Rect::from_min_max(pos2(10.0, 5.0), pos2(30.0, 25.0)),
            CornerRadius {
                nw: 8,
                ne: 0,
                sw: 4,
                se: 0,
            },
            Color32::WHITE,
        );
        rect.corner_radius.ne = 0;
        let mut shapes = vec![clipped(Shape::Rect(rect))];
        mirror_shapes(&mut shapes, 100.0);
        let Shape::Rect(mirrored) = &shapes[0].shape else {
            panic!("expected rect")
        };
        assert_eq!(mirrored.rect.min.x, 70.0);
        assert_eq!(mirrored.rect.max.x, 90.0);
        assert_eq!(mirrored.corner_radius.ne, 8);
        assert_eq!(mirrored.corner_radius.se, 4);
        assert_eq!(mirrored.corner_radius.nw, 0);
    }

    #[test]
    fn wrapped_rows_end_on_the_same_edge() {
        let context = egui::Context::default();
        context.set_fonts(egui::FontDefinitions::default());
        context
            .run_ui(egui::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
        let galley = context.fonts_mut(|fonts| {
            fonts.layout(
                "aaaa aaaa a".to_owned(),
                egui::FontId::proportional(14.0),
                Color32::WHITE,
                40.0,
            )
        });
        assert!(galley.rows.len() > 1);
        let mut shapes = vec![clipped(Shape::Text(egui::epaint::TextShape::new(
            pos2(0.0, 0.0),
            galley,
            Color32::WHITE,
        )))];
        mirror_shapes(&mut shapes, 100.0);
        let Shape::Text(text) = &shapes[0].shape else {
            panic!("expected text")
        };
        let right = |row: &egui::epaint::text::PlacedRow| row.pos.x + row.row.size.x;
        let first = right(&text.galley.rows[0]);
        assert!(
            text.galley
                .rows
                .iter()
                .all(|row| (right(row) - first).abs() < 0.01)
        );
    }

    #[test]
    fn caret_stays_on_its_glyph_boundary_after_mirroring() {
        let context = egui::Context::default();
        context.set_fonts(egui::FontDefinitions::default());
        context
            .run_ui(egui::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
        let galley = context.fonts_mut(|fonts| {
            fonts.layout_no_wrap(
                "abcd".to_owned(),
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            )
        });
        let height = galley.rect.height();
        let caret_at = 10.0 + galley.rect.width() * 0.25;
        let mut shapes = vec![
            clipped(Shape::Text(egui::epaint::TextShape::new(
                pos2(10.0, 0.0),
                galley,
                Color32::WHITE,
            ))),
            clipped(Shape::LineSegment {
                points: [pos2(caret_at, 0.0), pos2(caret_at, height)],
                stroke: egui::Stroke::new(1.0, Color32::WHITE).into(),
            }),
        ];
        mirror_shapes(&mut shapes, 200.0);
        let (Shape::Text(text), Shape::LineSegment { points, .. }) =
            (&shapes[0].shape, &shapes[1].shape)
        else {
            panic!("unexpected shapes")
        };
        let expected = text.pos.x + text.galley.rect.width() * 0.25;
        assert!((points[0].x - expected).abs() < 0.01);
    }

    #[test]
    fn image_mesh_is_repositioned_without_flipping_its_uvs() {
        let mut mesh = Mesh::with_texture(TextureId::Managed(1));
        let vertex = |x: f32, u: f32| Vertex {
            pos: pos2(x, 0.0),
            uv: pos2(u, 0.0),
            color: Color32::WHITE,
        };
        mesh.vertices = vec![vertex(10.0, 0.0), vertex(30.0, 1.0)];
        let mut shapes = vec![clipped(Shape::Mesh(Arc::new(mesh)))];
        mirror_shapes(&mut shapes, 100.0);
        let Shape::Mesh(mirrored) = &shapes[0].shape else {
            panic!("expected mesh")
        };
        assert_eq!(mirrored.vertices[0].pos.x, 70.0);
        assert_eq!(mirrored.vertices[1].pos.x, 90.0);
        assert_eq!(mirrored.vertices[0].uv.x, 0.0);
    }

    #[test]
    fn gradient_mesh_is_flipped_so_its_dark_side_follows_the_content() {
        let mut mesh = Mesh::default();
        let vertex = |x: f32, color: Color32| Vertex {
            pos: pos2(x, 0.0),
            uv: pos2(0.0, 0.0),
            color,
        };
        mesh.vertices = vec![
            vertex(0.0, Color32::BLACK),
            vertex(40.0, Color32::TRANSPARENT),
        ];
        let mut shapes = vec![clipped(Shape::Mesh(Arc::new(mesh)))];
        mirror_shapes(&mut shapes, 100.0);
        let Shape::Mesh(mirrored) = &shapes[0].shape else {
            panic!("expected mesh")
        };
        assert_eq!(mirrored.vertices[0].pos.x, 100.0);
        assert_eq!(mirrored.vertices[0].color, Color32::BLACK);
        assert_eq!(mirrored.vertices[1].pos.x, 60.0);
    }

    #[test]
    fn clip_rect_is_mirrored() {
        let mut shapes = vec![ClippedShape {
            clip_rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(30.0, 10.0)),
            shape: Shape::Noop,
            transform_after_tessellation: egui::emath::TSTransform::IDENTITY,
        }];
        mirror_shapes(&mut shapes, 100.0);
        assert_eq!(shapes[0].clip_rect.min.x, 70.0);
        assert_eq!(shapes[0].clip_rect.max.x, 100.0);
    }
}
