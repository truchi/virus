use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Atlases                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Atlases {
    mask_atlas: Allocator<Horizontal, GlyphKey, Placement>,
    mask_texture: Texture,
    color_atlas: Allocator<Horizontal, GlyphKey, Placement>,
    color_texture: Texture,
    animated_atlas: Allocator<Horizontal, AnimatedGlyphKey, Placement>,
    animated_texture: Texture,
}

impl Atlases {
    const MASK_BIN: u32 = 400;
    const COLOR_BIN: u32 = 400;
    const ANIMATED_BIN: u32 = 400;

    pub fn new(mask_texture: Texture, color_texture: Texture, animated_texture: Texture) -> Self {
        Self {
            mask_atlas: Allocator::new(mask_texture.width(), mask_texture.height(), Self::MASK_BIN),
            mask_texture,
            color_atlas: Allocator::new(
                color_texture.width(),
                color_texture.height(),
                Self::COLOR_BIN,
            ),
            color_texture,
            animated_atlas: Allocator::new(
                animated_texture.width(),
                animated_texture.height(),
                Self::ANIMATED_BIN,
            ),
            animated_texture,
        }
    }

    pub fn mask_texture(&self) -> &Texture {
        &self.mask_texture
    }

    pub fn color_texture(&self) -> &Texture {
        &self.color_texture
    }

    pub fn animated_texture(&self) -> &Texture {
        &self.animated_texture
    }

    pub fn insert_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        [top, left]: [i32; 2],
        font_size: FontSize,
        line_height: LineHeight,
        time: Duration,
        glyph: &Glyph,
    ) -> Option<(u32, ([i32; 2], [u32; 2]), [u32; 2])> {
        let animated = |(glyph_type, placement, [u, v]): (u32, Placement, [u32; 2])| {
            // Animated glyph has screen coordinate system, from top of line
            let center = ((line_height as f32 - placement.height as f32) / 2.0).round() as i32;
            let top = top - placement.top + center;
            let left = left + placement.left;

            (
                glyph_type,
                ([top, left], [placement.width, placement.height]),
                [u, v],
            )
        };

        let non_animated = |(glyph_type, placement, [u, v]): (u32, Placement, [u32; 2])| {
            // Swash image placement has vertical up, from baseline
            let top = top + font_size as i32 - placement.top;
            let left = left + placement.left;

            (
                glyph_type,
                ([top, left], [placement.width, placement.height]),
                [u, v],
            )
        };

        if glyph.is_animated() {
            self.insert_animated_glyph(queue, scaler, glyph, time)
                .map(animated)
        } else {
            self.insert_non_animated_glyph(queue, scaler, glyph)
                .map(non_animated)
        }
    }
}

/// Private.
impl Atlases {
    fn insert_non_animated_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        glyph: &Glyph,
    ) -> Option<(u32, Placement, [u32; 2])> {
        let key = glyph.key();

        // Check atlases for glyph
        if let Some((glyph_type, ([u, v], placement))) = {
            let in_mask = || self.mask_atlas.get(&key).map(|v| (MASK_GLYPH, v));
            let in_color = || self.color_atlas.get(&key).map(|v| (COLOR_GLYPH, v));
            in_mask().or_else(in_color)
        } {
            return Some((glyph_type, *placement, [u, v]));
        }

        // Render glyph
        let image = scaler.render(&glyph)?;
        let placement = image.placement;
        let [width, height] = [placement.width, placement.height];

        // Allocate glyph in atlas
        let (glyph_type, atlas, texture, channels) = match image.content {
            Content::Mask => (MASK_GLYPH, &mut self.mask_atlas, &self.mask_texture, 1),
            Content::Color => (COLOR_GLYPH, &mut self.color_atlas, &self.color_texture, 4),
            Content::SubpixelMask => unreachable!(),
        };
        let ([u, v], _) = atlas.insert(key, placement, [width, height]).unwrap();

        // Insert glyph in atlas
        queue.write_texture(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d { x: u, y: v, z: 0 },
                aspect: TextureAspect::All,
            },
            &image.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(channels * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Some((glyph_type, placement, [u, v]))
    }

    fn insert_animated_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        glyph: &Glyph,
        time: Duration,
    ) -> Option<(u32, Placement, [u32; 2])> {
        let id = glyph.animated_id?;
        let key = (glyph.size, id, scaler.frame(glyph, time)?);

        // Check atlas for frame
        if let Some(([u, v], placement)) = self.animated_atlas.get(&key) {
            return Some((ANIMATED_GLYPH, *placement, [u, v]));
        }

        // Render frames
        let frames = scaler.render_animated(&glyph)?;
        debug_assert!(FrameIndex::try_from(frames.len()).is_ok());

        for (index, frame) in frames.iter().enumerate() {
            let placement = frame.placement;
            let [width, height] = [placement.width, placement.height];

            // Allocate frame in atlas
            let ([u, v], _) = self
                .animated_atlas
                .insert(
                    (glyph.size, id, index as FrameIndex),
                    placement,
                    [width, height],
                )
                .unwrap();

            // Insert frame in atlas
            queue.write_texture(
                ImageCopyTexture {
                    texture: &self.animated_texture,
                    mip_level: 0,
                    origin: Origin3d { x: u, y: v, z: 0 },
                    aspect: TextureAspect::All,
                },
                &frame.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let ([u, v], placement) = self.animated_atlas.get(&key).unwrap();
        Some((ANIMATED_GLYPH, *placement, [u, v]))
    }
}
