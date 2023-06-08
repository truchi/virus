struct Sizes {
    // Surface `(width, height)` size.
    surface: vec2<u32>,

    // Texture `(width, height)` size.
    texture: vec2<u32>,
}

struct Vertex {
    // Vertex type:
    // - 0: a background rectangle (use `color`),
    // - 1: a mask glyph (use `uv` in the mask texture with `color`),
    // - 2: a color glyph (use `uv` in the color texture),
    @location(0) ty: u32,

    // Region world `(top, left)` coordinates.
    @location(1) region_position: vec2<i32>,

    // Region `(width, height)` size.
    @location(2) region_size: vec2<u32>,

    // Vertex `(top, left)` coordinates in region.
    @location(3) position: vec2<i32>,

    // Depth (far to near).
    @location(4) depth: u32,

    // Texture `(x, y)` coordinates.
    @location(5) uv: vec2<u32>,

    // sRGBA color.
    @location(6) color: vec4<u32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) ty: u32,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) min: vec2<f32>,
    @location(4) max: vec2<f32>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var<uniform> sizes: Sizes;

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    let surface = vec2<f32>(sizes.surface);
    let texture = vec2<f32>(sizes.texture);
    let region_position = vec2<f32>(vertex.region_position.yx);
    let region_size = vec2<f32>(vertex.region_size);
    let position = vec2<f32>(vertex.position.yx);
    let depth = f32(vertex.depth);
    let uv = vec2<f32>(vertex.uv);
    let color = vec4<f32>(vertex.color);

    var fragment: Fragment;
    fragment.ty = vertex.ty;
    fragment.position = vec4<f32>(
        (position.x + region_position.x) / surface.x - 1.0,
        1.0 - (position.y + region_position.y) /  surface.y,
        depth,
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
fn fragment(fragment: Fragment) -> @location(0) vec4<f32> {
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
            return vec4<f32>(
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
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
    }
}
