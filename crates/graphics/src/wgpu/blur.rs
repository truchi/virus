use super::*;

pub struct BlurPipeline {
    ping_direction_uniform: Buffer,
    pong_direction_uniform: Buffer,
    bind_group_layout: BindGroupLayout,
    ping_bind_group: BindGroup,
    pong_bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl BlurPipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        ping: &TextureView,
        pong: &TextureView,
    ) -> Self {
        let ping_direction_uniform = device.create_buffer(&BufferDescriptor {
            label: Some("[BlurPipeline] Ping direction uniform"),
            size: size_of::<u32>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pong_direction_uniform = device.create_buffer(&BufferDescriptor {
            label: Some("[BlurPipeline] Pong direction uniform"),
            size: size_of::<u32>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("[BlurPipeline] Bind group layout"),
            entries: &[
                // Direction uniform
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture
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
                // Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&Default::default());
        let ping_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[BlurPipeline] Ping bind group"),
            layout: &bind_group_layout,
            entries: &[
                // Direction uniform
                BindGroupEntry {
                    binding: 0,
                    resource: ping_direction_uniform.as_entire_binding(),
                },
                // Texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(ping),
                },
                // Sampler
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        let pong_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[BlurPipeline] Pong bind group"),
            layout: &bind_group_layout,
            entries: &[
                // Direction uniform
                BindGroupEntry {
                    binding: 0,
                    resource: pong_direction_uniform.as_entire_binding(),
                },
                // Texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(pong),
                },
                // Sampler
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = &device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[BlurPipeline] Pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let module = &device.create_shader_module(include_wgsl!("blur.wgsl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[BlurPipeline] Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module,
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module,
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
            ping_direction_uniform,
            pong_direction_uniform,
            bind_group_layout,
            ping_bind_group,
            pong_bind_group,
            pipeline,
        }
    }

    pub fn rebind(&mut self, device: &Device, ping: &TextureView, pong: &TextureView) {
        let sampler = device.create_sampler(&Default::default());
        self.ping_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[BlurPipeline] Ping bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                // Direction uniform
                BindGroupEntry {
                    binding: 0,
                    resource: self.ping_direction_uniform.as_entire_binding(),
                },
                // Texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(ping),
                },
                // Sampler
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        self.pong_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[BlurPipeline] Pong bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                // Direction uniform
                BindGroupEntry {
                    binding: 0,
                    resource: self.pong_direction_uniform.as_entire_binding(),
                },
                // Texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(pong),
                },
                // Sampler
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
    }

    pub fn ping<'pass>(&'pass mut self, queue: &Queue, render_pass: &mut RenderPass<'pass>) {
        queue.write_buffer(&self.ping_direction_uniform, 0, bytemuck::cast_slice(&[0]));
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.ping_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    pub fn pong<'pass>(&'pass mut self, queue: &Queue, render_pass: &mut RenderPass<'pass>) {
        queue.write_buffer(&self.pong_direction_uniform, 0, bytemuck::cast_slice(&[1]));
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.pong_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
