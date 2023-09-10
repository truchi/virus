use super::*;

macro_rules! label {
    ($label:literal) => {
        Some(concat!("[RectanglePipeline] ", $label))
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Instance                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Instance => Instance: [Position, Size, Rgba]);

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

    pub fn bind_group_layout(&self) -> BindGroupLayout {
        self.0.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: label!("Bind group layout"),
            entries: &[],
        })
    }

    pub fn bind_group(&self, bind_group_layout: &BindGroupLayout) -> BindGroup {
        self.0.create_bind_group(&BindGroupDescriptor {
            label: label!("Bind group"),
            layout: &bind_group_layout,
            entries: &[],
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
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl Pipeline {
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

    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.constants.resize(config);
    }

    pub fn push(&mut self, layer: u32, region: Rectangle, rectangle: Rectangle, color: Rgba) {
        let Some(rectangle) = rectangle.region(region) else {
            return;
        };

        if !color.is_visible() {
            return;
        }

        self.layers.entry(layer).or_default().push(Instance {
            position: rectangle.position(),
            size: rectangle.size(),
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
