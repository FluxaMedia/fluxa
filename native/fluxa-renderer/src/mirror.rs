use std::sync::Arc;

use egui::epaint::{ClippedShape, CornerRadiusF32, Shape};
use egui::{Pos2, Rect, TextureId, pos2};

pub fn mirror_shapes(shapes: &mut [ClippedShape], width: f32) {
    for clipped in shapes {
        clipped.clip_rect = mirror_rect(clipped.clip_rect, width);
        mirror_shape(&mut clipped.shape, width);
    }
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
        Shape::Noop => {}
        Shape::Vec(shapes) => shapes.iter_mut().for_each(|shape| mirror_shape(shape, width)),
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
        mesh.vertices = vec![vertex(0.0, Color32::BLACK), vertex(40.0, Color32::TRANSPARENT)];
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
        }];
        mirror_shapes(&mut shapes, 100.0);
        assert_eq!(shapes[0].clip_rect.min.x, 70.0);
        assert_eq!(shapes[0].clip_rect.max.x, 100.0);
    }
}
