use super::*;

macro_rules! label {
    ($label:literal) => {
        Some(concat!("[GlyphPipeline] ", $label))
    };
}

const MASK_ATLAS_BIN_WIDTH: u32 = 400;
const COLOR_ATLAS_BIN_WIDTH: u32 = 400;
const MASK_ATLAS_SURFACE_FACTOR: u32 = 2;
const COLOR_ATLAS_SURFACE_FACTOR: u32 = 2;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Type                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Type => Uint32);

/// Type: [`Type::MASK`]/[`Type::COLOR`].
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Type(u32);

impl Type {
    const MASK: Self = Self(0);
    const COLOR: Self = Self(1);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Instance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Instance => Instance: [Type, Position, Size, Position, Rgba]);

/// Instance.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Instance {
    /// Glyph type.
    ty: Type,
    /// Glyph position.
    position: Position,
    /// Glyph size.
    size: Size,
    /// Glyph uv.
    uv: Position,
    /// Glyph color.
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
            label: label!("Instance buffer"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn mask_texture(&self, max_texture_dimension: u32, config: &SurfaceConfiguration) -> Texture {
        self.0.create_texture(&TextureDescriptor {
            label: label!("Mask texture"),
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
            label: label!("Color texture"),
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
            label: label!("Bind group layout"),
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
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
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
            label: label!("Bind group"),
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
            label: label!("Pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stages: Constants::STAGES,
                range: 0..Constants::size(),
            }],
        });

        self.0.create_render_pipeline(&RenderPipelineDescriptor {
            label: label!("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: "vertex",
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
                entry_point: "fragment",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
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
    layers: BTreeMap<u32, (Vec<Instance>, Range<BufferAddress>)>,
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
        let layers = Default::default();
        let bind_group_layout = Init(device).bind_group_layout();
        let bind_group =
            Init(device).bind_group(&bind_group_layout, mask.texture(), color.texture());
        let pipeline = Init(device).pipeline(
            config,
            &bind_group_layout,
            &device.create_shader_module(include_wgsl!("glyph.wgsl")),
        );

        Self {
            constants,
            layers,
            buffer,
            mask,
            color,
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }

    /// Returns a sorted iterator of layers.
    pub fn layers(&self) -> impl '_ + Iterator<Item = u32> {
        self.layers.keys().copied()
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

    /// Pushes a glyph `key` to be rendered for `layer` in `region`
    /// with `position`, `font_size` and `color`.
    ///
    /// Image data will be obtained through `ìmage` and only called if not in atlas already.
    pub fn push<F: FnOnce() -> Image>(
        &mut self,
        queue: &Queue,
        layer: u32,
        region: Rectangle,
        position: Position,
        font_size: FontSize,
        key: GlyphKey,
        color: Rgba,
        image: F,
    ) {
        // Early return for invisible glyphs
        if !color.is_visible()
            || matches!(u32::try_from(position.top), Ok(top) if region.size().height <= top)
            || matches!(u32::try_from(position.left), Ok(left) if region.size().width <= left)
        {
            return;
        }

        // Get or insert glyph in atlas
        let (ty, uv, placement) = if let Some((ty, uv, placement)) = {
            let in_mask = || {
                self.mask
                    .get(&key)
                    .map(|(uv, placement)| (Type::MASK, uv, placement))
            };
            let in_color = || {
                self.color
                    .get(&key)
                    .map(|(uv, placement)| (Type::COLOR, uv, placement))
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
                Size {
                    width: image.placement.width,
                    height: image.placement.height,
                },
                &image.data,
            ) {
                Ok((uv, placement)) => (ty, uv, placement),
                Err(AtlasError::KeyExists) => unreachable!("Just checked this"),
                Err(AtlasError::OutOfSpace) => todo!("Atlas full"),
                Err(AtlasError::WontFit) => {
                    debug_assert!(false, "Glyph does not fit the atlas");
                    return;
                }
            }
        };

        // Crop to region
        let rectangle = Rectangle::from((font_size, *placement)) + position;
        let uv = uv
            - Position {
                top: rectangle.position().top.min(0),
                left: rectangle.position().left.min(0),
            };
        let Some(rectangle) = rectangle.region(region) else {
            return;
        };

        self.layers.entry(layer).or_default().0.push(Instance {
            ty,
            position: rectangle.position(),
            size: rectangle.size(),
            uv,
            color,
        });
    }

    /// Writes buffer.
    pub fn pre_render(&mut self, queue: &Queue) {
        let mut offset = 0;

        for (_, (instances, range)) in &mut self.layers {
            let instances = bytemuck::cast_slice(instances);
            queue.write_buffer(&self.buffer, offset, instances);
            *range = offset..offset + instances.len() as BufferAddress;
            offset = range.end;
        }
    }

    /// Renders `layer`.
    pub fn render<'pass>(
        &'pass self,
        layer: u32,
        queue: &Queue,
        render_pass: &mut RenderPass<'pass>,
    ) {
        let constants = self.constants.as_array();
        let (instances, range) = match self.layers.get(&layer) {
            Some((instances, range)) if !instances.is_empty() => (instances, range.clone()),
            _ => return,
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_push_constants(Constants::STAGES, 0, bytemuck::cast_slice(&constants));
        render_pass.set_vertex_buffer(0, self.buffer.slice(range));
        render_pass.draw(0..6, 0..instances.len() as u32);
    }

    /// Clears layers.
    pub fn post_render(&mut self) {
        for (instances, range) in self.layers.values_mut() {
            instances.clear();
            *range = Default::default();
        }
    }
}
