struct Sizes {
    // Surface `(width, height)` size.
    surface: vec2u,
}

struct Vertex {
    // Region `(top, left)` world coordinates.
    @location(0) region_position: vec2i,

    // Region `(width, height)` size.
    @location(1) region_size: vec2u,

    // Vertex `(top, left)` coordinates in region.
    @location(2) position: vec2i,

    // sRGBA color.
    @location(3) color: vec4u,
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
    let region_position = vec2f(vertex.region_position.yx);
    let region_size = vec2f(vertex.region_size);
    let position = vec2f(vertex.position.yx);
    let color = vec4f(vertex.color);

    var fragment: Fragment;
    fragment.position = vec4f(
        0.0 + 2.0 * (position.x + region_position.x) / surface.x - 1.0,
        0.0 - 2.0 * (position.y + region_position.y) / surface.y + 1.0,
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
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
