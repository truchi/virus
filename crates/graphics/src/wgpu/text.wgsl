// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Sizes {
    surface: vec2<u32>,
    texture: vec2<u32>,
}

struct VertexInput {
    @location(0) ty: u32,
    @location(1) position: vec3<i32>,
    @location(2) texture: vec2<u32>,
    @location(3) color: vec4<u32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ty: u32,
    @location(1) texture: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> sizes: Sizes;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.ty = input.ty;
    output.texture = vec2<f32>(
        f32(input.texture.x) / f32(sizes.texture.x),
        f32(input.texture.y) / f32(sizes.texture.y),
    );
    output.position = vec4<f32>(
        f32(input.position.x) / f32(sizes.surface.x) - 1.0,
        1.0 - f32(input.position.y) / f32(sizes.surface.y),
        f32(input.position.z),
        1.0,
    );
    output.color = vec4<f32>(
      f32(input.color.r) / 255.0,
      f32(input.color.g) / 255.0,
      f32(input.color.b) / 255.0,
      f32(input.color.a) / 255.0,
    );

    return output;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(1) var mask_texture: texture_2d<f32>;
@group(0) @binding(2) var color_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    switch input.ty {
        // Background rectangle
        case 0u {
            return input.color;
        }
        // Mask glyph
        case 1u {
            var mask = textureSampleLevel(mask_texture, texture_sampler, input.texture, 0.0).r;

            return vec4<f32>(
                input.color.r,
                input.color.g,
                input.color.b,
                input.color.a * mask,
            );
        }
        // Color glyph
        case 2u {
            return textureSampleLevel(color_texture, texture_sampler, input.texture, 0.0);
        }
        default {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
    }
}
