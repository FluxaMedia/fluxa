use egui::epaint::{Mesh, Vertex};
use egui::{Color32, Painter, Pos2, TextureId, Vec2};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Placement {
    pub pivot: Pos2,
    pub angle: f32,
    pub offset: Vec2,
}

impl Placement {
    pub fn tilted(pivot: Pos2, angle: f32, offset: Vec2) -> Self {
        Self {
            pivot,
            angle,
            offset,
        }
    }

    pub fn apply(&self, point: Pos2) -> Pos2 {
        if self.angle == 0.0 {
            return point + self.offset;
        }
        let (sin, cos) = self.angle.sin_cos();
        let d = point - self.pivot;
        self.pivot + Vec2::new(d.x * cos - d.y * sin, d.x * sin + d.y * cos) + self.offset
    }
}

pub(crate) struct Path {
    points: Vec<Pos2>,
}

impl Path {
    pub fn start(at: Pos2) -> Self {
        Self { points: vec![at] }
    }

    pub fn line_to(mut self, to: Pos2) -> Self {
        self.push(to);
        self
    }

    pub fn arc(mut self, center: Pos2, radius: f32, from: f32, to: f32) -> Self {
        if radius <= 0.0 {
            return self;
        }
        let steps = ((to - from).abs() * radius / 2.0).ceil().clamp(4.0, 48.0) as usize;
        for step in 0..=steps {
            let angle = from + (to - from) * step as f32 / steps as f32;
            self.push(center + Vec2::angled(angle) * radius);
        }
        self
    }

    pub fn close(self) -> Outline {
        let mut points = self.points;
        if points.len() > 1 && points[0].distance(points[points.len() - 1]) < 1e-3 {
            points.pop();
        }
        Outline { points }
    }

