use super::*;

pub(super) struct Ancestor {
    pub(super) classes: Vec<std::sync::Arc<str>>,
    pub(super) key: String,
    pub(super) state_id: Id,
    pub(super) inherited: crate::stylesheet::Style,
}

pub(super) fn class_name(class: &str) -> std::sync::Arc<str> {
    thread_local! {
        static NAMES: RefCell<HashMap<String, std::sync::Arc<str>, crate::fast::Build>> =
            RefCell::new(HashMap::default());
    }
    NAMES.with(|names| {
        let mut names = names.borrow_mut();
        if let Some(found) = names.get(class) {
            return found.clone();
        }
        let name: std::sync::Arc<str> = std::sync::Arc::from(class);
        names.insert(class.to_owned(), name.clone());
        name
    })
}

pub(super) fn index_positions(nodes: &[Resolved], into: &mut Positions) {
    for (index, node) in nodes.iter().enumerate() {
        into.insert(node.key.clone(), (index, nodes.len()));
        index_positions(&node.children, into);
    }
}

pub(super) type Positions = HashMap<String, (usize, usize), crate::fast::Build>;

thread_local! {
    static POSITIONS: RefCell<Vec<(Arc<Vec<Resolved>>, Arc<Positions>)>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn positions_of(tree: &Arc<Vec<Resolved>>) -> Arc<Positions> {
    POSITIONS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((_, found)) = cache.iter().find(|(known, _)| Arc::ptr_eq(known, tree)) {
            return found.clone();
        }
        let mut positions = Positions::default();
        index_positions(tree, &mut positions);
        let positions = Arc::new(positions);
        if cache.len() >= PREFETCH_PLANS {
            cache.remove(0);
        }
        cache.push((tree.clone(), positions.clone()));
        positions
    })
}

pub(super) type ClassParts = (&'static str, &'static [&'static str]);

pub(super) fn class_parts(node: &Resolved, position: Option<(usize, usize)>) -> Option<ClassParts> {
    struct Entry {
        class: Box<str>,
        kind: Box<str>,
        position: Option<(usize, usize)>,
        parts: Option<ClassParts>,
    }
    #[derive(Default)]
    struct Cache {
        generation: u64,
        entries: HashMap<u64, Vec<Entry>, crate::fast::Build>,
    }
    thread_local! {
        static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
    }
    let class = node.class.as_deref()?;
    let generation = crate::stylesheet::generation();
    let key = crate::fast::hash((class, &node.kind, position));
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.generation != generation {
            cache.entries.clear();
            cache.generation = generation;
        }
        let bucket = cache.entries.entry(key).or_default();
        if let Some(found) = bucket.iter().find(|entry| {
            *entry.class == *class && *entry.kind == *node.kind && entry.position == position
        }) {
            return found.parts;
        }
        crate::stats::count(&crate::stats::CLASS_MISSES);
        let parts = compute_class_parts(class, &node.kind, position);
        bucket.push(Entry {
            class: class.into(),
            kind: node.kind.as_str().into(),
            position,
            parts,
        });
        parts
    })
}

pub(super) fn compute_class_parts(
    class: &str,
    kind: &str,
    position: Option<(usize, usize)>,
) -> Option<ClassParts> {
    let classes: Vec<&str> = class.split_whitespace().collect();
    let index = classes
        .iter()
        .position(|class| crate::stylesheet::has_rule(class))?;
    let mut modifiers: Vec<String> = classes
        .iter()
        .enumerate()
        .filter(|(at, _)| *at != index)
        .map(|(_, class)| (*class).to_owned())
        .collect();
    let default = format!("<{kind}>");
    if crate::stylesheet::has_rule(&default) {
        modifiers.push(default);
    }
    if let Some((at, count)) = position {
        if at == 0 {
            modifiers.push("first-child".to_owned());
        }
        if at + 1 == count {
            modifiers.push("last-child".to_owned());
        }
        if count == 1 {
            modifiers.push("only-child".to_owned());
        }
        modifiers.push(format!("nth-child({})", at + 1));
        modifiers.push(format!(
            "nth-child({})",
            if at % 2 == 0 { "odd" } else { "even" }
        ));
    }
    let leaked: Vec<&'static str> = modifiers
        .into_iter()
        .map(|modifier| &*Box::leak(modifier.into_boxed_str()))
        .collect();
    Some((
        Box::leak(classes[index].to_owned().into_boxed_str()),
        Box::leak(leaked.into_boxed_slice()),
    ))
}

/// The ancestor chain as style scopes without interaction state, for layout
/// styles: descendant selectors (`.rows li`) then size and space their nodes too.
pub(super) fn plain_frames(chain: &[std::rc::Rc<Ancestor>]) -> Vec<crate::stylesheet::Frame> {
    chain
        .iter()
        .flat_map(|ancestor| {
            ancestor.classes.iter().enumerate().map(|(at, class)| {
                let frame = crate::stylesheet::Frame::plain(std::sync::Arc::clone(class), 0);
                if at == 0 {
                    frame.inheriting(&ancestor.inherited)
                } else {
                    frame
                }
            })
        })
        .collect()
}

