struct Hole {
    rect: vec4<f32>,
    radius: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> hole: Hole;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let half_size = hole.rect.zw * 0.5;
    let local = position.xy - hole.rect.xy - half_size;
    let corner = abs(local) - half_size + vec2<f32>(hole.radius.x);
    let distance = length(max(corner, vec2<f32>(0.0))) + min(max(corner.x, corner.y), 0.0) - hole.radius.x;
    let coverage = clamp(0.5 - distance, 0.0, 1.0);
    return vec4<f32>(0.0, 0.0, 0.0, coverage);
}
