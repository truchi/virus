use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPipeline                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const MAX_RECTANGLES: usize = 1_000;
const MAX_SHADOWS: usize = 10_000;
const MAX_GLYPHS: usize = 10_000;
const MAX_BLURS: usize = 1_000;
const RECTANGLE_VERTEX_SIZE: usize = size_of::<RectangleVertex>();
const SHADOW_VERTEX_SIZE: usize = size_of::<ShadowVertex>();
const GLYPH_VERTEX_SIZE: usize = size_of::<GlyphVertex>();
const BLUR_VERTEX_SIZE: usize = size_of::<GlyphVertex>();
const INDEX_SIZE: usize = size_of::<Index>();

/// Text pipeline.
#[derive(Debug)]
pub struct TextPipeline {
    //
    // Constants
    //
    surface_size: [f32; 2],
    texture_size: [f32; 2],

    //
    // Vertices and indices
    //
    rectangle_vertices: Vec<RectangleVertex>,
    rectangle_indices: Vec<Index>,
    rectangle_vertex_buffer: Buffer,
    rectangle_index_buffer: Buffer,
    shadow_vertices: Vec<ShadowVertex>,
    shadow_indices: Vec<Index>,
    shadow_vertex_buffer: Buffer,
    shadow_index_buffer: Buffer,
    glyph_vertices: Vec<GlyphVertex>,
    glyph_indices: Vec<Index>,
    glyph_vertex_buffer: Buffer,
    glyph_index_buffer: Buffer,
    blur_vertices: Vec<BlurVertex>,
    blur_indices: Vec<Index>,
    blur_vertex_buffer: Buffer,
    blur_index_buffer: Buffer,

    //
    // Atlases and textures
    //
    mask_atlas: Atlas<GlyphKey, Placement>,
    mask_texture: Texture,
    color_atlas: Atlas<GlyphKey, Placement>,
    color_texture: Texture,
    animated_atlas: Atlas<AnimatedGlyphKey, Placement>,
    animated_texture: Texture,
    blur_atlas: Atlas<usize, ()>,
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

        //
        // Buffers
        //

        let surface_size = [config.width as f32, config.height as f32];
        let texture_size = [max_texture_dimension as f32, max_texture_dimension as f32];

        let rectangle_vertex_buffer_size = 4 * MAX_RECTANGLES * RECTANGLE_VERTEX_SIZE;
        let rectangle_index_buffer_size = 6 * MAX_RECTANGLES * INDEX_SIZE;
        let shadow_vertex_buffer_size = 4 * MAX_SHADOWS * SHADOW_VERTEX_SIZE;
        let shadow_index_buffer_size = 6 * MAX_SHADOWS * INDEX_SIZE;
        let glyph_vertex_buffer_size = 4 * MAX_GLYPHS * GLYPH_VERTEX_SIZE;
        let glyph_index_buffer_size = 6 * MAX_GLYPHS * INDEX_SIZE;
        let blur_vertex_buffer_size = 4 * MAX_BLURS * BLUR_VERTEX_SIZE;
        let blur_index_buffer_size = 6 * MAX_BLURS * INDEX_SIZE;
        assert!(rectangle_vertex_buffer_size <= max_buffer_size);
        assert!(rectangle_index_buffer_size <= max_buffer_size);
        assert!(shadow_vertex_buffer_size <= max_buffer_size);
        assert!(shadow_index_buffer_size <= max_buffer_size);
        assert!(glyph_vertex_buffer_size <= max_buffer_size);
        assert!(glyph_index_buffer_size <= max_buffer_size);
        assert!(blur_vertex_buffer_size <= max_buffer_size);
        assert!(blur_index_buffer_size <= max_buffer_size);

