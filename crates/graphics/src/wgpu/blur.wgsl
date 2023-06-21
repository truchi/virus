struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@vertex
fn vertex(
    @builtin(vertex_index) vertex: u32,
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

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var<uniform> direction: u32;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    let dimensions = textureDimensions(texture);
    let w = 1.0 / f32(dimensions.x);
    let h = 1.0 / f32(dimensions.y);
    
    var dir = vec2f(0.0, 0.0);
    if direction == 0u {
        dir = vec2f(w, 0.0);
    } else {
        dir = vec2f(0.0, h);
    }
    
    let radius = 10;
    //let weight = 1.0 + 2.0 * f32(radius);
    let weight = 1.0 + 2.0 * f32(radius) + f32(radius) * f32(radius);
    let original = textureSample(texture, texture_sampler, fragment.uv);
    //var color = original;
    var color = original * f32(radius + 1);
    
    for (var i: i32 = 1; i <= radius; i++) {
        //let factor = 1.0;
        let factor = f32(radius + 1 - i);
        let offset = f32(i) * dir;
        color += textureSample(texture, texture_sampler, fragment.uv - offset) * factor;
        color += textureSample(texture, texture_sampler, fragment.uv + offset) * factor;
    }
    
    color = color / weight;
    return vec4f(1.0, 0.0, 0.0, color.a);
}
