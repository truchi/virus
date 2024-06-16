use super::*;

macro_rules! label {
    ($label:literal) => {
        Some(concat!("[RectanglePipeline] ", $label))
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Instance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

crate::muck!(unsafe Instance => Instance: [Position, Size, Rgba]);

/// Instance.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Instance {
    /// Rectangle position.
    position: Position,
    /// Rectangle size.
    size: Size,
    /// Rectangle color.
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

    fn bind_group_layout(&self) -> BindGroupLayout {
        self.0.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: label!("Bind group layout"),
            entries: &[],
        })
    }

    fn bind_group(&self, bind_group_layout: &BindGroupLayout) -> BindGroup {
        self.0.create_bind_group(&BindGroupDescriptor {
            label: label!("Bind group"),
            layout: &bind_group_layout,
            entries: &[],
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
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl Pipeline {
    /// Creates a new `Pipeline` for `device` and `config`.
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let max_buffer_size = limits.max_buffer_size;

        let constants = Constants {
            surface: [config.width as f32, config.height as f32],
        };
        let buffer = Init(device).buffer(max_buffer_size); // TODO limit this size
        let layers = Default::default();
        let bind_group_layout = Init(device).bind_group_layout();
        let bind_group = Init(device).bind_group(&bind_group_layout);
        let pipeline = Init(device).pipeline(
            config,
            &bind_group_layout,
            &device.create_shader_module(include_wgsl!("rectangle.wgsl")),
        );

        Self {
            constants,
            layers,
            buffer,
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
        self.constants.resize(config);
    }

    /// Pushes a `rectangle` to be rendered for `layer` in `region` with `color`.
    pub fn push(&mut self, layer: u32, region: Rectangle, rectangle: Rectangle, color: Rgba) {
        if !color.is_visible() {
            return;
        }

        let Some(rectangle) = rectangle.region(region) else {
            return;
        };

        self.layers.entry(layer).or_default().0.push(Instance {
            position: rectangle.position(),
            size: rectangle.size(),
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
