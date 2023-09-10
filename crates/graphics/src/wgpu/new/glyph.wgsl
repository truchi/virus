@group(0) @binding(0) var MASK: texture_2d<f32>;
@group(0) @binding(1) var COLOR: texture_2d<f32>;
@group(0) @binding(2) var SAMPLER: sampler;

var<push_constant> CONSTANTS: Constants;

struct Constants {
    // Surface `(width, height)` size.
    surface: vec2f,
}

fn to_clip(position: vec2f) -> vec4f {
    return vec4f(
        0.0 + 2.0 * position.x / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * position.y / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
}

fn to_texture(ty: u32, uv: vec2f) -> vec2f {
    switch ty {
        // Mask glyph
        case 0u: { return uv / vec2f(textureDimensions(MASK)); }
        // Color glyph
        case 1u: { return uv / vec2f(textureDimensions(COLOR)); }
        // Unreachable
        default: { return vec2f(0.0, 0.0); }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Instance {
    @builtin(vertex_index) index: u32,
    // Glyph type (0: mask, 1: color).
    @location(0) ty: u32,
    // Glyph `(top, left)` position.
    @location(1) position: vec2i,
    // Glyph `(width, height)` size.
    @location(2) size: vec2u,
    // Texture `(x, y)` position.
    @location(3) uv: vec2i,
    // Glyph sRGBA color.
    @location(4) color: vec4u,
}

fn position(index: u32, position: vec2i, size: vec2u) -> vec2f {
    let width = f32(size.x);
    let height = f32(size.y);
    let top = f32(position.x);
    let left = f32(position.y);
    let bottom = top + height;
    let right = left + width;

    switch index {
        // Top left
        case 0u: { return vec2f(left, top); }
        // Bottom right
        case 1u: { return vec2f(right, bottom); }
        // Top right
        case 2u: { return vec2f(right, top); }
        // Top left
        case 3u: { return vec2f(left, top); }
        // Bottom left
        case 4u: { return vec2f(left, bottom); }
        // Bottom right
        case 5u: { return vec2f(right, bottom); }
        // Unreachable
        default: { return vec2f(0.0, 0.0); }
    }
}

fn color(color: vec4u) -> vec4f {
    return pow(vec4f(color) / 255.0, vec4f(2.2, 2.2, 2.2, 1.0));
}

@vertex
fn vertex(instance: Instance) -> Fragment {
    var fragment: Fragment;
    fragment.position = to_clip(position(instance.index, instance.position, instance.size));
    fragment.ty = instance.ty;
    fragment.uv = to_texture(instance.ty, position(instance.index, instance.uv, instance.size));
    fragment.color = color(instance.color);

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) ty: u32,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
}

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    switch fragment.ty {
        // Mask glyph
        case 0u: {
            let mask = textureSampleLevel(MASK, SAMPLER, fragment.uv, 0.0).r;
            return vec4f(fragment.color.rgb, fragment.color.a * mask);
        }
        // Color glyph
        case 1u: {
            return textureSampleLevel(COLOR, SAMPLER, fragment.uv, 0.0);
        }
        default: {
            discard;
        }
    }
}
