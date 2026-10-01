struct FullOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Pane {
    rect: vec4<f32>,
    tint: vec4<f32>,
    params: vec4<f32>,
    screen: vec4<f32>,
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(1) @binding(0) var<uniform> pane: Pane;
@group(2) @binding(0) var clear_src: texture_2d<f32>;
@group(2) @binding(1) var clear_samp: sampler;

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> FullOut {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var out: FullOut;
    out.pos = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_down(in: FullOut) -> @location(0) vec4<f32> {
    let h = 1.0 / vec2<f32>(textureDimensions(src));
    var c = textureSample(src, samp, in.uv) * 4.0;
    c += textureSample(src, samp, in.uv - h);
    c += textureSample(src, samp, in.uv + h);
    c += textureSample(src, samp, in.uv + vec2<f32>(h.x, -h.y));
    c += textureSample(src, samp, in.uv - vec2<f32>(h.x, -h.y));
    return c / 8.0;
}

@fragment
fn fs_up(in: FullOut) -> @location(0) vec4<f32> {
    let h = 1.0 / vec2<f32>(textureDimensions(src));
    var c = textureSample(src, samp, in.uv + vec2<f32>(-2.0 * h.x, 0.0));
    c += textureSample(src, samp, in.uv + vec2<f32>(2.0 * h.x, 0.0));
    c += textureSample(src, samp, in.uv + vec2<f32>(0.0, -2.0 * h.y));
    c += textureSample(src, samp, in.uv + vec2<f32>(0.0, 2.0 * h.y));
    c += textureSample(src, samp, in.uv + vec2<f32>(-h.x, h.y)) * 2.0;
    c += textureSample(src, samp, in.uv + h) * 2.0;
    c += textureSample(src, samp, in.uv + vec2<f32>(h.x, -h.y)) * 2.0;
    c += textureSample(src, samp, in.uv - h) * 2.0;
    return c / 12.0;
}

@fragment
fn fs_blit(in: FullOut) -> @location(0) vec4<f32> {
    return textureSample(src, samp, in.uv);
}

@vertex
fn vs_pane(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32(i & 1u), f32((i >> 1u) & 1u));
    let p = mix(pane.rect.xy - 1.0, pane.rect.zw + 1.0, corner);
    return vec4<f32>(p / pane.screen.xy * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

fn rounded_box(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half + r;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_pane(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let center = (pane.rect.xy + pane.rect.zw) * 0.5;
    let half = (pane.rect.zw - pane.rect.xy) * 0.5;
    let r = min(pane.params.x, min(half.x, half.y));
    let p = frag.xy - center;
    let d = rounded_box(p, half, r);
    let coverage = clamp(0.5 - d, 0.0, 1.0);

    let e = 0.5;
    let grad = vec2<f32>(
        rounded_box(p + vec2<f32>(e, 0.0), half, r) - rounded_box(p - vec2<f32>(e, 0.0), half, r),
        rounded_box(p + vec2<f32>(0.0, e), half, r) - rounded_box(p - vec2<f32>(0.0, e), half, r),
    );
    let n = normalize(grad + vec2<f32>(1e-5));
    let bevel = max(min(pane.params.z, min(half.x, half.y)), 1.0);
    let edge = clamp(1.0 + d / bevel, 0.0, 1.0);
    let bend = edge * edge * edge * pane.params.y;

    let uv = (frag.xy - n * bend) / pane.screen.xy;
    let spread = n * bend * 0.18 / pane.screen.xy;
    let frost = mix(0.55, 0.12, edge);
    let sharp = vec3<f32>(
        textureSample(clear_src, clear_samp, uv - spread).r,
        textureSample(clear_src, clear_samp, uv).g,
        textureSample(clear_src, clear_samp, uv + spread).b,
    );
    let soft = textureSample(src, samp, uv);
    var rgb = mix(sharp, soft.rgb, frost);

    let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    rgb = max(mix(vec3<f32>(luma), rgb, 1.35), vec3<f32>(0.0));
    rgb *= 1.0 + 0.12 * (1.0 - edge) * (1.0 - n.y) * 0.5;

    let t = pane.tint;
    rgb = rgb * (1.0 - t.a) + t.rgb;
    var a = soft.a * (1.0 - t.a) + t.a;

    let rim_width = 1.4 * pane.screen.z;
    let band = smoothstep(-rim_width, 0.0, d);
    let facing = dot(n, normalize(vec2<f32>(-0.45, -1.0)));
    let rim = band * (0.75 * pow(max(facing, 0.0), 1.5) + 0.3 * pow(max(-facing, 0.0), 1.5) + 0.12);
    let glow = edge * edge * edge * (0.05 + 0.12 * max(facing, 0.0));
    let spec = pane.params.w * (rim + glow);
    rgb += vec3<f32>(spec);
    a = min(a + spec, 1.0);

    return vec4<f32>(rgb, a) * coverage;
}
