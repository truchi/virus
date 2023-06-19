use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Vertex                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    /// Vertex type:
    /// - 0: a background rectangle (use `color`),
    /// - 1: a mask glyph (use `uv` in the mask texture with `color`),
    /// - 2: a color glyph (use `uv` in the color texture),
    /// - 3: an animated glyph (use `uv` in the animated texture),
    ty: u32,
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

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    const BACKGROUND_RECTANGLE: u32 = 0;
    const MASK_GLYPH: u32 = 1;
    const COLOR_GLYPH: u32 = 2;
    const ANIMATED_GLYPH: u32 = 3;

    const ATTRIBUTES: [VertexAttribute; 6] = vertex_attr_array![
        0 => Uint32,   // ty
        1 => Sint32x2, // region position
        2 => Uint32x2, // region size
        3 => Sint32x2, // position
        4 => Uint32x2, // uv
        5 => Uint32x4, // color
    ];

    fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    fn new(
        ty: u32,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        uv: [u32; 2],
        color: Rgba,
    ) -> Self {
        Self {
            ty,
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

    fn quad(
        ty: u32,
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
            Vertex::new(ty, region, [top, left], [u, v], color),
            Vertex::new(ty, region, [top, right], [u2, v], color),
            Vertex::new(ty, region, [bottom, left], [u, v2], color),
            Vertex::new(ty, region, [bottom, right], [u2, v2], color),
        ]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Pipeline                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type Sizes = [[u32; 2]; 2];

#[derive(Debug)]
struct Pass {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    output: Texture,
}

impl Pass {
    fn new(output: Texture) -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1024),
            output,
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
    pass_pipeline: RenderPipeline,
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

        let size_uniform = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Size uniform"),
            size: size_of::<Sizes>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Vertex buffer"),
            size: max_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Index buffer"),
            size: max_buffer_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
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

        let mask_texture = device.create_texture(&Self::texture_descriptor(
            "[TextPipeline] Mask glyphs texture",
            [max_texture_dimension, max_texture_dimension],
            TextureFormat::R8Unorm,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        ));
        let color_texture = device.create_texture(&Self::texture_descriptor(
            "[TextPipeline] Color glyphs texture",
            [max_texture_dimension, max_texture_dimension],
            TextureFormat::Rgba8Unorm,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        ));
        let animated_texture = device.create_texture(&Self::texture_descriptor(
            "[TextPipeline] Animated glyphs texture",
            [max_texture_dimension, max_texture_dimension],
            TextureFormat::Rgba8Unorm,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        ));

        //
        // Passes
        //

        let [rectangle, blur, glyph] = Self::output_texture_descriptors(config);
        let rectangle = Pass::new(device.create_texture(&rectangle));
        let blur = Pass::new(device.create_texture(&blur));
        let glyph = Pass::new(device.create_texture(&glyph));

        //
        // Bind groups
        //

        let uniform_entry = |binding| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture_entry = |binding| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        };
        let sampler_entry = |binding| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };

        let pass_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("[TextPipeline] Pass bind group layout"),
            entries: &[
                // Size uniform
                uniform_entry(0),
                // Mask texture
                texture_entry(1),
                // Color texture
                texture_entry(2),
                // Animated texture
                texture_entry(3),
                // Sampler
                sampler_entry(4),
            ],
        });
        let pass_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[TextPipeline] Pass bind group"),
            layout: &pass_bind_group_layout,
            entries: &[
                // Size uniform
                BindGroupEntry {
                    binding: 0,
                    resource: size_uniform.as_entire_binding(),
                },
                // Mask texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &mask_texture.create_view(&Default::default()),
                    ),
                },
                // Color texture
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &color_texture.create_view(&Default::default()),
                    ),
                },
                // Animated texture
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(
                        &animated_texture.create_view(&Default::default()),
                    ),
                },
                // Sampler
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&device.create_sampler(&Default::default())),
                },
            ],
        });

        let compose_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("[TextPipeline] Compose bind group layout"),
                entries: &[
                    // Rectangle texture
                    texture_entry(0),
                    // Blur texture
                    texture_entry(1),
                    // Glyph texture
                    texture_entry(2),
                    // Sampler
                    sampler_entry(3),
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

        let pass_pipeline = {
            let shader_module = device.create_shader_module(include_wgsl!("text.wgsl"));
            let pass_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("[TextPipeline] Pass pipeline layout"),
                bind_group_layouts: &[&pass_bind_group_layout],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("[TextPipeline] Pass pipeline"),
                layout: Some(&pass_pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vertex",
                    buffers: &[Vertex::vertex_buffer_layout()],
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: "fragment",
                    targets: &[Some(ColorTargetState {
                        format: config.format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
            })
        };

        let compose_pipeline = {
            let shader_module = device.create_shader_module(include_wgsl!("compose.wgsl"));
            let compose_pipeline_layout =
                device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("[TextPipeline] Compose pipeline layout"),
                    bind_group_layouts: &[&compose_bind_group_layout],
                    push_constant_ranges: &[],
                });

            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("[TextPipeline] Compose pipeline"),
                layout: Some(&compose_pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vertex",
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: "fragment",
                    targets: &[Some(ColorTargetState {
                        format: config.format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
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
            pass_pipeline,
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

        let [rectangle, blur, glyph] = Self::output_texture_descriptors(config);
        self.rectangle.resize(device.create_texture(&rectangle));
        self.blur.resize(device.create_texture(&blur));
        self.glyph.resize(device.create_texture(&glyph));

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

            if let Some(_blur) = glyph
                .styles
                .blur
                .map(|blur| blur.radius > 0 && blur.color.a != 0)
            {
                self.blur.insert_quad(Vertex::quad(
                    ty,
                    region,
                    ([top, left], [width, height]),
                    [u, v],
                    glyph.styles.foreground,
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
            render_pass.set_pipeline(&self.pass_pipeline);
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
            render_pass.set_pipeline(&self.pass_pipeline);
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
            render_pass.set_pipeline(&self.pass_pipeline);
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
    fn texture_descriptor(
        label: &str,
        [width, height]: [u32; 2],
        format: TextureFormat,
        usage: TextureUsages,
    ) -> TextureDescriptor {
        TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        }
    }

    fn output_texture_descriptors(config: &SurfaceConfiguration) -> [TextureDescriptor; 3] {
        [
            Self::texture_descriptor(
                "[TextPipeline] Rectangle pass output texture",
                [config.width, config.height],
                config.format,
                TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            ),
            Self::texture_descriptor(
                "[TextPipeline] Blur pass output texture",
                [config.width, config.height],
                config.format,
                TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            ),
            Self::texture_descriptor(
                "[TextPipeline] Glyph pass output texture",
                [config.width, config.height],
                config.format,
                TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            ),
        ]
    }

    fn compose_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        rectangle: &Pass,
        blur: &Pass,
        glyph: &Pass,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("[TextPipeline] Compose bind group"),
            layout,
            entries: &[
                // Rectangle texture
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &rectangle.output.create_view(&Default::default()),
                    ),
                },
                // Blur texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &blur.output.create_view(&Default::default()),
                    ),
                },
                // Glyph texture
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &glyph.output.create_view(&Default::default()),
                    ),
                },
                // Sampler
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&device.create_sampler(&Default::default())),
                },
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
