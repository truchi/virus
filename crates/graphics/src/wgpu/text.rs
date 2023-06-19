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
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
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

/// Text pipeline.
#[derive(Debug)]
pub struct TextPipeline {
    sizes: Sizes,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    size_uniform: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    mask_atlas: Atlas<GlyphKey, Placement>,
    color_atlas: Atlas<GlyphKey, Placement>,
    animated_atlas: Atlas<AnimatedGlyphKey, Placement>,
    mask_texture: Texture,
    color_texture: Texture,
    animated_texture: Texture,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl TextPipeline {
    pub const ALTAS_ROW_HEIGHT: u32 = 400;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let [width, height] = [
            limits.max_texture_dimension_2d,
            limits.max_texture_dimension_2d,
        ];

        //
        // Buffers
        //

        let size_uniform = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Size uniform"),
            size: std::mem::size_of::<Sizes>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Vertex buffer"),
            size: limits.max_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Index buffer"),
            size: limits.max_buffer_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //
        // Atlases and textures
        //

        let mut mask_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, width, height);
        let mut color_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, width, height);
        let mut animated_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, width, height);
        mask_atlas.next_frame();
        color_atlas.next_frame();
        animated_atlas.next_frame();

        let texture_descriptor = |label, format| TextureDescriptor {
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
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let mask_texture = device.create_texture(&texture_descriptor(
            "[TextPipeline] Mask glyphs texture",
            TextureFormat::R8Unorm,
        ));
        let color_texture = device.create_texture(&texture_descriptor(
            "[TextPipeline] Color glyphs texture",
            TextureFormat::Rgba8Unorm,
        ));
        let animated_texture = device.create_texture(&texture_descriptor(
            "[TextPipeline] Animated glyphs texture",
            TextureFormat::Rgba8Unorm,
        ));

        //
        // Bind group
        //

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("[TextPipeline] Texture bind group layout"),
            entries: &[
                // Size uniform
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Mask texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Color texture
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Animated texture
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Sampler
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[TextPipeline] Texture bind group"),
            layout: &bind_group_layout,
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

        //
        // Pipeline
        //

        let shader_module = device.create_shader_module(include_wgsl!("text.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[TextPipeline] Pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[TextPipeline] Pipeline"),
            layout: Some(&pipeline_layout),
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
        });

        Self {
            sizes: [[config.width, config.height], [width, height]],
            vertices: Vec::with_capacity(1_024),
            indices: Vec::with_capacity(1_024),
            size_uniform,
            vertex_buffer,
            index_buffer,
            mask_atlas,
            color_atlas,
            animated_atlas,
            mask_texture,
            color_texture,
            animated_texture,
            bind_group,
            pipeline,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.sizes[0] = [size.width, size.height];
    }

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        self.insert_quad(Vertex::quad(
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

                self.insert_quad(Vertex::quad(
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

            self.insert_quad(Vertex::quad(
                ty,
                region,
                ([top, left], [width, height]),
                [u, v],
                glyph.styles.foreground,
            ));
        }
    }

    pub fn render<'pass>(&'pass mut self, queue: &Queue, render_pass: &mut RenderPass<'pass>) {
        queue.write_buffer(&self.size_uniform, 0, bytemuck::cast_slice(&self.sizes));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);

        self.vertices.clear();
        self.indices.clear();
        self.mask_atlas.next_frame();
        self.color_atlas.next_frame();
        self.animated_atlas.next_frame();
    }
}

/// Private.
impl TextPipeline {
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
}
