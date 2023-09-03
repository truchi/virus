var<push_constant> CONSTANTS: Constants;

struct Constants {
    // Surface `(width, height)` size.
    surface: vec2f,
}

struct Instance {
    @builtin(vertex_index) index: u32,
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
    let width = f32(instance.size.x);
    let height = f32(instance.size.y);
    let top = f32(instance.position.x);
    let left = f32(instance.position.y);
    let bottom = top + height;
    let right = left + width;

    var position: vec2f;
    switch instance.index {
        // Bottom right
        case 1u, 5u: {
            position = vec2f(right, bottom);
        }
        // Top right
        case 2u: {
            position = vec2f(right, top);
        }
        // Bottom left
        case 4u: {
            position = vec2f(left, bottom);
        }
        // Top left
        default: {
            position = vec2f(left, top);
        }
    }

    var fragment: Fragment;
    fragment.position = vec4f(
        0.0 + 2.0 * position.x / CONSTANTS.surface.x - 1.0,
        0.0 - 2.0 * position.y / CONSTANTS.surface.y + 1.0,
        0.0,
        1.0,
    );
    fragment.color = pow(vec4f(instance.color) / 255.0, vec4f(2.2, 2.2, 2.2, 1.0));

    return fragment;
}

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
