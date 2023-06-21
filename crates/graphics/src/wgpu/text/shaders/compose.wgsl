struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) texture: u32,
    @location(1) uv: vec2f,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@vertex
fn vertex(
    @builtin(vertex_index) vertex: u32,
    @builtin(instance_index) instance: u32,
) -> Fragment {
    var fragment: Fragment;
    switch vertex {
        case 0u { fragment.position = vec4f(-1.0,  1.0, 0.0, 1.0); } // Top left
        case 1u { fragment.position = vec4f( 1.0, -1.0, 0.0, 1.0); } // Bottom right
        case 2u { fragment.position = vec4f( 1.0,  1.0, 0.0, 1.0); } // Top right
        case 3u { fragment.position = vec4f(-1.0,  1.0, 0.0, 1.0); } // Top left
        case 4u { fragment.position = vec4f(-1.0, -1.0, 0.0, 1.0); } // Bottom left
        case 5u { fragment.position = vec4f( 1.0, -1.0, 0.0, 1.0); } // Bottom right
        default {}
    }
    switch vertex {
        case 0u { fragment.uv = vec2f(0.0, 0.0); } // Top left
        case 1u { fragment.uv = vec2f(1.0, 1.0); } // Bottom right
        case 2u { fragment.uv = vec2f(1.0, 0.0); } // Top right
        case 3u { fragment.uv = vec2f(0.0, 0.0); } // Top left
        case 4u { fragment.uv = vec2f(0.0, 1.0); } // Bottom left
        case 5u { fragment.uv = vec2f(1.0, 1.0); } // Bottom right
        default {}
    }
    fragment.texture = instance;

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var rectangle_texture: texture_2d<f32>;
@group(0) @binding(1) var blur_texture: texture_2d<f32>;
@group(0) @binding(2) var glyph_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    switch fragment.texture {
        // Rectangle texture
        case 0u {
            return textureSampleLevel(rectangle_texture, texture_sampler, fragment.uv, 0.0);
        }
        // Blur texture
        case 1u {
            let color = textureSampleLevel(blur_texture, texture_sampler, fragment.uv, 0.0);
            //return color;
            let alpha = color.a;
            let blur = vec4f(1.0, 0.0, 0.0, 1.0);
            return vec4f(blur.rgb, blur.a * alpha);
        }
        // Glyph texture
        case 2u {
            return textureSampleLevel(glyph_texture, texture_sampler, fragment.uv, 0.0);
        }
        default {
            return vec4f(0.0, 0.0, 0.0, 0.0);
        }
    }
}
