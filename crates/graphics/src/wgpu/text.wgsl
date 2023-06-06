struct Sizes {
    surface: vec2<u32>,
    texture: vec2<u32>,
}

struct Vertex {
    @location(0) ty: u32,
    @location(1) position: vec3<i32>,
    @location(2) size: vec2<u32>,
    @location(3) texture: vec2<i32>,
    @location(4) glyph: vec2<u32>,
    @location(5) color: vec4<u32>,
    @location(6) blur_radius: u32,
    @location(7) blur_color: vec3<u32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) ty: u32,
    @location(1) size: vec2<f32>,
    @location(2) texture: vec2<f32>,
    @location(3) glyph: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) blur_radius: i32,
    @location(6) blur_color: vec3<f32>,
    @location(7) glyph_min: vec2<f32>,
    @location(8) glyph_max: vec2<f32>,
    @location(9) weight: f32,
    @location(10) pxw: f32,
    @location(11) pxh: f32,
}

@group(0) @binding(0) var<uniform> sizes: Sizes;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    var fragment: Fragment;
    fragment.ty = vertex.ty;
    fragment.position = vec4<f32>(
              f32(vertex.position.x) / f32(sizes.surface.x) - 1.0,
        1.0 - f32(vertex.position.y) / f32(sizes.surface.y),
              f32(vertex.position.z),
        1.0,
    );
    fragment.size = vec2<f32>(
        f32(vertex.size.x) / f32(sizes.texture.x),
        f32(vertex.size.y) / f32(sizes.texture.y),
    );
    fragment.texture = vec2<f32>(
        f32(vertex.texture.x) / f32(sizes.texture.x),
        f32(vertex.texture.y) / f32(sizes.texture.y),
    );
    fragment.glyph = vec2<f32>(
        f32(vertex.glyph.x) / f32(sizes.texture.x),
        f32(vertex.glyph.y) / f32(sizes.texture.y),
    );
    fragment.color = vec4<f32>(
        f32(vertex.color.r) / 255.0,
        f32(vertex.color.g) / 255.0,
        f32(vertex.color.b) / 255.0,
        f32(vertex.color.a) / 255.0,
    );
    fragment.blur_radius = i32(vertex.blur_radius);
    fragment.blur_color = vec3<f32>(
        f32(vertex.blur_color.r) / 255.0,
        f32(vertex.blur_color.g) / 255.0,
        f32(vertex.blur_color.b) / 255.0,
    );
    fragment.glyph_min = fragment.glyph;
    fragment.glyph_max = fragment.glyph + fragment.size;
    var blur_diameter = f32(2 * fragment.blur_radius + 1);
    fragment.weight = blur_diameter * blur_diameter;
    fragment.pxw = 1.0 / f32(sizes.texture.x);
    fragment.pxh = 1.0 / f32(sizes.texture.y);

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
    let uv = fragment.texture;

    switch fragment.ty {
        // Background rectangle
        case 0u {
            return fragment.color;
        }
        // Mask glyph
        case 1u {
            if fragment.blur_radius != 0 {
                var r = fragment.blur_radius;
                var pxw = fragment.pxw;
                var pxh = fragment.pxh;
                var min = fragment.glyph_min;
                var max = fragment.glyph_max;
                var weight = fragment.weight;

                var mask = 0.0;
                var inside = min <= uv & uv < max;
                if inside.x && inside.y {
                    mask = textureSampleLevel(mask_texture, texture_sampler, uv, 0.0).r;
                }

                var m = 0.0;
                for (var i = -r; i <= r; i++) {
                    for (var j = -r; j <= r; j++) {
                        var position = uv + vec2<f32>(f32(i) * pxw, f32(j) * pxh);
                        var inside = min <= position & position < max;
                        if inside.x && inside.y {
                            m += textureSampleLevel(mask_texture, texture_sampler, position, 0.0).r / weight;
                        }
                    }
                }

                return vec4<f32>(
                    mix(fragment.blur_color.r, fragment.color.r, mask),
                    mix(fragment.blur_color.g, fragment.color.g, mask),
                    mix(fragment.blur_color.b, fragment.color.b, mask),
                    fragment.color.a * m,
                );
            } else {
                var mask = textureSampleLevel(mask_texture, texture_sampler, uv, 0.0).r;
                return vec4<f32>(
                    fragment.color.r,
                    fragment.color.g,
                    fragment.color.b,
                    fragment.color.a * mask,
                );
            }

        }
        // Color glyph
        case 2u {
            return textureSampleLevel(color_texture, texture_sampler, uv, 0.0);
        }
        default {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
    }
}
