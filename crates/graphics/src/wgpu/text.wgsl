struct Sizes {
    surface: vec2<u32>,
    texture: vec2<u32>,
}

struct Vertex {
    // Vertex type:
    // - 0: a background rectangle (use `color`),
    // - 1: a mask glyph (use `texture` in the mask texture with `color`),
    // - 2: a color glyph (use `texture` in the color texture),
    @location(0) ty: u32,

    // World `[top, left]` coordinates.
    @location(1) position: vec2<i32>,

    // Depth (far to near).
    @location(2) depth: u32,

    // Texture `[x, y]` coordinates.
    @location(3) uv: vec2<u32>,

    /// sRGBA color.
    @location(4) color: vec4<u32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) ty: u32,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var<uniform> sizes: Sizes;

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    var fragment: Fragment;
    fragment.ty = vertex.ty;
    fragment.position = vec4<f32>(
              f32(vertex.position.y) / f32(sizes.surface.x) - 1.0,
        1.0 - f32(vertex.position.x) / f32(sizes.surface.y),
              f32(vertex.depth),
        1.0,
    );
    fragment.uv = vec2<f32>(
        f32(vertex.uv.x) / f32(sizes.texture.x),
        f32(vertex.uv.y) / f32(sizes.texture.y),
    );
    fragment.color = vec4<f32>(
        f32(vertex.color.r) / 255.0,
        f32(vertex.color.g) / 255.0,
        f32(vertex.color.b) / 255.0,
        f32(vertex.color.a) / 255.0,
    );

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
    switch fragment.ty {
        // Background rectangle
        case 0u {
            return fragment.color;
        }
        // Mask glyph
        case 1u {
            var mask = textureSampleLevel(mask_texture, texture_sampler, fragment.uv, 0.0).r;
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
