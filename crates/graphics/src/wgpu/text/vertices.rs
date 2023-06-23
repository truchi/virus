use super::*;

pub const MASK_GLYPH: u32 = 0;
pub const COLOR_GLYPH: u32 = 1;
pub const ANIMATED_GLYPH: u32 = 2;

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
    /// Glyph type ([`MASK_GLYPH`]/[`COLOR_GLYPH`]/[`ANIMATED_GLYPH`]).
    glyph_type: u32,
    /// Region `[top, left]` world coordinates.
    region_position: [i32; 2],
    /// Region `[width, height]` size.
    region_size: [u32; 2],
    /// Vertex `[top, left]` coordinates in region.
    position: [i32; 2],
    /// Texture `[x, y]` coordinates.
    uv: [u32; 2],
}

impl ShadowVertex {
    const ATTRIBUTES: [VertexAttribute; 5] = vertex_attr_array![
        0 => Uint32,   // ty
        1 => Sint32x2, // region position
        2 => Uint32x2, // region size
        3 => Sint32x2, // position
        4 => Uint32x2, // uv
    ];

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(
        glyph_type: u32,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        uv: [u32; 2],
    ) -> Self {
        Self {
            glyph_type,
            region_position: [region_top, region_left],
            region_size: [region_width, region_height],
            position: [top, left],
            uv,
        }
    }

    pub fn quad(
        glyph_type: u32,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        [u, v]: [u32; 2],
    ) -> [Self; 4] {
        let region = ([region_top, region_left], [region_width, region_height]);
        let right = left + width as i32;
        let bottom = top + height as i32;
        let u2 = u + width;
        let v2 = v + height;

        [
            Self::new(glyph_type, region, [top, left], [u, v]),
            Self::new(glyph_type, region, [top, right], [u2, v]),
            Self::new(glyph_type, region, [bottom, left], [u, v2]),
            Self::new(glyph_type, region, [bottom, right], [u2, v2]),
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
    /// Glyph type ([`MASK_GLYPH`]/[`COLOR_GLYPH`]/[`ANIMATED_GLYPH`]).
    glyph_type: u32,
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
        glyph_type: u32,
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
        glyph_type: u32,
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

// TODO
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlurVertex {
    position: [i32; 2],
}

impl BlurVertex {
    const ATTRIBUTES: [VertexAttribute; 1] = vertex_attr_array![
        0 => Sint32x2, // position
    ];

    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new() -> Self {
        todo!()
    }

    pub fn quad() -> [Self; 4] {
        todo!()
    }
}
