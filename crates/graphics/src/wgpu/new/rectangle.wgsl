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
//                                            Instance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Instance {
    @builtin(vertex_index) index: u32,
    // Rectangle `(top, left)` position.
    @location(0) position: vec2i,
    // Rectangle `(width, height)` size.
    @location(1) size: vec2u,
    // Rectangle sRGBA color.
    @location(2) color: vec4u,
}

fn position(index: u32, position: vec2i, size: vec2u) -> vec2f {
    let width = f32(size.x);
    let height = f32(size.y);
    let top = f32(position.x);
    let left = f32(position.y);
    let bottom = top + height;
    let right = left + width;

    switch index {
        // Top left
        case 0u: { return vec2f(left, top); }
        // Bottom right
        case 1u: { return vec2f(right, bottom); }
        // Top right
        case 2u: { return vec2f(right, top); }
        // Top left
        case 3u: { return vec2f(left, top); }
        // Bottom left
        case 4u: { return vec2f(left, bottom); }
        // Bottom right
        case 5u: { return vec2f(right, bottom); }
        // Unreachable
        default: { return vec2f(0.0, 0.0); }
    }
}

fn color(color: vec4u) -> vec4f {
    return pow(vec4f(color) / 255.0, vec4f(2.2, 2.2, 2.2, 1.0));
}

@vertex
fn vertex(instance: Instance) -> Fragment {
    var fragment: Fragment;
    fragment.position = to_clip(position(instance.index, instance.position, instance.size));
    fragment.color = color(instance.color);

    return fragment;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Fragment                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    return fragment.color;
}
