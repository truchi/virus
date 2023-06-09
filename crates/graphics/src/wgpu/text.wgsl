struct Sizes {
    // Surface `(width, height)` size.
    surface: vec2u,

    // Texture `(width, height)` size.
    texture: vec2u,
}

struct Vertex {
    // Vertex type:
    // - 0: a background rectangle (use `color`),
    // - 1: a mask glyph (use `uv` in the mask texture with `color`),
    // - 2: a color glyph (use `uv` in the color texture),
    @location(0) ty: u32,

    // Region world `(top, left)` coordinates.
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

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) ty: u32,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
    @location(3) min: vec2f,
    @location(4) max: vec2f,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var<uniform> sizes: Sizes;

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    let surface = vec2f(sizes.surface);
    let texture = vec2f(sizes.texture);
    let region_position = vec2f(vertex.region_position.yx);
    let region_size = vec2f(vertex.region_size);
    let position = vec2f(vertex.position.yx);
    let uv = vec2f(vertex.uv);
    let color = vec4f(vertex.color);

    var fragment: Fragment;
    fragment.ty = vertex.ty;
    fragment.position = vec4f(
        (position.x + region_position.x) / surface.x - 1.0,
        1.0 - (position.y + region_position.y) /  surface.y,
        0.0,
        1.0,
    );
    fragment.uv = uv / texture;
    fragment.color = color / 255.0;
    fragment.min = region_position / 2.0;
    fragment.max = fragment.min + region_size / 2.0;

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(1) var mask_texture: texture_2d<f32>;
@group(0) @binding(2) var color_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    // Clip region
    let inside = fragment.min <= fragment.position.xy & fragment.position.xy < fragment.max;
    if !(inside.x && inside.y) {
        discard;
    }

    switch fragment.ty {
        // Background rectangle
        case 0u {
            return fragment.color;
        }
        // Mask glyph
        case 1u {
            let mask = textureSampleLevel(mask_texture, texture_sampler, fragment.uv, 0.0).r;
            return vec4f(
                fragment.color.r,
                fragment.color.g,
                fragment.color.b,
                fragment.color.a * mask,
            );
        }
        // Color glyph
        case 2u {
            return textureSampleLevel(color_texture, texture_sampler, fragment.uv, 0.0);
        }
        default {
            return vec4f(0.0, 0.0, 0.0, 0.0);
        }
    }
}
