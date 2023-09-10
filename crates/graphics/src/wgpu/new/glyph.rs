use super::*;

macro_rules! label {
    ($label:literal) => {
        Some(concat!("[GlyphPipeline] ", $label))
    };
}

const MASK_BIN: u32 = 400;
const COLOR_BIN: u32 = 400;
const MASK_ATLAS_FACTOR: u32 = 2;
const COLOR_ATLAS_FACTOR: u32 = 2;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Type                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Type => Uint32);

/// [`Type::MASK`]/[`Type::COLOR`].
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

muck!(unsafe Instance => Instance: [Type, Position, Size, Position, Rgba]);

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

pub struct Init<'a>(pub &'a Device);

impl<'a> Init<'a> {
    pub fn buffer(&self, size: BufferAddress) -> Buffer {
        self.0.create_buffer(&BufferDescriptor {
            label: label!("Instance buffer"),
            size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn mask_texture(
        &self,
        max_texture_dimension: u32,
        config: &SurfaceConfiguration,
    ) -> Texture {
        self.0.create_texture(&TextureDescriptor {
            label: label!("Mask texture"),
            size: Extent3d {
                width: max_texture_dimension.min(MASK_ATLAS_FACTOR * config.width),
                height: max_texture_dimension.min(MASK_ATLAS_FACTOR * config.height),
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

    pub fn color_texture(
        &self,
        max_texture_dimension: u32,
        config: &SurfaceConfiguration,
    ) -> Texture {
        self.0.create_texture(&TextureDescriptor {
            label: label!("Color texture"),
            size: Extent3d {
                width: max_texture_dimension.min(COLOR_ATLAS_FACTOR * config.width),
                height: max_texture_dimension.min(COLOR_ATLAS_FACTOR * config.height),
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

    pub fn bind_group_layout(&self) -> BindGroupLayout {
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

    pub fn bind_group(
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

    pub fn pipeline(
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
                range: 0..Constants::SIZE as u32,
            }],
        });

        self.0.create_render_pipeline(&RenderPipelineDescriptor {
            label: label!("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: "vertex",
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

/// Rectangle pipeline.
#[derive(Debug)]
pub struct Pipeline {
    constants: Constants,
    layers: BTreeMap<u32, Vec<Instance>>,
    buffer: Buffer,
    mask: Atlas<Horizontal, GlyphKey, Placement>,
    color: Atlas<Horizontal, GlyphKey, Placement>,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl Pipeline {
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
            MASK_BIN,
        );
        let color = Atlas::new(
            Init(device).color_texture(max_texture_dimension, config),
            COLOR_BIN,
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

    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        let max_texture_dimension = device.limits().max_texture_dimension_2d;

        self.constants.resize(config);
        self.mask.clear_and_resize(
            Init(device).mask_texture(max_texture_dimension, config),
            MASK_BIN,
        );
        self.color.clear_and_resize(
            Init(device).color_texture(max_texture_dimension, config),
            COLOR_BIN,
        );
        self.bind_group = Init(device).bind_group(
            &self.bind_group_layout,
            self.mask.texture(),
            self.color.texture(),
        );
    }

    pub fn push<F: FnOnce() -> Option<Image>>(
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
        if !color.is_visible() {
            return;
        }

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
            let image = if let Some(image) = image() {
                image
            } else {
                return;
            };

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
                Err(AtlasError::KeyExists) => unreachable!(),
                Err(AtlasError::WontFit) => return,
                Err(AtlasError::OutOfSpace) => unimplemented!(),
            }
        };

        // TODO apply region to position, size and uv
        self.layers.entry(layer).or_default().push(Instance {
            ty,
            // Swash image placement has vertical up, from baseline
            position: Position {
                top: region.top + position.top + font_size as i32 - placement.top,
                left: region.left + position.left + placement.left,
            },
            size: Size {
                width: placement.width,
                height: placement.height,
            },
            uv,
            color,
        });
    }

    pub fn render<'pass>(
        &'pass self,
        layer: u32,
        queue: &Queue,
        render_pass: &mut RenderPass<'pass>,
    ) {
        let instances: &[Instance] = self
            .layers
            .get(&layer)
            .map(Vec::as_slice)
            .unwrap_or_default();

        if !instances.is_empty() {
            let constants = self.constants.as_array();

            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(instances));
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_push_constants(Constants::STAGES, 0, bytemuck::cast_slice(&constants));
            render_pass.set_vertex_buffer(0, self.buffer.slice(..));
            render_pass.draw(0..6, 0..instances.len() as u32);
        }
    }

    pub fn post_render(&mut self) {
        for layer in self.layers.values_mut() {
            layer.clear();
        }
    }
}
