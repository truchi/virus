struct Sizes {
    surface: vec2<u32>,
}

struct Vertex {
    @location(0) position: vec2<i32>,
    @location(1) color: vec4<u32>,
}

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var<uniform> sizes: Sizes;

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    var fragment: Fragment;
    fragment.position = vec4<f32>(
        f32(vertex.position.x) / f32(sizes.surface.x) - 1.0,
        1.0 - f32(vertex.position.y) / f32(sizes.surface.y),
        0.0,
        1.0,
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

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4<f32> {
    return fragment.color;
}
