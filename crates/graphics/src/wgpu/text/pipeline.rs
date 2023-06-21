use super::*;

type Sizes = [[u32; 2]; 2];

#[derive(Debug)]
struct Pass {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    output: Texture,
    pipeline: Option<RenderPipeline>,
}

impl Pass {
    fn new(output: Texture) -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1024),
            output,
            pipeline: None,
        }
    }

    fn vertices(&self) -> BufferAddress {
        (self.vertices.len() * size_of::<Vertex>()) as BufferAddress
    }

    fn indices(&self) -> BufferAddress {
        (self.indices.len() * size_of::<u32>()) as BufferAddress
    }

    fn is_empty(&self) -> bool {
        debug_assert!(!(self.vertices.is_empty() ^ self.indices.is_empty()));
        self.indices.is_empty()
    }

    fn len(&self) -> u32 {
        self.indices.len() as u32
    }

    fn insert_quad(&mut self, [top_left, top_right, bottom_left, bottom_right]: [Vertex; 4]) {
        let i = self.vertices.len() as u32;

        self.vertices
            .extend_from_slice(&[top_left, top_right, bottom_left, bottom_right]);

        let top_left = i;
        let top_right = i + 1;
        let bottom_left = i + 2;
        let bottom_right = i + 3;

        self.indices.extend_from_slice(&[
            top_left,
            bottom_right,
            top_right,
            top_left,
            bottom_left,
            bottom_right,
        ]);
    }

    fn resize(&mut self, output: Texture) {
        self.output.destroy();
        self.output = output;
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPipeline                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Text pipeline.
#[derive(Debug)]
pub struct TextPipeline {
    sizes: Sizes,
    size_uniform: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    mask_atlas: Atlas<GlyphKey, Placement>,
    color_atlas: Atlas<GlyphKey, Placement>,
    animated_atlas: Atlas<AnimatedGlyphKey, Placement>,
    mask_texture: Texture,
    color_texture: Texture,
    animated_texture: Texture,
    rectangle: Pass,
    blur: Pass,
    glyph: Pass,
    pass_bind_group: BindGroup,
    compose_bind_group_layout: BindGroupLayout,
    compose_bind_group: BindGroup,
    compose_pipeline: RenderPipeline,
}

impl TextPipeline {
    pub const ALTAS_ROW_HEIGHT: u32 = 400;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let max_buffer_size = limits.max_buffer_size;
        let max_texture_dimension = limits.max_texture_dimension_2d;

        //
        // Buffers
        //

        let size_uniform = device.create_buffer(&buffer! {
            label: "[TextPipeline] Size uniform",
            size: size_of::<Sizes>(),
            usage: UNIFORM | COPY_DST,
        });
        let vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Vertex buffer",
            size: max_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Index buffer",
            size: max_buffer_size,
            usage: INDEX | COPY_DST,
        });

        //
        // Atlases and textures
        //

        let mut mask_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            max_texture_dimension,
            max_texture_dimension,
        );
        let mut color_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            max_texture_dimension,
            max_texture_dimension,
        );
        let mut animated_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            max_texture_dimension,
            max_texture_dimension,
        );
        mask_atlas.next_frame();
        color_atlas.next_frame();
        animated_atlas.next_frame();

        let mask_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Mask glyphs texture",
            size: [max_texture_dimension, max_texture_dimension],
            format: TextureFormat::R8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let color_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Color glyphs texture",
            size: [max_texture_dimension, max_texture_dimension],
            format: TextureFormat::Rgba8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let animated_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Animated glyphs texture",
            size: [max_texture_dimension, max_texture_dimension],
            format: TextureFormat::Rgba8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });

        //
        // Passes
        //

        let [rectangle, blur, glyph] = Self::pass_textures(device, config);
        let mut rectangle = Pass::new(rectangle);
        let mut blur = Pass::new(blur);
        let mut glyph = Pass::new(glyph);

        //
        // Bind groups
        //

        let pass_bind_group_layout = device.create_bind_group_layout(&bind_group_layout! {
            label: "[TextPipeline] Pass bind group layout",
            entries: [
                // Size uniform
                { binding: 0, visibility: VERTEX, ty: Uniform },
                // Mask texture
                { binding: 1, visibility: FRAGMENT, ty: Texture },
                // Color texture
                { binding: 2, visibility: FRAGMENT, ty: Texture },
                // Animated texture
                { binding: 3, visibility: FRAGMENT, ty: Texture },
                // Sampler
                { binding: 4, visibility: FRAGMENT, ty: Sampler(Filtering) },
            ],
        });
        let pass_bind_group = device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Pass bind group",
            layout: pass_bind_group_layout,
            entries: [
                // Size uniform
                { binding: 0, resource: Buffer(size_uniform) },
                // Mask texture
                { binding: 1, resource: Texture(mask_texture) },
                // Color texture
                { binding: 2, resource: Texture(color_texture) },
                // Animated texture
                { binding: 3, resource: Texture(animated_texture) },
                // Sampler
                { binding: 4, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        });

        let compose_bind_group_layout = device.create_bind_group_layout(&bind_group_layout! {
            label: "[TextPipeline] Compose bind group layout",
            entries: [
                // Rectangle texture
                { binding: 0, visibility: FRAGMENT, ty: Texture },
                // Blur texture
                { binding: 1, visibility: FRAGMENT, ty: Texture },
                // Glyph texture
                { binding: 2, visibility: FRAGMENT, ty: Texture },
                // Sampler
                { binding: 3, visibility: FRAGMENT, ty: Sampler(Filtering) },
            ],
        });
        let compose_bind_group = Self::compose_bind_group(
            device,
            &compose_bind_group_layout,
            &rectangle,
            &blur,
            &glyph,
        );

        //
        // Pipelines
        //

        let targets = [Some(ColorTargetState {
            format: config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        {
            let layout = device.create_pipeline_layout(&pipeline_layout! {
                label: "[TextPipeline] Pass pipeline layout",
                bind_group_layouts: [pass_bind_group_layout],
            });
            let module = device.create_shader_module(include_wgsl!("shaders/text.wgsl"));

            rectangle.pipeline = Some(device.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Rectangle pass pipeline",
                layout: layout,
                module: module,
                vertex: "vertex",
                buffers: [Vertex::vertex_buffer_layout()],
                fragment: "rectangle_fragment",
                targets: targets,
                topology: TriangleList,
            }));
            blur.pipeline = Some(device.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Blur pass pipeline",
                layout: layout,
                module: module,
                vertex: "vertex",
                buffers: [Vertex::vertex_buffer_layout()],
                fragment: "blur_fragment",
                targets: targets,
                topology: TriangleList,
            }));
            glyph.pipeline = Some(device.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Glyph pass pipeline",
                layout: layout,
                module: module,
                vertex: "vertex",
                buffers: [Vertex::vertex_buffer_layout()],
                fragment: "glyph_fragment",
                targets: targets,
                topology: TriangleList,
            }));
        };

        let compose_pipeline = {
            let module = &device.create_shader_module(include_wgsl!("shaders/compose.wgsl"));
            let layout = &device.create_pipeline_layout(&pipeline_layout! {
                label: "[TextPipeline] Compose pipeline layout",
                bind_group_layouts: [compose_bind_group_layout],
            });

            device.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Compose pipeline",
                layout: layout,
                module: module,
                vertex: "vertex",
                buffers: [],
                fragment: "fragment",
                targets: targets,
                topology: TriangleList,
            })
        };

        Self {
            sizes: [
                [config.width, config.height],
                [max_texture_dimension, max_texture_dimension],
            ],
            size_uniform,
            vertex_buffer,
            index_buffer,
            mask_atlas,
            color_atlas,
            animated_atlas,
            mask_texture,
            color_texture,
            animated_texture,
            rectangle,
            blur,
            glyph,
            pass_bind_group,
            compose_bind_group_layout,
            compose_bind_group,
            compose_pipeline,
        }
    }

    pub fn rectangle_texture_view(&self) -> TextureView {
        self.rectangle.output.create_view(&Default::default())
    }

    pub fn blur_texture_view(&self) -> TextureView {
        self.blur.output.create_view(&Default::default())
    }

    pub fn glyph_texture_view(&self) -> TextureView {
        self.glyph.output.create_view(&Default::default())
    }

    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.sizes[0] = [config.width, config.height];

        let [rectangle, blur, glyph] = Self::pass_textures(device, config);
        self.rectangle.resize(rectangle);
        self.blur.resize(blur);
        self.glyph.resize(glyph);

        self.compose_bind_group = Self::compose_bind_group(
            device,
            &self.compose_bind_group_layout,
            &self.rectangle,
            &self.blur,
            &self.glyph,
        );
    }

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        self.rectangle.insert_quad(Vertex::quad(
            Vertex::BACKGROUND_RECTANGLE,
            ([region_top, region_left], [region_width, region_height]),
            ([top, left], [width, height]),
            [0, 0],
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

        // Discard when outside region. This suppposes that:
        // - glyphs are not bigger that line height (~ font size < line height)
        // - glyphs outside do not affect what's inside (~ no blur)
        // - no further transforms are applied in the shader
        // Of course the GPU would have done that for us. Don't fear to remove if necessary.
        {
            let above = top + (line_height as i32) < 0;
            let below = top >= region_height as i32;

            if above || below {
                return;
            }
        }

        //
        // Add backgrounds
        //

        for (Range { start, end }, background) in line.backgrounds() {
            if background.a != 0 {
                let left = left + start as i32;
                let width = (end - start) as u32;

                self.rectangle.insert_quad(Vertex::quad(
                    Vertex::BACKGROUND_RECTANGLE,
                    region,
                    ([top, left], [width, line_height]),
                    [0, 0],
                    background,
                ));
            }
        }

        //
        // Add glyphs
        //

        let mut scaler = line.scaler(context);

        for glyph in line.glyphs() {
            let (ty, ([top, left], [width, height]), [u, v]) = if glyph.is_animated() {
                if let Some((ty, [u, v], placement)) =
                    self.insert_animated_glyph(queue, &mut scaler, glyph, time)
                {
                    debug_assert!(placement.top == 0 && placement.left == 0);

                    // Centering vertically by hand
                    let top =
                        top + ((line_height as f32 - placement.width as f32) / 2.0).round() as i32;
                    let left = left + glyph.offset.round() as i32;
                    let width = placement.width;
                    let height = placement.height;

                    (ty, ([top, left], [width, height]), [u, v])
                } else {
                    continue;
                }
            } else {
                if let Some((ty, [u, v], placement)) = self.insert_glyph(queue, &mut scaler, glyph)
                {
                    // Swash image has placement (vertical up from baseline)
                    let top = top + line.size() as i32 - placement.top;
                    let left = left + glyph.offset.round() as i32 + placement.left;
                    let width = placement.width;
                    let height = placement.height;

                    (ty, ([top, left], [width, height]), [u, v])
                } else {
                    continue;
                }
            };

            self.glyph.insert_quad(Vertex::quad(
                ty,
                region,
                ([top, left], [width, height]),
                [u, v],
                glyph.styles.foreground,
            ));

            if let Some(blur) = glyph
                .styles
                .blur
                .filter(|blur| blur.radius > 0 && blur.color.a != 0)
            {
                self.blur.insert_quad(Vertex::quad(
                    ty,
                    region,
                    ([top, left], [width, height]),
                    [u, v],
                    blur.color,
                ));
            }
        }
    }

    pub fn pre_render(&mut self, queue: &Queue) {
        let vertices = self
            .rectangle
            .vertices
            .iter()
            .chain(&self.blur.vertices)
            .chain(&self.glyph.vertices)
            .copied()
            .collect::<Vec<_>>();
        let indices = self
            .rectangle
            .indices
            .iter()
            .chain(&self.blur.indices)
            .chain(&self.glyph.indices)
            .copied()
            .collect::<Vec<_>>();

        queue.write_buffer(&self.size_uniform, 0, bytemuck::cast_slice(&self.sizes));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
    }

    pub fn render_rectangles<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>) {
        let vertices = ..self.rectangle.vertices();
        let indices = ..self.rectangle.indices();

        if !self.rectangle.is_empty() {
            render_pass.set_pipeline(self.rectangle.pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, &self.pass_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(vertices));
            render_pass.set_index_buffer(self.index_buffer.slice(indices), IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.rectangle.len(), 0, 0..1);
        }
    }

    pub fn render_blurs<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>) {
        let vertices = self.rectangle.vertices()..self.rectangle.vertices() + self.blur.vertices();
        let indices = self.rectangle.indices()..self.rectangle.indices() + self.blur.indices();

        if !self.blur.is_empty() {
            render_pass.set_pipeline(self.blur.pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, &self.pass_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(vertices));
            render_pass.set_index_buffer(self.index_buffer.slice(indices), IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.blur.len(), 0, 0..1);
        }
    }

    pub fn render_glyphs<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>) {
        let vertices = self.rectangle.vertices() + self.blur.vertices()..;
        let indices = self.rectangle.indices() + self.blur.indices()..;

        if !self.glyph.is_empty() {
            render_pass.set_pipeline(self.glyph.pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, &self.pass_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(vertices));
            render_pass.set_index_buffer(self.index_buffer.slice(indices), IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.glyph.len(), 0, 0..1);
        }
    }

    pub fn compose<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.compose_pipeline);
        render_pass.set_bind_group(0, &self.compose_bind_group, &[]);
        render_pass.draw(0..6, 0..3);
    }

    pub fn post_render(&mut self) {
        self.rectangle.clear();
        self.blur.clear();
        self.glyph.clear();

        self.mask_atlas.next_frame();
        self.color_atlas.next_frame();
        self.animated_atlas.next_frame();
    }
}

