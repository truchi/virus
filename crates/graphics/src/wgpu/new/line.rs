use super::*;

macro_rules! label {
    ($label:literal) => {
        Some(concat!("[LinePipeline] ", $label))
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Vertex                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

muck!(unsafe Vertex => Vertex: [Position, Rgba]);

/// Vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    /// Point position.
    position: Position,
    /// Point color.
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
            label: label!("Vertex buffer"),
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
                buffers: &[Vertex::buffer_layout()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
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
//                                             Pipeline                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Pipeline.
#[derive(Debug)]
pub struct Pipeline {
    constants: Constants,
    layers: BTreeMap<u32, Vec<Vertex>>,
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
            &device.create_shader_module(include_wgsl!("line.wgsl")),
        );

        Self {
            constants,
            layers,
            buffer,
            bind_group,
            pipeline,
        }
    }

    /// Resizes the `Pipeline`.
    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.constants.resize(config);
    }

    /// Pushes `points` to be rendered for `layer` in `region`.
    pub fn polyline<T: IntoIterator<Item = (Position, Rgba)>>(
        &mut self,
        layer: u32,
        region: Rectangle,
        points: T,
    ) {
        // TODO crop to region

        let mut points = points.into_iter();
        let (mut prev_position, mut prev_color) = if let Some(prev) = points.next() {
            prev
        } else {
            debug_assert!(false);
            return;
        };
        prev_position = prev_position + region.position();

        let layer = &mut self.layers.entry(layer).or_default();
        for (mut curr_position, curr_color) in points {
            curr_position = curr_position + region.position();

            layer.push(Vertex {
                position: prev_position,
                color: prev_color,
            });
            layer.push(Vertex {
                position: curr_position,
                color: curr_color,
            });

            prev_position = curr_position;
            prev_color = curr_color;
        }
    }

    /// Renders `layer`.
    pub fn render<'pass>(
        &'pass self,
        layer: u32,
        queue: &Queue,
        render_pass: &mut RenderPass<'pass>,
    ) {
        let vertices = self
            .layers
            .get(&layer)
            .map(Vec::as_slice)
            .unwrap_or_default();

        if !vertices.is_empty() {
            let constants = self.constants.as_array();

            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(vertices));
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_push_constants(Constants::STAGES, 0, bytemuck::cast_slice(&constants));
            render_pass.set_vertex_buffer(0, self.buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }
    }

    /// Clears layers.
    pub fn post_render(&mut self) {
        for layer in self.layers.values_mut() {
            layer.clear();
        }
    }
}
