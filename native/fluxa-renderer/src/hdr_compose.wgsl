struct Params {
    video_rect: vec4<f32>,
    values: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> params: Params;
@group(0) @binding(1)
var ui: texture_2d<f32>;
@group(0) @binding(2)
var video: texture_2d<f32>;
@group(0) @binding(3)
var video_sampler: sampler;

const PQ_M1: f32 = 0.1593017578125;
const PQ_M2: f32 = 78.84375;
const PQ_C1: f32 = 0.8359375;
const PQ_C2: f32 = 18.8515625;
const PQ_C3: f32 = 18.6875;

fn pq_to_nits(encoded: vec3<f32>) -> vec3<f32> {
    let p = pow(max(encoded, vec3<f32>(0.0)), vec3<f32>(1.0 / PQ_M2));
    let numerator = max(p - vec3<f32>(PQ_C1), vec3<f32>(0.0));
    let denominator = vec3<f32>(PQ_C2) - PQ_C3 * p;
    return pow(numerator / denominator, vec3<f32>(1.0 / PQ_M1)) * 10000.0;
}

fn nits_to_pq(nits: vec3<f32>) -> vec3<f32> {
    let y = pow(clamp(nits / 10000.0, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(PQ_M1));
    return pow((vec3<f32>(PQ_C1) + PQ_C2 * y) / (vec3<f32>(1.0) + PQ_C3 * y), vec3<f32>(PQ_M2));
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

fn bt709_to_bt2020(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(0.6274, 0.3293, 0.0433), c),
        dot(vec3<f32>(0.0691, 0.9195, 0.0114), c),
        dot(vec3<f32>(0.0164, 0.0880, 0.8956), c),
    );
}

fn bt2020_to_bt709(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(vec3<f32>(1.6605, -0.5876, -0.0728), c),
        dot(vec3<f32>(-0.1246, 1.1329, -0.0083), c),
        dot(vec3<f32>(-0.0182, -0.1006, 1.1187), c),
    );
}

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let white = params.values.x;
    let scrgb = params.values.y > 0.5;
    let has_video = params.values.z > 0.5;
    let ui_encoded = params.values.w > 0.5;

    var pixel = textureLoad(ui, vec2<i32>(position.xy), 0);
    var ui_linear = pixel.rgb;
    if (ui_encoded) {
        ui_linear = srgb_to_linear(ui_linear);
    }
    let ui_nits = bt709_to_bt2020(ui_linear) * white;

    var behind = vec3<f32>(0.0);
    let inside = all(position.xy >= params.video_rect.xy)
        && all(position.xy < params.video_rect.xy + params.video_rect.zw);
    if (has_video && inside) {
        let uv = (position.xy - params.video_rect.xy) / params.video_rect.zw;
        behind = pq_to_nits(textureSampleLevel(video, video_sampler, uv, 0.0).rgb);
    }

    let peak = max(behind.r, max(behind.g, behind.b));
    let capped = behind * min(1.0, white / max(peak, 0.001));
    let backdrop = mix(behind, capped, pixel.a);
    let nits = ui_nits + backdrop * (1.0 - pixel.a);
    if (scrgb) {
        return vec4<f32>(bt2020_to_bt709(nits) / 80.0, 1.0);
    }
    return vec4<f32>(nits_to_pq(nits), 1.0);
}
