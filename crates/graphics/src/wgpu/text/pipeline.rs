use super::*;

// TODO
// - Confusion with pixel/channel width in atlas/texture?
// - Blur atlas/textures must be bigger than output surface
// - Early discard lines only when shadows don't bleed inside

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPipeline                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Text pipeline.
#[derive(Debug)]
pub struct TextPipeline {
    //
    // Constants
    //
    constants: Constants,

    //
    // Vertices and indices
    //
    rectangles: Buffers<RectangleVertex>,
    shadows: Buffers<ShadowVertex>,
    glyphs: Buffers<GlyphVertex>,
    blurs: Buffers<BlurVertex>,

    //
    // Atlases and textures
    //
    atlases: Atlases,
    blur_atlas: Allocator<Horizontal, usize, ()>,
    blur_ping_texture: Texture,
    blur_pong_texture: Texture,

    //
    // Bind group and pipelines
    //
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
    pub const ALTAS_ROW_HEIGHT: u32 = 400;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let max_buffer_size = limits.max_buffer_size as usize;
        let max_texture_dimension = limits.max_texture_dimension_2d;

        // Constants
        let constants = Constants {
            surface: [config.width as f32, config.height as f32],
            texture: [max_texture_dimension as f32, max_texture_dimension as f32],
        };

        // Buffers
        let (rectangles, shadows, glyphs, blurs) = Init(device).buffers(max_buffer_size);

        // Atlases and textures
        let (mask_texture, color_texture, animated_texture) =
            Init(device).atlases(max_texture_dimension);
        let atlases = Atlases::new(mask_texture, color_texture, animated_texture);
        let blur_atlas = Allocator::new(config.width, config.height, Self::ALTAS_ROW_HEIGHT);
        let [blur_ping_texture, blur_pong_texture] = Init(device).blur_textures(config);

        // Bind groups
        let bind_group_layout = Init(device).bind_group_layout();
        let [ping_bind_group, pong_bind_group] = Init(device).bind_groups(
            &bind_group_layout,
            atlases.mask_texture(),
            atlases.color_texture(),
            atlases.animated_texture(),
            &blur_ping_texture,
            &blur_pong_texture,
        );

        // Pipelines
        let [rectangle_pipeline, shadow_pipeline, glyph_pipeline, blur_ping_pipeline, blur_pong_pipeline] =
            Init(device).pipelines(config, &bind_group_layout);

        Self {
            constants,
            rectangles,
            shadows,
            glyphs,
            blurs,
            atlases,
            blur_atlas,
            blur_ping_texture,
            blur_pong_texture,
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
        self.blur_ping_texture.create_view(&Default::default())
    }

    pub fn blur_pong_texture_view(&self) -> TextureView {
        self.blur_pong_texture.create_view(&Default::default())
    }

    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.constants.surface = [config.width as f32, config.height as f32];

        self.blur_atlas
            .clear_and_resize(Self::ALTAS_ROW_HEIGHT, config.width, config.height);
        let [blur_ping_texture, blur_pong_texture] = Init(device).blur_textures(config);
        self.blur_ping_texture = blur_ping_texture;
        self.blur_pong_texture = blur_pong_texture;

        let [ping_bind_group, pong_bind_group] = Init(device).bind_groups(
            &self.bind_group_layout,
            &self.atlases.mask_texture(),
            &self.atlases.color_texture(),
            &self.atlases.animated_texture(),
            &self.blur_ping_texture,
            &self.blur_pong_texture,
        );
        self.ping_bind_group = ping_bind_group;
        self.pong_bind_group = pong_bind_group;
    }

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        self.rectangles.push(RectangleVertex::quad(
            ([region_top, region_left], [region_width, region_height]),
            ([top, left], [width, height]),
            color,
        ));
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
        let region = ([region_top, region_left], [region_width, region_height]);
        let mut scaler = line.scaler(context);

        // Discard when outside region. This suppposes that:
        // - glyphs are not bigger that line height (~ font size < line height)
        // - glyphs outside do not affect what's inside (~ no shadows, TODO oops)
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
            let key = self.blur_atlas.len();
            let [width, height] = [
                2 * shadow.radius as u32 + (end - start).ceil() as u32,
                2 * shadow.radius as u32 + line_height,
            ];
            let [shadow_top, shadow_left] = [
                -(shadow.radius as i32) + top,
                -(shadow.radius as i32) + left + start.round() as i32,
            ];
            let [blur_left, blur_top] =
                if let Ok(([x, y], ())) = self.blur_atlas.insert(key, (), [width, height]) {
                    [x, y]
                } else {
                    // Atlas/Texture is too small, forget about this shadow
                    // This can happen at startup/resize, and this is fine
                    // It can also happen when there is too much text to blur, with large radius...
                    continue;
                };

            self.blurs.push(BlurVertex::quad(
                region,
                [shadow_top, shadow_left],
                [blur_top, blur_left],
                [width, height],
                shadow,
            ));

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

                self.shadows.push(ShadowVertex::quad(
                    glyph_type,
                    ([top, left], [width, height]),
                    [u, v],
                ));
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

            self.rectangles.push(RectangleVertex::quad(
                ([region_top, region_left], [region_width, region_height]),
                ([top, left], [width, line_height]),
                background,
            ));
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

            self.glyphs.push(GlyphVertex::quad(
                glyph_type,
                region,
                ([top, left], [width, height]),
                [u, v],
                glyph.styles.foreground,
            ));
        }
    }

    pub fn pre_render(&mut self, queue: &Queue) {
        self.rectangles.write(queue);
        self.shadows.write(queue);
        self.glyphs.write(queue);
        self.blurs.write(queue);
    }

    pub fn render_rectangles<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        self.render(
            render_pass,
            &self.rectangles,
            &self.pong_bind_group,
            &self.rectangle_pipeline,
        );
    }

    pub fn render_shadows<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        self.render(
            render_pass,
            &self.shadows,
            &self.pong_bind_group,
            &self.shadow_pipeline,
        );
    }

    pub fn blur_ping<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        self.render(
            render_pass,
            &self.blurs,
            &self.ping_bind_group,
            &self.blur_ping_pipeline,
        );
    }

    pub fn blur_pong<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        self.render(
            render_pass,
            &self.blurs,
            &self.pong_bind_group,
            &self.blur_pong_pipeline,
        );
    }

    pub fn render_glyphs<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        self.render(
            render_pass,
            &self.glyphs,
            &self.pong_bind_group,
            &self.glyph_pipeline,
        );
    }

    pub fn post_render(&mut self) {
        self.rectangles.clear();
        self.shadows.clear();
        self.glyphs.clear();
        self.blurs.clear();
        self.blur_atlas.clear();
    }
}

/// Private.
impl TextPipeline {
    fn render<'pass, T>(
        &self,
        render_pass: &mut RenderPass<'pass>,
        buffers: &'pass Buffers<T>,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
    ) {
        let constants = self.constants.as_array();
        let constants = bytemuck::cast_slice(&constants);

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_push_constants(ShaderStages::VERTEX_FRAGMENT, 0, constants);
        buffers.render(render_pass);
    }
}
