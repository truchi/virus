struct Constants {
    // Surface `(width, height)` size.
    surface: vec2f,

    // Texture `(width, height)` size.
    texture: vec2f,
}

var<push_constant> CONSTANTS: Constants;
@group(0) @binding(0) var MASK: texture_2d<f32>;
@group(0) @binding(1) var COLOR: texture_2d<f32>;
@group(0) @binding(2) var ANIMATED: texture_2d<f32>;
@group(0) @binding(3) var BLUR: texture_2d<f32>;
@group(0) @binding(4) var SAMPLER: sampler;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Rectangle                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct RectangleVertex {
    // Region `(top, left)` world coordinates.
    @location(0) region_position: vec2i,
    // Region `(width, height)` size.
    @location(1) region_size: vec2u,
    // Vertex `(top, left)` coordinates in region.
    @location(2) position: vec2i,
    // sRGBA color.
    @location(3) color: vec4u,
}

struct RectangleFragment {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) min: vec2f,
    @location(2) max: vec2f,
}

@vertex
fn rectangle_vertex(vertex: RectangleVertex) -> RectangleFragment {
    let region_position = vec2f(vertex.region_position.yx);
    let region_size = vec2f(vertex.region_size);
    let position = vec2f(vertex.position.yx);
    let color = vec4f(vertex.color) / 255.0;

    var fragment: RectangleFragment;
    fragment.position = vec4f(
        0.0 + 2.0 * (position.x + region_position.x) / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * (position.y + region_position.y) / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.color = pow(color, vec4f(2.2, 2.2, 2.2, 1.0));
    fragment.min = region_position;
    fragment.max = region_position + region_size;

    return fragment;
}

@fragment
fn rectangle_fragment(fragment: RectangleFragment) -> @location(0) vec4f {
    // Clip region
    let inside = fragment.min <= fragment.position.xy & fragment.position.xy < fragment.max;
    if !(inside.x && inside.y) {
        discard;
    }

    // Background rectangle
    return fragment.color;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Shadow                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct ShadowVertex {
    // Glyph type (0: mask, 1: color, 2: animated).
    @location(0) glyph_type: u32,
    // Region `(top, left)` world coordinates.
    @location(1) region_position: vec2i,
    // Region `(width, height)` size.
    @location(2) region_size: vec2u,
    // Vertex `(top, left)` coordinates in region.
    @location(3) position: vec2i,
    // Texture `(x, y)` coordinates.
    @location(4) uv: vec2u,
}

struct ShadowFragment {
    @builtin(position) position: vec4f,
    @location(0) glyph_type: u32,
    @location(1) uv: vec2f,
    @location(2) min: vec2f,
    @location(3) max: vec2f,
}

@vertex
fn shadow_vertex(vertex: ShadowVertex) -> ShadowFragment {
    let region_position = vec2f(vertex.region_position.yx);
    let region_size = vec2f(vertex.region_size);
    let position = vec2f(vertex.position.yx);
    let uv = vec2f(vertex.uv);

    var fragment: ShadowFragment;
    fragment.position = vec4f(
        0.0 + 2.0 * (position.x + region_position.x) / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * (position.y + region_position.y) / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.glyph_type = vertex.glyph_type;
    fragment.uv = uv / CONSTANTS.texture;
    fragment.min = region_position;
    fragment.max = region_position + region_size;

    return fragment;
}

@fragment
fn shadow_fragment(fragment: ShadowFragment) -> @location(0) vec4f {
    // Clip region
    let inside = fragment.min <= fragment.position.xy & fragment.position.xy < fragment.max;
    if !(inside.x && inside.y) {
        discard;
    }

    switch fragment.glyph_type {
        // Mask glyph
        case 0u {
            // TODO
            // let mask = textureSampleLevel(MASK, SAMPLER, fragment.uv, 0.0).r;
            return vec4f(0.0, 0.0, 0.0, 0.0);
        }
        // Color glyph
        case 1u {
            return textureSampleLevel(COLOR, SAMPLER, fragment.uv, 0.0);
        }
        // Animated glyph
        case 2u {
            return textureSampleLevel(ANIMATED, SAMPLER, fragment.uv, 0.0);
        }
        default {
            discard;
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Glyph                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct GlyphVertex {
    // Glyph type (0: mask, 1: color, 2: animated).
    @location(0) glyph_type: u32,
    // Region `(top, left)` world coordinates.
    @location(1) region_position: vec2i,
    // Region `(width, height)` size.
    @location(2) region_size: vec2u,
    // Vertex `(top, left)` coordinates in region.
    @location(3) position: vec2i,
    // Texture `(x, y)` coordinates.
    @location(4) uv: vec2u,
    // sRGBA color.
    @location(5) color: vec4u,
}

struct GlyphFragment {
    @builtin(position) position: vec4f,
    @location(0) glyph_type: u32,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
    @location(3) min: vec2f,
    @location(4) max: vec2f,
}

@vertex
fn glyph_vertex(vertex: GlyphVertex) -> GlyphFragment {
    let region_position = vec2f(vertex.region_position.yx);
    let region_size = vec2f(vertex.region_size);
    let position = vec2f(vertex.position.yx);
    let uv = vec2f(vertex.uv);
    let color = vec4f(vertex.color) / 255.0;

    var fragment: GlyphFragment;
    fragment.position = vec4f(
        0.0 + 2.0 * (position.x + region_position.x) / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * (position.y + region_position.y) / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.glyph_type = vertex.glyph_type;
    fragment.uv = uv / CONSTANTS.texture;
    fragment.color = pow(color, vec4f(2.2, 2.2, 2.2, 1.0));
    fragment.min = region_position;
    fragment.max = region_position + region_size;

    return fragment;
}

@fragment
fn glyph_fragment(fragment: GlyphFragment) -> @location(0) vec4f {
    // Clip region
    let inside = fragment.min <= fragment.position.xy & fragment.position.xy < fragment.max;
    if !(inside.x && inside.y) {
        discard;
    }

    switch fragment.glyph_type {
        // Mask glyph
        case 0u {
            let mask = textureSampleLevel(MASK, SAMPLER, fragment.uv, 0.0).r;
            return vec4f(fragment.color.rgb, fragment.color.a * mask);
        }
        // Color glyph
        case 1u {
            return textureSampleLevel(COLOR, SAMPLER, fragment.uv, 0.0);
        }
        // Animated glyph
        case 2u {
            return textureSampleLevel(ANIMATED, SAMPLER, fragment.uv, 0.0);
        }
        default {
            discard;
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Blur                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO
struct BlurVertex {
    @location(0) position: vec2i,
}

// TODO
struct BlurFragment {
    @builtin(position) position: vec4f,
}

@vertex
fn blur_vertex(vertex: BlurVertex) -> BlurFragment {
    var fragment: BlurFragment;

    return fragment;
}

@fragment
fn blur_fragment(fragment: BlurFragment) -> @location(0) vec4f {
    discard;
}
