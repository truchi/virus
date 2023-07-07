use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            GlyphType                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

unsafe impl bytemuck::Zeroable for GlyphType {}
unsafe impl bytemuck::Pod for GlyphType {}

/// [`GlyphType::MASK`]/[`GlyphType::COLOR`].
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GlyphType(u32);

impl GlyphType {
    pub const MASK: Self = Self(0);
    pub const COLOR: Self = Self(1);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         RectangleVertex                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

unsafe impl bytemuck::Zeroable for RectangleVertex {}
unsafe impl bytemuck::Pod for RectangleVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RectangleVertex {
    /// Region `[top, left]` world coordinates.
    region_position: [i32; 2],
    /// Region `[width, height]` size.
    region_size: [u32; 2],
    /// Vertex `[top, left]` coordinates in region.
    position: [i32; 2],
    /// sRGBA color.
    color: [u32; 4],
}

impl RectangleVertex {
    const ATTRIBUTES: [VertexAttribute; 4] = vertex_attr_array![
        0 => Sint32x2, // region position
        1 => Uint32x2, // region size
        2 => Sint32x2, // position
        3 => Uint32x4, // color
    ];

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        color: Rgba,
    ) -> Self {
        Self {
            region_position: [region_top, region_left],
            region_size: [region_width, region_height],
            position: [top, left],
            color: [
                color.r as u32,
                color.g as u32,
                color.b as u32,
                color.a as u32,
            ],
        }
    }

