use super::*;

const MASK_ATLAS_BIN_WIDTH: u32 = 400;
const COLOR_ATLAS_BIN_WIDTH: u32 = 400;
const MASK_ATLAS_SURFACE_FACTOR: u32 = 2;
const COLOR_ATLAS_SURFACE_FACTOR: u32 = 2;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Type                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Type => Uint32);

/// Type: [`Type::RECTANGLE`]/[`Type::MASK`]/[`Type::COLOR`].
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Type(u32);

impl Type {
    pub const RECTANGLE: Self = Self(0);
    pub const MASK: Self = Self(1);
    pub const COLOR: Self = Self(2);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Instance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Instance => Instance: [Type, Position, Size, Position, Rgba]);

/// Instance.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Instance {
    ty: Type,
    position: Position,
    size: Size,
    uv: Position,
    color: Rgba,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Init                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Inits the `Pipeline`.
struct Init<'a>(&'a Device);

impl<'a> Init<'a> {
    fn buffer(&self, size: BufferAddress) -> Buffer {
        self.0.create_buffer(&BufferDescriptor {
            label: Some("Instance buffer"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn mask_texture(&self, max_texture_dimension: u32, config: &SurfaceConfiguration) -> Texture {
        self.0.create_texture(&TextureDescriptor {
            label: Some("Mask texture"),
            size: Extent3d {
                width: max_texture_dimension.min(MASK_ATLAS_SURFACE_FACTOR * config.width),
                height: max_texture_dimension.min(MASK_ATLAS_SURFACE_FACTOR * config.height),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    fn color_texture(&self, max_texture_dimension: u32, config: &SurfaceConfiguration) -> Texture {
        self.0.create_texture(&TextureDescriptor {
            label: Some("Color texture"),
            size: Extent3d {
                width: max_texture_dimension.min(COLOR_ATLAS_SURFACE_FACTOR * config.width),
                height: max_texture_dimension.min(COLOR_ATLAS_SURFACE_FACTOR * config.height),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    fn bind_group_layout(&self) -> BindGroupLayout {
        self.0.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bind group layout"),
            entries: &[
                // Mask texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Color texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        })
    }

    fn bind_group(
        &self,
        bind_group_layout: &BindGroupLayout,
        mask: &Texture,
        color: &Texture,
    ) -> BindGroup {
        self.0.create_bind_group(&BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &bind_group_layout,
            entries: &[
                // Mask texture
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&mask.create_view(&Default::default())),
                },
                // Color texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&color.create_view(&Default::default())),
                },
                // Sampler
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.0.create_sampler(&Default::default())),
                },
            ],
        })
    }

    fn pipeline(
        &self,
        config: &SurfaceConfiguration,
        bind_group_layout: &BindGroupLayout,
        module: &ShaderModule,
    ) -> RenderPipeline {
        let pipeline_layout = self.0.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stages: Constants::STAGES,
                range: 0..Constants::size(),
            }],
        });

        self.0.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[Instance::buffer_layout()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Pipeline                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Pipeline.
#[derive(Debug)]
pub struct Pipeline {
    constants: Constants,
    instances: u32,
    bytes: Range<BufferAddress>,
    buffer: Buffer,
    mask: Atlas<GlyphKey, Placement>,
    color: Atlas<GlyphKey, Placement>,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl Pipeline {
    /// Creates a new `Pipeline` for `device` and `config`.
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let max_buffer_size = limits.max_buffer_size;
        let max_texture_dimension = limits.max_texture_dimension_2d;

        let constants = Constants {
            surface: [config.width as f32, config.height as f32],
        };
        let buffer = Init(device).buffer(max_buffer_size); // TODO limit size
        let mask = Atlas::new(
            Init(device).mask_texture(max_texture_dimension, config),
            MASK_ATLAS_BIN_WIDTH,
        );
        let color = Atlas::new(
            Init(device).color_texture(max_texture_dimension, config),
            COLOR_ATLAS_BIN_WIDTH,
        );
        let bind_group_layout = Init(device).bind_group_layout();
        let bind_group =
            Init(device).bind_group(&bind_group_layout, mask.texture(), color.texture());
        let pipeline = Init(device).pipeline(
            config,
            &bind_group_layout,
            &device.create_shader_module(include_wgsl!("./shader.wgsl")),
        );

        Self {
            constants,
            instances: 0,
            bytes: 0..0,
            buffer,
            mask,
            color,
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }

    /// Resizes the `Pipeline`.
    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        let max_texture_dimension = device.limits().max_texture_dimension_2d;

        self.constants.resize(config);
        self.mask.clear_and_resize(
            Init(device).mask_texture(max_texture_dimension, config),
            MASK_ATLAS_BIN_WIDTH,
        );
        self.color.clear_and_resize(
            Init(device).color_texture(max_texture_dimension, config),
            COLOR_ATLAS_BIN_WIDTH,
        );
        self.bind_group = Init(device).bind_group(
            &self.bind_group_layout,
            self.mask.texture(),
            self.color.texture(),
        );
    }

    /// Writes a `rectangle` to be rendered in `region` with `color`.
    pub fn write_rectangle(
        &mut self,
        queue: &Queue,
        region: Rectangle,
        rectangle: Rectangle,
        color: Rgba,
    ) {
        if !color.is_visible() {
            return;
        }

        let Some(rectangle) = rectangle.intersection_in(region) else {
            return;
        };

        let instance = &[Instance {
            ty: Type::RECTANGLE,
            position: rectangle.position(),
            size: rectangle.size(),
            uv: Position::default(),
            color,
        }];
        let bytes = bytemuck::cast_slice::<_, u8>(instance);
        let len = NonZeroU64::new(bytes.len() as BufferAddress).unwrap();

        queue
            .write_buffer_with(&self.buffer, self.bytes.end, len)
            .expect("large enough buffer")
            .clone_from_slice(bytes);

        self.instances += 1;
        self.bytes.end += len.get();
    }

    /// Writes a glyph `key` to be rendered in `region` with `position` and `color`.
    ///
    /// Image data will be obtained through `ìmage` and only called if not in atlas already.
    pub fn write_glyph<F: FnOnce() -> Image>(
        &mut self,
        queue: &Queue,
        region: Rectangle,
        position: Position,
        key: GlyphKey,
        color: Rgba,
        image: F,
    ) {
        // Early return for invisible glyphs
        if !color.is_visible()
            || 0 <= position.top && region.height() <= position.top as u32
            || 0 <= position.left && region.width() <= position.left as u32
        {
            return;
        }

        // Get or insert glyph in atlas
        let (ty, uv, placement) = if let Some((ty, uv, placement)) = {
            let in_mask = || {
                self.mask
                    .get(&key)
                    .map(|item| (Type::MASK, item.position, item.value))
            };
            let in_color = || {
                self.color
                    .get(&key)
                    .map(|item| (Type::COLOR, item.position, item.value))
            };

            in_mask().or_else(in_color)
        } {
            (ty, uv, placement)
        } else {
            let image = image();
            let (ty, atlas) = match image.content {
                Content::Mask => (Type::MASK, &mut self.mask),
                Content::Color => (Type::COLOR, &mut self.color),
                Content::SubpixelMask => unimplemented!(),
            };

            match atlas.insert(
                queue,
                key,
                image.placement,
                Size::new(image.placement.width, image.placement.height),
                &image.data,
            ) {
                Ok(item) => (ty, item.position, item.value),
                Err(AtlasError::KeyExists) => unreachable!("Just checked this"),
                Err(AtlasError::OutOfSpace) => todo!("Atlas full"),
                Err(AtlasError::WontFit) => {
                    debug_assert!(false, "Glyph does not fit the atlas");
                    return;
                }
            }
        };

        // Crop to region
        let rectangle = Rectangle::new(
            // Swash image placement has vertical upward from baseline
            position.top + key.1 as i32 - placement.top,
            position.left + placement.left,
            placement.width,
            placement.height,
        );
        let uv = uv - Position::new(rectangle.top.min(0), rectangle.left.min(0));

        let Some(rectangle) = rectangle.intersection_in(region) else {
            return;
        };

        let instance = &[Instance {
            ty,
            position: rectangle.position(),
            size: rectangle.size(),
            uv,
            color,
        }];
        let bytes = bytemuck::cast_slice::<_, u8>(instance);
        let len = NonZeroU64::new(bytes.len() as BufferAddress).unwrap();

        queue
            .write_buffer_with(&self.buffer, self.bytes.end, len)
            .expect("large enough buffer")
            .clone_from_slice(bytes);

        self.instances += 1;
        self.bytes.end += len.get();
    }

    /// Draws.
    pub fn draw(&mut self, render_pass: &mut RenderPass) {
        if self.bytes.is_empty() {
            debug_assert!(self.instances == 0);
            return;
        } else {
            debug_assert!(!self.instances != 0);
        }

        let constants = self.constants.as_array();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_push_constants(Constants::STAGES, 0, bytemuck::cast_slice(&constants));
        render_pass.set_vertex_buffer(0, self.buffer.slice(self.bytes.clone()));
        render_pass.draw(0..6, 0..self.instances);

        self.instances = 0;
        self.bytes = 0..0;
    }
}
