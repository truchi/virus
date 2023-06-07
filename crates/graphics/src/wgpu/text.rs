use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Vertex                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    /// Vertex type:
    /// - 0: a background rectangle (use `color`),
    /// - 1: a mask glyph (use `texture` in the mask texture with `color`),
    /// - 2: a color glyph (use `texture` in the color texture),
    ty: u32,
    /// World coordinates.
    position: [i32; 3],
    /// World/Texture size.
    size: [u32; 2],
    /// Texture coordinates.
    texture: [i32; 2],
    /// Glyph position.
    glyph: [u32; 2],
    /// Rgba color.
    color: [u32; 4],
    /// Blur radius.
    blur_radius: u32,
    /// Blur color.
    blur_color: [u32; 3],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    const BACKGROUND_RECTANGLE_TYPE: u32 = 0;
    const MASK_GLYPH_TYPE: u32 = 1;
    const COLOR_GLYPH_TYPE: u32 = 2;
    const ATTRIBUTES: [VertexAttribute; 8] = vertex_attr_array![
        0 => Uint32,   // ty
        1 => Sint32x3, // position
        2 => Uint32x2, // size
        3 => Sint32x2, // texture
        4 => Uint32x2, // glyph
        5 => Uint32x4, // color
        6 => Uint32,   // blur_radius
        7 => Uint32x3, // blur_color
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
        position: [i32; 3],
        size: [u32; 2],
        texture: [i32; 2],
        glyph: [u32; 2],
        color: Rgba,
        blur_radius: u32,
    ) -> Self {
        Self {
            ty,
            position,
            size,
            texture,
            glyph,
            color: [
                color.r as u32,
                color.g as u32,
                color.b as u32,
                color.a as u32,
            ],
            blur_radius,
            blur_color: [45, 70, 77],
        }
    }

    fn quad(
        ty: u32,
        [top, left, depth]: [i32; 3],
        [width, height]: [u32; 2],
        [u, v]: [u32; 2],
        color: Rgba,
        blur_radius: u32,
    ) -> [Self; 4] {
        let size = [width, height];
        let glyph = [u, v];

        let top = top - blur_radius as i32;
        let left = left - blur_radius as i32;
        let width = (width + 2 * blur_radius) as i32;
        let height = (height + 2 * blur_radius) as i32;
        let u = u as i32 - blur_radius as i32;
        let v = v as i32 - blur_radius as i32;

        let right = left + width;
        let bottom = top + height;
        let u2 = u + width;
        let v2 = v + height;

        [
            Vertex::new(
                ty,
                [left, top, depth],
                size,
                [u, v],
                glyph,
                color,
                blur_radius,
            ),
            Vertex::new(
                ty,
                [right, top, depth],
                size,
                [u2, v],
                glyph,
                color,
                blur_radius,
            ),
            Vertex::new(
                ty,
                [left, bottom, depth],
                size,
                [u, v2],
                glyph,
                color,
                blur_radius,
            ),
            Vertex::new(
                ty,
                [right, bottom, depth],
                size,
                [u2, v2],
                glyph,
                color,
                blur_radius,
            ),
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
    mask_atlas: Atlas<(FontKey, GlyphId, FontSize)>,
    color_atlas: Atlas<(FontKey, GlyphId, FontSize)>,
    mask_texture: Texture,
    color_texture: Texture,
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
        mask_atlas.next_frame();
        color_atlas.next_frame();

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

        //
        // Bind group
        //

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("[TextPipeline] Texture bind group layout"),
            entries: &[
                // Size uniform
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
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
                // Sampler
                BindGroupLayoutEntry {
                    binding: 3,
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
                // Sampler
                BindGroupEntry {
                    binding: 3,
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
            mask_texture,
            color_texture,
            bind_group,
            pipeline,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.sizes[0] = [size.width, size.height];
    }

    pub fn insert(
        &mut self,
        queue: &Queue,
        context: &mut Context,
        top: i32,
        left: i32,
        depth: i32,
        line: &Line,
        line_height: LineHeight,
    ) {
        //
        // Add backgrounds
        //

        for (Range { start, end }, background) in line.backgrounds() {
            if background.a != 0 {
                self.insert_quad(Vertex::quad(
                    Vertex::BACKGROUND_RECTANGLE_TYPE,
                    [top, left + start as i32, depth],
                    [(end - start) as u32, line_height],
                    Default::default(),
                    background,
                    Default::default(),
                ));
            }
        }

        //
        // Add glyphs
        //

        let mut scaler = line.scaler(context);

        while let Some((advance, glyph, image)) = scaler.next() {
            let image = if let Some(image) = image {
                image
            } else {
                continue;
            };

            let top = top + line.size() as i32;
            let left = left + advance as i32;
            let key = (glyph.styles.font, glyph.id, line.size());

            // Swash image has placement
            let top = top - image.placement.top;
            let left = left + image.placement.left;
            let width = image.placement.width;
            let height = image.placement.height;

            let (ty, ([x, y], is_new), texture, channels) = match image.content {
                Content::Mask => (
                    Vertex::MASK_GLYPH_TYPE,
                    self.mask_atlas.insert(key, [width, height]).unwrap(),
                    &self.mask_texture,
                    1,
                ),
                Content::Color => (
                    Vertex::COLOR_GLYPH_TYPE,
                    self.color_atlas.insert(key, [4 * width, height]).unwrap(),
                    &self.color_texture,
                    4,
                ),
                Content::SubpixelMask => unreachable!(),
            };

            if is_new {
                queue.write_texture(
                    ImageCopyTexture {
                        texture,
                        mip_level: 0,
                        origin: Origin3d { x, y, z: 0 },
                        aspect: TextureAspect::All,
                    },
                    &image.data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(width * channels),
                        rows_per_image: Some(height),
                    },
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            self.insert_quad(Vertex::quad(
                ty,
                [top, left, depth],
                [width, height],
                [x, y],
                glyph.styles.foreground,
                10,
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