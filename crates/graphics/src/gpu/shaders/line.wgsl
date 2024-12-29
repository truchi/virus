var<push_constant> CONSTANTS: Constants;

struct Constants {
    // Surface `(width, height)` size.
    surface: vec2f,
}

fn to_clip(position: vec2f) -> vec4f {
    return vec4f(
        0.0 + 2.0 * position.x / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * position.y / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Vertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Vertex {
    // Region `(top, left)` position.
    @location(0) region_position: vec2i,
    // Region `(width, height)` size.
    @location(1) region_size: vec2u,
    // Point `(top, left)` position.
    @location(2) position: vec2i,
    // Point sRGBA color.
    @location(3) color: vec4u,
}

fn color(color: vec4u) -> vec4f {
    return pow(vec4f(color) / 255.0, vec4f(2.2, 2.2, 2.2, 1.0));
}

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    var fragment: Fragment;
    fragment.position = to_clip(vec2f(vertex.region_position.yx + vertex.position.yx));
    fragment.color = color(vertex.color);
    fragment.min = vec2f(vertex.region_position.yx);
    fragment.max = vec2f(vertex.region_position.yx) + vec2f(vertex.region_size);

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) min: vec2f,
    @location(2) max: vec2f,
}

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    let inside = fragment.min <= fragment.position.xy & fragment.position.xy < fragment.max;
    if !(inside.x && inside.y) {
        discard;
    }

    return fragment.color;
}