    fn push(&mut self, point: Pos2) {
        if self
            .points
            .last()
            .is_none_or(|last| last.distance(point) > 1e-3)
        {
            self.points.push(point);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Outline {
    points: Vec<Pos2>,
}

impl Outline {
    fn densified(&self, step: f32) -> Vec<Pos2> {
        let mut out = Vec::with_capacity(self.points.len() * 2);
        for (index, &from) in self.points.iter().enumerate() {
            let to = self.points[(index + 1) % self.points.len()];
            let pieces = (from.distance(to) / step).ceil().max(1.0) as usize;
            for piece in 0..pieces {
                out.push(from + (to - from) * (piece as f32 / pieces as f32));
            }
        }
        out
    }

    pub fn fill(
        &self,
        painter: &Painter,
        placement: Placement,
        step: f32,
        color: &dyn Fn(Pos2) -> Color32,
    ) {
        self.paint(painter, placement, step, None, &|local| {
            (Pos2::ZERO, color(local))
        });
    }

    pub fn fill_texture(
        &self,
        painter: &Painter,
        placement: Placement,
        step: f32,
        texture: TextureId,
        uv: &dyn Fn(Pos2) -> Pos2,
        tint: &dyn Fn(Pos2) -> Color32,
    ) {
        self.paint(painter, placement, step, Some(texture), &|local| {
            (uv(local), tint(local))
        });
    }

    pub fn stroke(&self, painter: &Painter, placement: Placement, stroke: egui::Stroke) {
        let points = self
            .points
            .iter()
            .map(|&point| placement.apply(point))
            .collect();
        painter.add(egui::Shape::closed_line(points, stroke));
    }

    fn paint(
        &self,
        painter: &Painter,
        placement: Placement,
        step: f32,
        texture: Option<TextureId>,
        sample: &dyn Fn(Pos2) -> (Pos2, Color32),
    ) {
        if self.points.len() < 3 {
            return;
        }
        let ring = self.densified(step);
        let triangles = triangulate(&ring);
        let mut mesh = match texture {
            Some(id) => Mesh::with_texture(id),
            None => Mesh::default(),
        };
        let emit = |local: Pos2, mesh: &mut Mesh| {
            let (uv, color) = sample(local);
            mesh.vertices.push(Vertex {
                pos: placement.apply(local),
                uv,
                color,
            });
            (mesh.vertices.len() - 1) as u32
        };
        for [a, b, c] in triangles {
            let mut stack = vec![[ring[a], ring[b], ring[c]]];
            while let Some([p, q, r]) = stack.pop() {
                let edges = [(p, q, r), (q, r, p), (r, p, q)];
                let (from, to, apex) = edges
                    .into_iter()
                    .max_by(|x, y| x.0.distance(x.1).total_cmp(&y.0.distance(y.1)))
                    .unwrap();
                if from.distance(to) > step * 1.5 {
                    let mid = from.lerp(to, 0.5);
                    stack.push([from, mid, apex]);
                    stack.push([mid, to, apex]);
                    continue;
                }
                let i = [emit(p, &mut mesh), emit(q, &mut mesh), emit(r, &mut mesh)];
                mesh.indices.extend(i);
            }
        }
        let feather = 1.0 / painter.ctx().pixels_per_point();
        let outward = if signed_area(&ring) > 0.0 { 1.0 } else { -1.0 };
        let count = ring.len();
        let base = mesh.vertices.len() as u32;
        for (index, &point) in ring.iter().enumerate() {
            let before = ring[(index + count - 1) % count];
            let after = ring[(index + 1) % count];
            let along = (after - before).normalized();
            let normal = Vec2::new(along.y, -along.x) * outward;
            let (uv, color) = sample(point);
            mesh.vertices.push(Vertex {
                pos: placement.apply(point),
                uv,
                color,
            });
            mesh.vertices.push(Vertex {
                pos: placement.apply(point + normal * feather),
                uv,
                color: Color32::TRANSPARENT,
            });
        }
        for index in 0..count as u32 {
            let next = (index + 1) % count as u32;
            let (a, b) = (base + index * 2, base + index * 2 + 1);
            let (c, d) = (base + next * 2, base + next * 2 + 1);
            mesh.indices.extend([a, b, d, a, d, c]);
        }
        painter.add(egui::Shape::mesh(mesh));
    }
}

fn signed_area(points: &[Pos2]) -> f32 {
    let mut sum = 0.0;
    for (index, point) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        sum += point.x * next.y - next.x * point.y;
    }
    sum * 0.5
}

fn cross(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
}

fn inside(point: Pos2, a: Pos2, b: Pos2, c: Pos2) -> bool {
    let sign = |p: Pos2, q: Pos2, r: Pos2| (p.x - r.x) * (q.y - r.y) - (q.x - r.x) * (p.y - r.y);
    let (d1, d2, d3) = (sign(point, a, b), sign(point, b, c), sign(point, c, a));
    let negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(negative && positive)
}

pub(crate) fn triangulate(points: &[Pos2]) -> Vec<[usize; 3]> {
    let mut order: Vec<usize> = (0..points.len()).collect();
    if signed_area(points) < 0.0 {
        order.reverse();
    }
    let mut triangles = Vec::with_capacity(points.len().saturating_sub(2));
    while order.len() > 3 {
        let count = order.len();
        let ear = (0..count).find(|&at| {
            let (a, b, c) = (
                order[(at + count - 1) % count],
                order[at],
                order[(at + 1) % count],
            );
            if cross(points[a], points[b], points[c]) <= 1e-4 {
                return false;
            }
            !order.iter().any(|&other| {
                other != a
                    && other != b
                    && other != c
                    && points[other] != points[a]
                    && points[other] != points[b]
                    && points[other] != points[c]
                    && inside(points[other], points[a], points[b], points[c])
            })
        });
        match ear {
            Some(at) => {
                let (a, b, c) = (
                    order[(at + count - 1) % count],
                    order[at],
                    order[(at + 1) % count],
                );
                triangles.push([a, b, c]);
                order.remove(at);
            }
            None => {
                order.remove(0);
            }
        }
    }
    if order.len() == 3 && cross(points[order[0]], points[order[1]], points[order[2]]).abs() > 1e-4
    {
        triangles.push([order[0], order[1], order[2]]);
    }
    triangles
}

pub(crate) fn over(top: Color32, bottom: Color32) -> Color32 {
    let keep = 255 - top.a() as u32;
    let mix = |t: u8, b: u8| (t as u32 + b as u32 * keep / 255).min(255) as u8;
    Color32::from_rgba_premultiplied(
        mix(top.r(), bottom.r()),
        mix(top.g(), bottom.g()),
        mix(top.b(), bottom.b()),
        mix(top.a(), bottom.a()),
    )
}

pub(crate) fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let lerp = |x: u8, y: u8| {
        (x as f32 + (y as f32 - x as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgba_premultiplied(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        lerp(a.a(), b.a()),
    )
}

pub(crate) fn sample_stops(stops: &[(f32, Color32)], t: f32) -> Color32 {
    let Some(&(first_at, first)) = stops.first() else {
        return Color32::TRANSPARENT;
    };
    if t <= first_at {
        return first;
    }
    for pair in stops.windows(2) {
        let ((from_at, from), (to_at, to)) = (pair[0], pair[1]);
        if t <= to_at {
            let span = (to_at - from_at).max(f32::EPSILON);
            return mix(from, to, (t - from_at) / span);
        }
    }
    stops[stops.len() - 1].1
}

pub(crate) fn linear(from: Pos2, to: Pos2, stops: Vec<(f32, Color32)>) -> impl Fn(Pos2) -> Color32 {
    move |point| {
        let axis = to - from;
        let t = ((point - from).dot(axis) / axis.length_sq().max(f32::EPSILON)).clamp(0.0, 1.0);
        sample_stops(&stops, t)
    }
}

pub(crate) fn layers<'a>(
    layers: Vec<Box<dyn Fn(Pos2) -> Color32 + 'a>>,
) -> impl Fn(Pos2) -> Color32 + 'a {
    move |point| {
        layers.iter().fold(Color32::TRANSPARENT, |below, layer| {
            over(layer(point), below)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Rect;

    fn tri_area(points: &[Pos2], triangles: &[[usize; 3]]) -> f32 {
        triangles
            .iter()
            .map(|[a, b, c]| (cross(points[*a], points[*b], points[*c]) * 0.5).abs())
            .sum()
    }

    #[test]
    fn concave_notch_triangulates_to_the_polygon_area() {
        use std::f32::consts::{FRAC_PI_2, PI};
        let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(300.0, 120.0));
        let tear = 70.0;
        let notch = 13.0;
        let outline = Path::start(Pos2::new(rect.left() + 20.0, rect.top()))
            .line_to(Pos2::new(rect.right() - 20.0, rect.top()))
            .arc(
                Pos2::new(rect.right() - 20.0, rect.top() + 20.0),
                20.0,
                -FRAC_PI_2,
                0.0,
            )
            .line_to(Pos2::new(rect.right(), tear - notch))
            .arc(Pos2::new(rect.right(), tear), notch, -FRAC_PI_2, -PI)
            .line_to(Pos2::new(rect.left() + notch, tear))
            .arc(Pos2::new(rect.left(), tear), notch, 0.0, -FRAC_PI_2)
            .line_to(Pos2::new(rect.left(), rect.top() + 20.0))
            .arc(
                Pos2::new(rect.left() + 20.0, rect.top() + 20.0),
                20.0,
                PI,
                PI + FRAC_PI_2,
            )
            .close();
        let ring = outline.densified(12.0);
        let triangles = triangulate(&ring);
        let expected = signed_area(&ring).abs();
        assert!((tri_area(&ring, &triangles) - expected).abs() < expected * 0.001);
        assert!(expected < rect.width() * (tear - rect.top()));
    }

    #[test]
    fn rotation_keeps_distances_and_turns_around_the_pivot() {
        let placement = Placement::tilted(
            Pos2::new(10.0, 10.0),
            std::f32::consts::FRAC_PI_2,
            Vec2::ZERO,
        );
        let moved = placement.apply(Pos2::new(20.0, 10.0));
        assert!((moved.x - 10.0).abs() < 1e-4 && (moved.y - 20.0).abs() < 1e-4);
    }
}