        let rectangle_vertices = Vec::with_capacity(4 * MAX_RECTANGLES);
        let rectangle_indices = Vec::with_capacity(6 * MAX_RECTANGLES);
        let rectangle_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Rectangle vertex buffer",
            size: rectangle_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let rectangle_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Rectangle index buffer",
            size: rectangle_index_buffer_size,
            usage: INDEX | COPY_DST,
        });
        let shadow_vertices = Vec::with_capacity(4 * MAX_SHADOWS);
        let shadow_indices = Vec::with_capacity(6 * MAX_SHADOWS);
        let shadow_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Shadow vertex buffer",
            size: shadow_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let shadow_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Shadow index buffer",
            size: shadow_index_buffer_size,
            usage: INDEX | COPY_DST,
        });
        let glyph_vertices = Vec::with_capacity(4 * MAX_GLYPHS);
        let glyph_indices = Vec::with_capacity(6 * MAX_GLYPHS);
        let glyph_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Glyph vertex buffer",
            size: glyph_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let glyph_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Glyph index buffer",
            size: glyph_index_buffer_size,
            usage: INDEX | COPY_DST,
        });
        let blur_vertices = Vec::with_capacity(4 * MAX_BLURS);
        let blur_indices = Vec::with_capacity(6 * MAX_BLURS);
        let blur_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Blur vertex buffer",
            size: blur_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let blur_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Blur index buffer",
            size: blur_index_buffer_size,
            usage: INDEX | COPY_DST,
        });

        //
        // Atlases and textures
        //

        // FIXME do we use atlas/texture sizes properly?

        let mut mask_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            max_texture_dimension,
            max_texture_dimension,
        );
        let mask_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Mask glyphs texture",
            size: [max_texture_dimension, max_texture_dimension],
            format: TextureFormat::R8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let mut color_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            max_texture_dimension,
            max_texture_dimension,
        );
        let color_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Color glyphs texture",
            size: [max_texture_dimension, max_texture_dimension],
            format: TextureFormat::Rgba8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let mut animated_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            max_texture_dimension,
            max_texture_dimension,
        );
        let animated_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Animated glyphs texture",
            size: [max_texture_dimension, max_texture_dimension],
            format: TextureFormat::Rgba8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let mut blur_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, config.width, config.height);
        let blur_ping_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Blur ping texture",
            size: [config.width, config.height],
            format: TextureFormat::R8Unorm,
            usage: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_DST,
        });
        let blur_pong_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Blur pong texture",
            size: [config.width, config.height],
            format: TextureFormat::R8Unorm,
            usage: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_DST,
        });

        mask_atlas.next_frame();
        color_atlas.next_frame();
        animated_atlas.next_frame();
        blur_atlas.next_frame();

        //
        // Bind group
        //

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout! {
            label: "[TextPipeline] Bind group layout",
            entries: [
                // Mask texture
                { binding: 0, visibility: FRAGMENT, ty: Texture },
                // Color texture
                { binding: 1, visibility: FRAGMENT, ty: Texture },
                // Animated texture
                { binding: 2, visibility: FRAGMENT, ty: Texture },
                // Blur texture
                { binding: 3, visibility: FRAGMENT, ty: Texture },
                // Sampler
                { binding: 4, visibility: FRAGMENT, ty: Sampler(Filtering) },
            ],
        });
        let ping_bind_group = device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Ping bind group",
            layout: bind_group_layout,
            entries: [
                // Mask texture
                { binding: 0, resource: Texture(mask_texture) },
                // Color texture
                { binding: 1, resource: Texture(color_texture) },
                // Animated texture
                { binding: 2, resource: Texture(animated_texture) },
                // Ping blur texture
                { binding: 3, resource: Texture(blur_ping_texture) },
                // Sampler
                { binding: 4, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        });
        let pong_bind_group = device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Pong bind group",
            layout: bind_group_layout,
            entries: [
                // Mask texture
                { binding: 0, resource: Texture(mask_texture) },
                // Color texture
                { binding: 1, resource: Texture(color_texture) },
                // Animated texture
                { binding: 2, resource: Texture(animated_texture) },
                // Pong blur texture
                { binding: 3, resource: Texture(blur_pong_texture) },
                // Sampler
                { binding: 4, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        });

        //
        // Pipeline
        //

        let config_target = [Some(ColorTargetState {
            format: config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        let r8_target = [Some(TextureFormat::R8Unorm.into())];
        let layout = device.create_pipeline_layout(&pipeline_layout! {
            label: "[TextPipeline] Pipeline layout",
            bind_group_layouts: [bind_group_layout],
            push_constant_ranges: [(VERTEX_FRAGMENT, 0..16)],
        });
        let module = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));
        let rectangle_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Rectangle pipeline",
            layout: layout,
            module: module,
            vertex: "rectangle_vertex",
            buffers: [RectangleVertex::buffer_layout()],
            fragment: "rectangle_fragment",
            targets: config_target,
            topology: TriangleList,
        });
        let shadow_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Shadow pipeline",
            layout: layout,
            module: module,
            vertex: "shadow_vertex",
            buffers: [ShadowVertex::buffer_layout()],
            fragment: "shadow_fragment",
            targets: r8_target,
            topology: TriangleList,
        });
        let glyph_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Glyph pipeline",
            layout: layout,
            module: module,
            vertex: "glyph_vertex",
            buffers: [GlyphVertex::buffer_layout()],
            fragment: "glyph_fragment",
            targets: config_target,
            topology: TriangleList,
        });
        let blur_ping_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Blur ping pipeline",
            layout: layout,
            module: module,
            vertex: "blur_ping_vertex",
            buffers: [BlurVertex::buffer_layout()],
            fragment: "blur_ping_fragment",
            targets: r8_target,
            topology: TriangleList,
        });
        let blur_pong_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Blur pong pipeline",
            layout: layout,
            module: module,
            vertex: "blur_pong_vertex",
            buffers: [BlurVertex::buffer_layout()],
            fragment: "blur_pong_fragment",
            targets: config_target,
            topology: TriangleList,
        });

        Self {
            surface_size,
            texture_size,
            rectangle_vertices,
            rectangle_indices,
            rectangle_vertex_buffer,
            rectangle_index_buffer,
            shadow_vertices,
            shadow_indices,
            shadow_vertex_buffer,
            shadow_index_buffer,
            glyph_vertices,
            glyph_indices,
            glyph_vertex_buffer,
            glyph_index_buffer,
            blur_vertices,
            blur_indices,
            blur_vertex_buffer,
            blur_index_buffer,
            mask_atlas,
            mask_texture,
            color_atlas,
            color_texture,
            animated_atlas,
            animated_texture,
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
        self.surface_size = [config.width as f32, config.height as f32];

        self.blur_atlas
            .clear_and_resize(Self::ALTAS_ROW_HEIGHT, config.width, config.height);
        self.blur_ping_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Blur ping texture",
            size: [config.width, config.height],
            format: TextureFormat::R8Unorm,
            usage: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_DST,
        });
        self.blur_pong_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Blur pong texture",
            size: [config.width, config.height],
            format: TextureFormat::R8Unorm,
            usage: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_DST,
        });
        self.ping_bind_group = device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Ping bind group",
            layout: self.bind_group_layout,
            entries: [
                // Mask texture
                { binding: 0, resource: Texture(self.mask_texture) },
                // Color texture
                { binding: 1, resource: Texture(self.color_texture) },
                // Animated texture
                { binding: 2, resource: Texture(self.animated_texture) },
                // Ping blur texture
                { binding: 3, resource: Texture(self.blur_ping_texture) },
                // Sampler
                { binding: 4, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        });
        self.pong_bind_group = device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Pong bind group",
            layout: self.bind_group_layout,
            entries: [
                // Mask texture
                { binding: 0, resource: Texture(self.mask_texture) },
                // Color texture
                { binding: 1, resource: Texture(self.color_texture) },
                // Animated texture
                { binding: 2, resource: Texture(self.animated_texture) },
                // Pong blur texture
                { binding: 3, resource: Texture(self.blur_pong_texture) },
                // Sampler
                { binding: 4, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        });
    }

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        Self::push_quad(
            (&mut self.rectangle_vertices, &mut self.rectangle_indices),
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
        let region = ([region_top, region_left], [region_width, region_height]);
        let mut scaler = line.scaler(context);

        // Discard when outside region. This suppposes that:
        // - glyphs are not bigger that line height (~ font size < line height)
        // - glyphs outside do not affect what's inside (~ no shadow FIXME we can check that)
        // - no further transforms are applied in the shader
        // Of course the GPU would have done that for us. Don't fear to remove if necessary.
        {
            // let above = top + (line_height as i32) < 0;
            // let below = top >= region_height as i32;

            // if above || below {
            //     return;
            // }
        }

        //
        // Add blurs
        //

        // TODO blur atlas/textures must be bigger than output surface!

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
            let [blur_left, blur_top] = if self.blur_atlas.insert(key, (), [width, height]).is_ok()
            {
                self.blur_atlas.get(&key).unwrap().0
            } else {
                // Atlas/Texture is too small, forget about this shadow
                // This can happen at startup/resize, and this is fine
                // It can also happen when there is too much text to blur, with large radius...
                continue;
            };

            Self::push_quad(
                (&mut self.blur_vertices, &mut self.blur_indices),
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
                    self.insert_glyph(
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

                Self::push_quad(
                    (&mut self.shadow_vertices, &mut self.shadow_indices),
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

            Self::push_quad(
                (&mut self.rectangle_vertices, &mut self.rectangle_indices),
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
            let (glyph_type, ([top, left], [width, height]), [u, v]) = if let Some(inserted) = self
                .insert_glyph(
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

            Self::push_quad(
                (&mut self.glyph_vertices, &mut self.glyph_indices),
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
        let rectangle_vertices = bytemuck::cast_slice(&self.rectangle_vertices);
        let rectangle_indices = bytemuck::cast_slice(&self.rectangle_indices);
        let shadow_vertices = bytemuck::cast_slice(&self.shadow_vertices);
        let shadow_indices = bytemuck::cast_slice(&self.shadow_indices);
        let glyph_vertices = bytemuck::cast_slice(&self.glyph_vertices);
        let glyph_indices = bytemuck::cast_slice(&self.glyph_indices);
        let blur_vertices = bytemuck::cast_slice(&self.blur_vertices);
        let blur_indices = bytemuck::cast_slice(&self.blur_indices);

        queue.write_buffer(&self.rectangle_vertex_buffer, 0, rectangle_vertices);
        queue.write_buffer(&self.rectangle_index_buffer, 0, rectangle_indices);
        queue.write_buffer(&self.shadow_vertex_buffer, 0, shadow_vertices);
        queue.write_buffer(&self.shadow_index_buffer, 0, shadow_indices);
        queue.write_buffer(&self.glyph_vertex_buffer, 0, glyph_vertices);
        queue.write_buffer(&self.glyph_index_buffer, 0, glyph_indices);
        queue.write_buffer(&self.blur_vertex_buffer, 0, blur_vertices);
        queue.write_buffer(&self.blur_index_buffer, 0, blur_indices);
    }

    pub fn render_rectangles<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.rectangle_pipeline);
        render_pass.set_bind_group(0, &self.pong_bind_group, &[]);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[
                self.surface_size[0],
                self.surface_size[1],
                self.texture_size[0],
                self.texture_size[1],
            ]),
        );
        render_pass.set_vertex_buffer(0, self.rectangle_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.rectangle_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.rectangle_indices.len() as u32, 0, 0..1);
    }

    pub fn render_shadows<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.shadow_pipeline);
        render_pass.set_bind_group(0, &self.pong_bind_group, &[]);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[
                self.surface_size[0],
                self.surface_size[1],
                self.texture_size[0],
                self.texture_size[1],
            ]),
        );
        render_pass.set_vertex_buffer(0, self.shadow_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.shadow_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.shadow_indices.len() as u32, 0, 0..1);
    }

    pub fn blur_ping<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.blur_ping_pipeline);
        render_pass.set_bind_group(0, &self.ping_bind_group, &[]);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[
                self.surface_size[0],
                self.surface_size[1],
                self.texture_size[0],
                self.texture_size[1],
            ]),
        );
        render_pass.set_vertex_buffer(0, self.blur_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.blur_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.blur_indices.len() as u32, 0, 0..1);
    }

    pub fn blur_pong<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.blur_pong_pipeline);
        render_pass.set_bind_group(0, &self.pong_bind_group, &[]);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[
                self.surface_size[0],
                self.surface_size[1],
                self.texture_size[0],
                self.texture_size[1],
            ]),
        );
        render_pass.set_vertex_buffer(0, self.blur_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.blur_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.blur_indices.len() as u32, 0, 0..1);
    }

    pub fn render_glyphs<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.glyph_pipeline);
        render_pass.set_bind_group(0, &self.pong_bind_group, &[]);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[
                self.surface_size[0],
                self.surface_size[1],
                self.texture_size[0],
                self.texture_size[1],
            ]),
        );
        render_pass.set_vertex_buffer(0, self.glyph_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.glyph_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.glyph_indices.len() as u32, 0, 0..1);
    }

    pub fn post_render(&mut self) {
        self.rectangle_vertices.clear();
        self.rectangle_indices.clear();
        self.shadow_vertices.clear();
        self.shadow_indices.clear();
        self.glyph_vertices.clear();
        self.glyph_indices.clear();
        self.blur_vertices.clear();
        self.blur_indices.clear();

        self.mask_atlas.next_frame();
        self.color_atlas.next_frame();
        self.animated_atlas.next_frame();
        self.blur_atlas.clear();
    }
}

