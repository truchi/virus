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
        [top, left]: [i32; 2],
        [width, height]: [u32; 2],
        ([u, v], [texture_width, texture_height]): ([u32; 2], [u32; 2]),
        color: Rgba,
    ) -> [Self; 4] {
        let region = ([region_top, region_left], [region_width, region_height]);
        let right = left + width as i32;
        let bottom = top + height as i32;
        let u2 = u + texture_width;
        let v2 = v + texture_height;

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
                    resource: BindingResource::Sampler(&device.create_sampler(
                        &SamplerDescriptor {
                            label: Some("[TextPipeline] Sampler descriptor"),
                            address_mode_u: AddressMode::ClampToEdge,
                            address_mode_v: AddressMode::ClampToEdge,
                            address_mode_w: AddressMode::ClampToEdge,
                            mag_filter: FilterMode::Linear,
                            min_filter: FilterMode::Linear,
                            mipmap_filter: FilterMode::Linear,
                            lod_min_clamp: 0.0,
                            lod_max_clamp: 0.0,
                            compare: None,
                            anisotropy_clamp: 1,
                            border_color: None,
                        },
                    )),
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

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        self.insert_quad(Vertex::quad(
            Vertex::BACKGROUND_RECTANGLE,
            ([region_top, region_left], [region_width, region_height]),
            [top, left],
            [width, height],
            ([0, 0], [0, 0]),
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
                    [top, left],
                    [width, line_height],
                    ([0, 0], [0, 0]),
                    background,
                ));
            }
        }

        //
        // Add glyphs
        //

        let mut scaler = line.scaler(context);

        while let Some((glyph, image)) = scaler.next() {
            let image = if let Some(image) = image {
                image
            } else {
                continue;
            };

            let key = (glyph.font, glyph.id, line.size());
            let width = image.placement.width;
            let height = image.placement.height;

            // Allocate glyph in atlas
            let (ty, ([u, v], is_new), texture, channels) = match image.content {
                Content::Mask => (
                    Vertex::MASK_GLYPH,
                    self.mask_atlas.insert(key, [width, height]).unwrap(),
                    &self.mask_texture,
                    1,
                ),
                Content::Color => (
                    Vertex::COLOR_GLYPH,
                    self.color_atlas.insert(key, [4 * width, height]).unwrap(),
                    &self.color_texture,
                    4,
                ),
                Content::SubpixelMask => unreachable!(),
            };

            // Insert glyph in atlas
            if is_new {
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

            // Scale emoji glyphs
            let scaled = if let Some(advance) = glyph.emoji {
                let scale = glyph.advance / advance;

                swash::zeno::Placement {
                    top: (scale * image.placement.top as f32).round() as i32,
                    left: (scale * image.placement.left as f32).round() as i32,
                    width: (scale * image.placement.width as f32).round() as u32,
                    height: (scale * image.placement.height as f32).round() as u32,
                }
            } else {
                image.placement
            };

            // Swash image has placement (vertical up from baseline)
            let top = top + line.size() as i32 - scaled.top;
            let left = left + glyph.offset.round() as i32 + scaled.left;

            self.insert_quad(Vertex::quad(
                ty,
                region,
                [top, left],
                [scaled.width, scaled.height],
                ([u, v], [image.placement.width, image.placement.height]),
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
