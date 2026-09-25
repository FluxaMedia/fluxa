struct Params {
    src_width: u32,
    src_height: u32,
    pitch: u32,
    dst_width: u32,
    dst_height: u32,
    tile_x: u32,
    tile_y: u32,
    has_alpha: u32,
};

@group(0) @binding(0)
var<storage, read> nv12_bytes: array<u32>;

@group(0) @binding(1)
var atlas: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> params: Params;

@group(0) @binding(3)
var<storage, read> alpha_bytes: array<u32>;

fn load_byte(byte_offset: u32) -> u32 {
    let word = nv12_bytes[byte_offset / 4u];
    let shift = (byte_offset % 4u) * 8u;
    return (word >> shift) & 255u;
}

fn load_alpha(byte_offset: u32) -> u32 {
    let word = alpha_bytes[byte_offset / 4u];
    let shift = (byte_offset % 4u) * 8u;
    return (word >> shift) & 255u;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (params.src_width == 0u || params.src_height == 0u || params.dst_width == 0u || params.dst_height == 0u) {
        return;
    }
    if (gid.x >= params.dst_width || gid.y >= params.dst_height) {
        return;
    }

    let src_x = min(gid.x * params.src_width / params.dst_width, params.src_width - 1u);
    let src_y = min(gid.y * params.src_height / params.dst_height, params.src_height - 1u);

    let y_offset = src_y * params.pitch + src_x;
    let uv_base = params.pitch * params.src_height;
    let uv_offset = uv_base + (src_y / 2u) * params.pitch + (src_x / 2u) * 2u;

    let y_limited = f32(load_byte(y_offset)) - 16.0;
    let cb = f32(load_byte(uv_offset)) - 128.0;
    let cr = f32(load_byte(uv_offset + 1u)) - 128.0;

    let luma = 1.164383 * y_limited;
    let red = (luma + 1.596027 * cr) / 255.0;
    let green = (luma - 0.391762 * cb - 0.812968 * cr) / 255.0;
    let blue = (luma + 2.017232 * cb) / 255.0;

    let atlas_size = textureDimensions(atlas);
    let dst_x = params.tile_x + gid.x;
    let dst_y = params.tile_y + gid.y;
    if (dst_x >= atlas_size.x || dst_y >= atlas_size.y) {
        return;
    }
    let dst = vec2<i32>(i32(dst_x), i32(dst_y));
    let alpha = select(
        255.0,
        f32(load_alpha(src_y * params.src_width + src_x)),
        params.has_alpha != 0u,
    ) / 255.0;
    textureStore(
        atlas,
        dst,
        vec4<f32>(red * alpha, green * alpha, blue * alpha, alpha),
    );
}