pub(super) fn ancestor_states(
    context: &crate::gfx::Context,
    chain: &[std::rc::Rc<Ancestor>],
) -> Vec<crate::stylesheet::Frame> {
    let pointer = hover_pos(context);
    let mut found = Vec::new();
    for ancestor in chain {
        let recorded = context.data(|data| data.get_temp::<(Rect, u8)>(ancestor.state_id));
        let mut states = recorded.map_or(0, |(_, flags)| flags);
        if let Some((rect, _)) = recorded
            && pointer.is_some_and(|pos| rect.contains(pos))
        {
            states |= crate::stylesheet::HOVERED;
        }
        for (at, class) in ancestor.classes.iter().enumerate() {
            let frame = crate::stylesheet::Frame::plain(std::sync::Arc::clone(class), states);
            found.push(if at == 0 {
                frame.inheriting(&ancestor.inherited)
            } else {
                frame
            });
        }
    }
    found
}

pub(super) fn paint_box(
    painter: &crate::gfx::Painter,
    rect: Rect,
    radius: f32,
    style: &crate::stylesheet::Style,
) {
    if let Some(glow) = style.shadow
        && glow.color.a() > 0
    {
        painter.add(
            crate::gfx::Shadow {
                offset: [glow.x.round() as i8, glow.y.round() as i8],
                blur: glow.blur.clamp(0.0, 255.0) as u8,
                spread: glow.spread.clamp(0.0, 255.0) as u8,
                color: glow.color.gamma_multiply(style.opacity),
            }
            .as_shape(rect, radius),
        );
    }
    if style.backdrop > 0.0 {
        painter.add(crate::gfx::Shape::Callback(crate::gfx::PaintCallback {
            rect,
            callback: std::sync::Arc::new(crate::Glass {
                radius: radius.min(rect.height() * 0.5).min(rect.width() * 0.5),
                tint: Color32::TRANSPARENT,
                refraction: 0.0,
                bevel: 1.0,
                rim: 0.0,
                frost: (style.backdrop / 16.0).clamp(0.0, 1.0),
            }),
        }));
    }
    match style.gradient {
        Some(gradient) => gradient_fill(painter, rect, radius, gradient, style.opacity),
        None => {
            painter.rect_filled(rect, radius, style.fill.gamma_multiply(style.opacity));
        }
    }
}

pub(super) fn gradient_fill(
    painter: &crate::gfx::Painter,
    rect: Rect,
    radius: f32,
    gradient: crate::stylesheet::Gradient,
    opacity: f32,
) {
    let radius = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let mut outline = Vec::new();
    let corners = [
        (
            Pos2::new(rect.right() - radius, rect.top() + radius),
            -std::f32::consts::FRAC_PI_2,
        ),
        (
            Pos2::new(rect.right() - radius, rect.bottom() - radius),
            0.0,
        ),
        (
            Pos2::new(rect.left() + radius, rect.bottom() - radius),
            std::f32::consts::FRAC_PI_2,
        ),
        (
            Pos2::new(rect.left() + radius, rect.top() + radius),
            std::f32::consts::PI,
        ),
    ];
    let steps = if radius > 0.5 { 10 } else { 1 };
    for (center, start) in corners {
        for step in 0..=steps {
            let angle = start + std::f32::consts::FRAC_PI_2 * step as f32 / steps as f32;
            let normal = Vec2::new(angle.cos(), angle.sin());
            outline.push((center + normal * radius, normal));
        }
    }
    let angle = gradient.angle.to_radians();
    let direction = Vec2::new(angle.sin(), -angle.cos());
    let length = (rect.width() * angle.sin()).abs() + (rect.height() * angle.cos()).abs();
    let color_at = |point: Pos2| {
        let t = ((point - rect.center()).dot(direction) / length.max(0.001) + 0.5).clamp(0.0, 1.0);
        let stops = &gradient.stops[..gradient.len.max(1)];
        let mut color = stops[0].1;
        if t >= stops[stops.len() - 1].0 {
            color = stops[stops.len() - 1].1;
        } else {
            for pair in stops.windows(2) {
                if t >= pair[0].0 && t <= pair[1].0 {
                    let span = (pair[1].0 - pair[0].0).max(0.0001);
                    color = pair[0].1.lerp_to_gamma(pair[1].1, (t - pair[0].0) / span);
                    break;
                }
            }
        }
        color.gamma_multiply(opacity)
    };
    let mut mesh = crate::gfx::epaint::Mesh::default();
    let vertex = |pos: Pos2, color: Color32| crate::gfx::epaint::Vertex {
        pos,
        uv: Pos2::ZERO,
        color,
    };
    mesh.vertices
        .push(vertex(rect.center(), color_at(rect.center())));
    let count = outline.len() as u32;
    for (point, normal) in &outline {
        let color = color_at(*point);
        mesh.vertices.push(vertex(*point - *normal * 0.5, color));
    }
    for (point, normal) in &outline {
        mesh.vertices
            .push(vertex(*point + *normal * 0.5, Color32::TRANSPARENT));
    }
    for index in 0..count {
        let next = (index + 1) % count;
        mesh.indices.extend([0, 1 + index, 1 + next]);
        let (inner, inner_next) = (1 + index, 1 + next);
        let (outer, outer_next) = (1 + count + index, 1 + count + next);
        mesh.indices
            .extend([inner, outer, outer_next, inner, outer_next, inner_next]);
    }
    painter.add(crate::gfx::Shape::mesh(mesh));
}