    pub fn quad(
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) -> [Self; 4] {
        let region = ([region_top, region_left], [region_width, region_height]);
        let right = left + width as i32;
        let bottom = top + height as i32;

        [
            Self::new(region, [top, left], color),
            Self::new(region, [top, right], color),
            Self::new(region, [bottom, left], color),
            Self::new(region, [bottom, right], color),
        ]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           ShadowVertex                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

unsafe impl bytemuck::Zeroable for ShadowVertex {}
unsafe impl bytemuck::Pod for ShadowVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ShadowVertex {
    /// Glyph type.
    glyph_type: GlyphType,
    /// Vertex `[top, left]` coordinates in region.
    position: [i32; 2],
    /// Texture `[x, y]` coordinates.
    uv: [u32; 2],
}

impl ShadowVertex {
    const ATTRIBUTES: [VertexAttribute; 3] = vertex_attr_array![
        0 => Uint32,   // ty
        1 => Sint32x2, // position
        2 => Uint32x2, // uv
    ];

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(glyph_type: GlyphType, [top, left]: [i32; 2], uv: [u32; 2]) -> Self {
        Self {
            glyph_type,
            position: [top, left],
            uv,
        }
    }

    pub fn quad(
        glyph_type: GlyphType,
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        [u, v]: [u32; 2],
    ) -> [Self; 4] {
        let right = left + width as i32;
        let bottom = top + height as i32;
        let u2 = u + width;
        let v2 = v + height;

        [
            Self::new(glyph_type, [top, left], [u, v]),
            Self::new(glyph_type, [top, right], [u2, v]),
            Self::new(glyph_type, [bottom, left], [u, v2]),
            Self::new(glyph_type, [bottom, right], [u2, v2]),
        ]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           GlyphVertex                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

unsafe impl bytemuck::Zeroable for GlyphVertex {}
unsafe impl bytemuck::Pod for GlyphVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GlyphVertex {
    /// Glyph type.
    glyph_type: GlyphType,
    /// Region `[top, left]` world coordinates.
    region_position: [i32; 2],
    /// Region `[width, height]` size.
    region_size: [u32; 2],
    /// Vertex `[top, left]` coordinates in region.
    position: [i32; 2],
    /// Texture `[x, y]` coordinates.
    uv: [u32; 2],
    /// sRGBA color.
    color: [u32; 4],
}

impl GlyphVertex {
    const ATTRIBUTES: [VertexAttribute; 6] = vertex_attr_array![
        0 => Uint32,   // ty
        1 => Sint32x2, // region position
        2 => Uint32x2, // region size
        3 => Sint32x2, // position
        4 => Uint32x2, // uv
        5 => Uint32x4, // color
    ];

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(
        glyph_type: GlyphType,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        uv: [u32; 2],
        color: Rgba,
    ) -> Self {
        Self {
            glyph_type,
            region_position: [region_top, region_left],
            region_size: [region_width, region_height],
            position: [top, left],
            uv,
            color: [
                color.r as u32,
                color.g as u32,
                color.b as u32,
                color.a as u32,
            ],
        }
    }

    pub fn quad(
        glyph_type: GlyphType,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        [u, v]: [u32; 2],
        color: Rgba,
    ) -> [Self; 4] {
        let region = ([region_top, region_left], [region_width, region_height]);
        let right = left + width as i32;
        let bottom = top + height as i32;
        let u2 = u + width;
        let v2 = v + height;

        [
            Self::new(glyph_type, region, [top, left], [u, v], color),
            Self::new(glyph_type, region, [top, right], [u2, v], color),
            Self::new(glyph_type, region, [bottom, left], [u, v2], color),
            Self::new(glyph_type, region, [bottom, right], [u2, v2], color),
        ]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           BlurVertex                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

unsafe impl bytemuck::Zeroable for BlurVertex {}
unsafe impl bytemuck::Pod for BlurVertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlurVertex {
    /// Region `[top, left]` world coordinates.
    region_position: [i32; 2],
    /// Region `[width, height]` size.
    region_size: [u32; 2],
    /// Vertex `[top, left]` coordinates in region (in output texture).
    shadow_position: [i32; 2],
    /// Vertex `[top, left]` coordinates (in blur textures).
    blur_position: [u32; 2],
    /// Blur radius.
    radius: u32,
    /// sRGBA color.
    color: [u32; 4],
}

impl BlurVertex {
    const ATTRIBUTES: [VertexAttribute; 6] = vertex_attr_array![
        0 => Sint32x2, // region position
        1 => Uint32x2, // region size
        2 => Sint32x2, // shadow position
        3 => Uint32x2, // blur position
        4 => Uint32,   // radius
        5 => Uint32x4, // color
    ];

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [shadow_top, shadow_left]: [i32; 2],
        [blur_top, blur_left]: [u32; 2],
        shadow: Shadow,
    ) -> Self {
        Self {
            region_position: [region_top, region_left],
            region_size: [region_width, region_height],
            shadow_position: [shadow_top, shadow_left],
            blur_position: [blur_top, blur_left],
            radius: shadow.radius as u32,
            color: [
                shadow.color.r as u32,
                shadow.color.g as u32,
                shadow.color.b as u32,
                shadow.color.a as u32,
            ],
        }
    }

    pub fn quad(
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [shadow_top, shadow_left]: [i32; 2],
        [blur_top, blur_left]: [u32; 2],
        [width, height]: [u32; 2],
        shadow: Shadow,
    ) -> [Self; 4] {
        let region = ([region_top, region_left], [region_width, region_height]);
        let shadow_right = shadow_left + width as i32;
        let shadow_bottom = shadow_top + height as i32;
        let blur_right = blur_left + width;
        let blur_bottom = blur_top + height;

        [
            Self::new(
                region,
                [shadow_top, shadow_left],
                [blur_top, blur_left],
                shadow,
            ),
            Self::new(
                region,
                [shadow_top, shadow_right],
                [blur_top, blur_right],
                shadow,
            ),
            Self::new(
                region,
                [shadow_bottom, shadow_left],
                [blur_bottom, blur_left],
                shadow,
            ),
            Self::new(
                region,
                [shadow_bottom, shadow_right],
                [blur_bottom, blur_right],
                shadow,
            ),
        ]
    }
}