/// Private.
impl TextPipeline {
    fn insert_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        [top, left]: [i32; 2],
        font_size: FontSize,
        line_height: LineHeight,
        time: Duration,
        glyph: &Glyph,
    ) -> Option<(u32, ([i32; 2], [u32; 2]), [u32; 2])> {
        if glyph.is_animated() {
            self.insert_animated_glyph(queue, scaler, glyph, time).map(
                |(glyph_type, placement, [u, v])| {
                    // Animated glyph has screen coordinate system, from top of line
                    let center =
                        ((line_height as f32 - placement.height as f32) / 2.0).round() as i32;
                    (
                        glyph_type,
                        (
                            [top - placement.top + center, left + placement.left],
                            [placement.width, placement.height],
                        ),
                        [u, v],
                    )
                },
            )
        } else {
            self.insert_non_animated_glyph(queue, scaler, glyph).map(
                |(glyph_type, placement, [u, v])| {
                    // Swash image placement has vertical up, from baseline
                    (
                        glyph_type,
                        (
                            [
                                top + font_size as i32 - placement.top,
                                left + placement.left,
                            ],
                            [placement.width, placement.height],
                        ),
                        [u, v],
                    )
                },
            )
        }
    }

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
        let ([u, v], _) = {
            atlas.insert(key, placement, [width, height]).unwrap();
            atlas.get(&key).unwrap()
        };

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
            let ([u, v], _) = {
                let key = (glyph.size, id, index as FrameIndex);
                self.animated_atlas
                    .insert(key, placement, [width, height])
                    .unwrap();
                self.animated_atlas.get(&key).unwrap()
            };

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

    fn push_quad<T: Copy>(
        (vertices, indices): (&mut Vec<T>, &mut Vec<Index>),
        [top_left, top_right, bottom_left, bottom_right]: [T; 4],
    ) {
        let i = vertices.len() as u32;

        vertices.extend_from_slice(&[top_left, top_right, bottom_left, bottom_right]);

        let top_left = i;
        let top_right = i + 1;
        let bottom_left = i + 2;
        let bottom_right = i + 3;

        indices.extend_from_slice(&[
            top_left,
            bottom_right,
            top_right,
            top_left,
            bottom_left,
            bottom_right,
        ]);
    }
}
