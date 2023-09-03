var<push_constant> CONSTANTS: Constants;

struct Constants {
    // Surface `(width, height)` size.
    surface: vec2f,
}

struct Instance {
    @builtin(vertex_index) my_index: u32,
    // Vertex `(top, left)` position.
    @location(0) position: vec2i,
    // Vertex `(width, height)` size.
    @location(1) size: vec2u,
    // sRGBA color.
    @location(2) color: vec4u,
}

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vertex(instance: Instance) -> Fragment {
    // TODO

    let region_position = vec2f(instance.region_position.yx);
    let region_size = vec2f(instance.region_size);
    let position = vec2f(instance.position.yx);
    let color = vec4f(instance.color) / 255.0;

    var fragment: Fragment;
    fragment.position = vec4f(
        0.0 + 2.0 * (position.x + region_position.x) / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * (position.y + region_position.y) / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.color = pow(color, vec4f(2.2, 2.2, 2.2, 1.0));
    fragment.min = region_position;
    fragment.max = region_position + region_size;

    return fragment;
}

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
