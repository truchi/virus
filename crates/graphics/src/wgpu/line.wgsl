struct Sizes {
    // Surface `(width, height)` size.
    surface: vec2u,
}

struct Vertex {
    // Screen `(top, left)` coordinates.
    @location(0) position: vec2i,

    // sRGBA color.
    @location(1) color: vec4u,
}

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@group(0) @binding(0) var<uniform> sizes: Sizes;

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    let surface = vec2f(sizes.surface);
    let position = vec2f(vertex.position.yx);
    let color = vec4f(vertex.color);

    var fragment: Fragment;
    fragment.position = vec4f(
        0.0 + 2.0 * position.x / surface.x - 1.0,
        0.0 - 2.0 * position.y / surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.color = color / 255.0;

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4<f32> {
    return fragment.color;
}
