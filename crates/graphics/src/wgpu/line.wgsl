struct Constants {
    // Surface `(width, height)` size.
    surface: vec2f,
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

var<push_constant> CONSTANTS: Constants;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    let region_position = vec2f(vertex.region_position.yx);
    let region_size = vec2f(vertex.region_size);
    let position = vec2f(vertex.position.yx);
    let color = vec4f(vertex.color) / 255.0;

    var fragment: Fragment;
    fragment.position = vec4f(
        0.0 + 2.0 * (position.x + region_position.x) / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * (position.y + region_position.y) / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.color = pow(color, vec4f(2.2, 2.2, 2.2, 1.0));

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
