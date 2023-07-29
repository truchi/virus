use super::*;

// How many times the surface must the altases be.
// (The color atlas is at `limits.max_texture_dimension_2d`)
const MASK_ATLAS_FACTOR: u32 = 1;
const BLUR_ATLAS_FACTOR: u32 = 2;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPipeline                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Text pipeline.
#[derive(Debug)]
pub struct TextPipeline {
    constants: Constants,
    buffers: Buffers,
    atlases: Atlases,
    bind_group_layout: BindGroupLayout,
    ping_bind_group: BindGroup,
    pong_bind_group: BindGroup,
    rectangle_pipeline: RenderPipeline,
    shadow_pipeline: RenderPipeline,
    glyph_pipeline: RenderPipeline,
    blur_ping_pipeline: RenderPipeline,
    blur_pong_pipeline: RenderPipeline,
}

impl TextPipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let max_buffer_size = limits.max_buffer_size as usize;
        let max_texture_dimension = limits.max_texture_dimension_2d;

        let constants = Constants {
            surface: [config.width as f32, config.height as f32],
        };
        let buffers = Init(device).buffers(max_buffer_size);
        let atlases = Atlases::new(
            Init(device).mask_texture([
                MASK_ATLAS_FACTOR * config.width,
                MASK_ATLAS_FACTOR * config.height,
            ]),
            Init(device).color_texture([max_texture_dimension, max_texture_dimension]),
            Init(device).blur_textures([
                BLUR_ATLAS_FACTOR * config.width,
                BLUR_ATLAS_FACTOR * config.height,
            ]),
        );
        let bind_group_layout = Init(device).bind_group_layout();
        let [ping_bind_group, pong_bind_group] = Init(device).bind_groups(
            &bind_group_layout,
            atlases.mask_texture(),
            atlases.color_texture(),
            &atlases.blur_ping_texture(),
            &atlases.blur_pong_texture(),
        );
        let [rectangle_pipeline, shadow_pipeline, glyph_pipeline, blur_ping_pipeline, blur_pong_pipeline] =
            Init(device).pipelines(config, &bind_group_layout);

        Self {
            constants,
            buffers,
            atlases,
            bind_group_layout,
            ping_bind_group,
            pong_bind_group,
            rectangle_pipeline,
            shadow_pipeline,
            glyph_pipeline,
            blur_ping_pipeline,
            blur_pong_pipeline,
        }
    }

    pub fn blur_ping_texture_view(&self) -> TextureView {
        self.atlases
            .blur_ping_texture()
            .create_view(&Default::default())
    }

    pub fn blur_pong_texture_view(&self) -> TextureView {
        self.atlases
            .blur_pong_texture()
            .create_view(&Default::default())
    }

    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.constants.surface = [config.width as f32, config.height as f32];
        self.atlases.resize_mask(Init(device).mask_texture([
            MASK_ATLAS_FACTOR * config.width,
            MASK_ATLAS_FACTOR * config.height,
        ]));
        self.atlases.resize_blur(Init(device).blur_textures([
            BLUR_ATLAS_FACTOR * config.width,
            BLUR_ATLAS_FACTOR * config.height,
        ]));
        [self.ping_bind_group, self.pong_bind_group] = Init(device).bind_groups(
            &self.bind_group_layout,
            &self.atlases.mask_texture(),
            &self.atlases.color_texture(),
            &self.atlases.blur_ping_texture(),
            &self.atlases.blur_pong_texture(),
        );
    }

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        let layer = 0; // TODO

        self.buffers.push_rectangle(
            layer,
            RectangleVertex::quad(
                ([region_top, region_left], [region_width, region_height]),
                ([top, left], [width, height]),
                color,
            ),
        );
    }

    pub fn glyphs(
        &mut self,
        queue: &Queue,
        context: &mut Context,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        line: &Line,
        line_height: LineHeight,
        time: Duration,
    ) {
        let layer = 0; // TODO
        let region = ([region_top, region_left], [region_width, region_height]);
        let mut scaler = line.scaler(context);

        // Discard when outside region. This suppposes that:
        // - glyphs are not bigger that line height (~ font size < line height)
        // - glyphs outside do not affect what's inside (~ no shadows, but that's fine)
        // - no further transforms are applied in the shader
        // Of course the GPU would have done that for us. Don't fear to remove if necessary.
        {
            let above = top + (line_height as i32) <= 0;
            let below = top >= region_height as i32;

            if above || below {
                return;
            }
        }

        //
        // Add blurs
        //

        for (Range { start, end }, glyphs, shadow) in line
            .segments(|glyph| glyph.styles.shadow)
            .filter_map(|(range, glyphs, shadow)| {
                shadow
                    .and_then(|shadow| shadow.color.is_visible().then_some((range, glyphs, shadow)))
            })
        {
            let [width, height] = [
                2 * shadow.radius as u32 + (end - start).ceil() as u32,
                2 * shadow.radius as u32 + line_height,
            ];
            let [shadow_top, shadow_left] = [
                -(shadow.radius as i32) + top,
                -(shadow.radius as i32) + left + start.round() as i32,
            ];
            let [blur_top, blur_left] =
                if let Some([top, left]) = self.atlases.insert_blur([width, height]) {
                    [top, left]
                } else {
                    // Atlas/Texture is too small, forget about this shadow
                    // This can happen at startup/resize, and this is fine
                    // It can also happen when there is too much text to blur, with large radius...
                    continue;
                };

            self.buffers.push_blur(
                layer,
                BlurVertex::quad(
                    region,
                    [shadow_top, shadow_left],
                    [blur_top, blur_left],
                    [width, height],
                    shadow,
                ),
            );

            //
            // Add shadows
            //

            for glyph in glyphs {
                let (glyph_type, ([top, left], [width, height]), [u, v]) = if let Some(inserted) =
                    self.atlases.insert_glyph(
                        queue,
                        &mut scaler,
                        [
                            shadow.radius as i32 + blur_top as i32,
                            shadow.radius as i32
                                + blur_left as i32
                                + (glyph.offset - start).round() as i32,
                        ],
                        line.font_size(),
                        line_height,
                        time,
                        glyph,
                    ) {
                    inserted
                } else {
                    continue;
                };

                self.buffers.push_shadow(
                    layer,
                    ShadowVertex::quad(glyph_type, ([top, left], [width, height]), [u, v]),
                );
            }
        }

        //
        // Add backgrounds
        //

        for (Range { start, end }, _, background) in line
            .segments(|glyph| glyph.styles.background)
            .filter(|(_, _, background)| background.is_visible())
        {
            let left = left + start as i32;
            let width = (end - start) as u32;

            self.buffers.push_rectangle(
                layer,
                RectangleVertex::quad(
                    ([region_top, region_left], [region_width, region_height]),
                    ([top, left], [width, line_height]),
                    background,
                ),
            );
        }

        //
        // Add glyphs
        //

        for glyph in line
            .glyphs()
            .iter()
            .filter(|glyph| glyph.styles.foreground.is_visible())
        {
            let (glyph_type, ([top, left], [width, height]), [u, v]) = if let Some(inserted) =
                self.atlases.insert_glyph(
                    queue,
                    &mut scaler,
                    [top, left + glyph.offset.round() as i32],
                    line.font_size(),
                    line_height,
                    time,
                    glyph,
                ) {
                inserted
            } else {
                continue;
            };

            self.buffers.push_glyph(
                layer,
                GlyphVertex::quad(
                    glyph_type,
                    region,
                    ([top, left], [width, height]),
                    [u, v],
                    glyph.styles.foreground,
                ),
            );
        }
    }

    pub fn pre_render(&mut self, queue: &Queue) {
        self.buffers.pre_render(queue);
    }

    pub fn render_rectangles<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let layer = 0; // TODO

        self.buffers.render_rectangles(
            layer,
            render_pass,
            &self.constants,
            &self.pong_bind_group,
            &self.rectangle_pipeline,
        );
    }

    pub fn render_shadows<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let layer = 0; // TODO

        self.buffers.render_shadows(
            layer,
            render_pass,
            &self.constants,
            &self.pong_bind_group,
            &self.shadow_pipeline,
        );
    }

    pub fn blur_ping<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let layer = 0; // TODO

        self.buffers.render_blurs(
            layer,
            render_pass,
            &self.constants,
            &self.ping_bind_group,
            &self.blur_ping_pipeline,
        );
    }

    pub fn blur_pong<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let layer = 0; // TODO

        self.buffers.render_blurs(
            layer,
            render_pass,
            &self.constants,
            &self.pong_bind_group,
            &self.blur_pong_pipeline,
        );
    }

    pub fn render_glyphs<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let layer = 0; // TODO

        self.buffers.render_glyphs(
            layer,
            render_pass,
            &self.constants,
            &self.pong_bind_group,
            &self.glyph_pipeline,
        );
    }

    pub fn post_render(&mut self) {
        self.buffers.clear();
        self.atlases.clear_blurs();
    }
}