/// Private.
impl TextPipeline {
    fn pass_textures(device: &Device, config: &SurfaceConfiguration) -> [Texture; 3] {
        [
            device.create_texture(&texture! {
                label: "[TextPipeline] Rectangle pass output texture",
                size: [config.width, config.height],
                format: config.format,
                usage: TEXTURE_BINDING | RENDER_ATTACHMENT,
            }),
            device.create_texture(&texture! {
                label: "[TextPipeline] Blur pass output texture",
                size: [config.width, config.height],
                format: config.format,
                usage: TEXTURE_BINDING | RENDER_ATTACHMENT,
            }),
            device.create_texture(&texture! {
                label: "[TextPipeline] Glyph pass output texture",
                size: [config.width, config.height],
                format: config.format,
                usage: TEXTURE_BINDING | RENDER_ATTACHMENT,
            }),
        ]
    }

    fn compose_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        rectangle: &Pass,
        blur: &Pass,
        glyph: &Pass,
    ) -> BindGroup {
        device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Compose bind group",
            layout: layout,
            entries: [
                // Rectangle texture
                { binding: 0, resource: Texture(rectangle.output) },
                // Blur texture
                { binding: 1, resource: Texture(blur.output) },
                // Glyph texture
                { binding: 2, resource: Texture(glyph.output) },
                // Sampler
                { binding: 3, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        })
    }

    fn insert_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        glyph: &Glyph,
    ) -> Option<(u32, [u32; 2], Placement)> {
        let key = glyph.key();

        // Check atlases for glyph
        if let Some((ty, ([u, v], placement))) = {
            let in_mask = || self.mask_atlas.get(&key).map(|v| (Vertex::MASK_GLYPH, v));
            let in_color = || self.color_atlas.get(&key).map(|v| (Vertex::COLOR_GLYPH, v));
            in_mask().or_else(in_color)
        } {
            return Some((ty, [u, v], *placement));
        }

        // Render glyph
        let image = scaler.render(&glyph)?;
        let placement = image.placement;
        let [width, height] = [placement.width, placement.height];

        // Allocate glyph in atlas
        let (ty, atlas, texture, channels) = match image.content {
            Content::Mask => (
                Vertex::MASK_GLYPH,
                &mut self.mask_atlas,
                &self.mask_texture,
                1,
            ),
            Content::Color => (
                Vertex::COLOR_GLYPH,
                &mut self.color_atlas,
                &self.color_texture,
                4,
            ),
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

        Some((ty, [u, v], placement))
    }

    fn insert_animated_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        glyph: &Glyph,
        time: Duration,
    ) -> Option<(u32, [u32; 2], Placement)> {
        let id = glyph.animated_id?;
        let key = (glyph.size, id, scaler.frame(glyph, time)?);

        // Check atlas for frame
        if let Some(([u, v], placement)) = self.animated_atlas.get(&key) {
            return Some((Vertex::ANIMATED_GLYPH, [u, v], *placement));
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
        Some((Vertex::ANIMATED_GLYPH, [u, v], *placement))
    }
}
