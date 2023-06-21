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
        //
        // Buffers
        //

        let ping_direction_uniform = device.create_buffer(&buffer! {
            label: "[BlurPipeline] Ping direction uniform",
            size: size_of::<u32>(),
            usage: UNIFORM | COPY_DST,
        });
        let pong_direction_uniform = device.create_buffer(&buffer! {
            label: "[BlurPipeline] Pong direction uniform",
            size: size_of::<u32>(),
            usage: UNIFORM | COPY_DST,
        });

        //
        // Bind groups
        //

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout! {
            label: "[BlurPipeline] Bind group layout",
            entries: [
                // Direction uniform
                { binding: 0, visibility: FRAGMENT, ty: Uniform },
                // Texture
                { binding: 1, visibility: FRAGMENT, ty: Texture },
                // Sampler
                { binding: 2, visibility: FRAGMENT, ty: Sampler(Filtering) },
            ],
        });
        let [ping_bind_group, pong_bind_group] = Self::bind_groups(
            device,
            &bind_group_layout,
            &ping_direction_uniform,
            &pong_direction_uniform,
            ping,
            pong,
            &device.create_sampler(&Default::default()),
        );

        //
        // Pipeline
        //

        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout! {
            label: "[BlurPipeline] Pipeline layout",
            bind_group_layouts: [bind_group_layout],
        });
        let module = device.create_shader_module(include_wgsl!("blur.wgsl"));
        let pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[BlurPipeline] Pipeline",
            layout: pipeline_layout,
            module: module,
            vertex: "vertex",
            buffers: [],
            fragment: "fragment",
            targets: [Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            topology: TriangleList,
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
        let [ping_bind_group, pong_bind_group] = Self::bind_groups(
            device,
            &self.bind_group_layout,
            &self.ping_direction_uniform,
            &self.pong_direction_uniform,
            ping,
            pong,
            &device.create_sampler(&Default::default()),
        );

        self.ping_bind_group = ping_bind_group;
        self.pong_bind_group = pong_bind_group;
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

impl BlurPipeline {
    fn bind_groups(
        device: &Device,
        layout: &BindGroupLayout,
        ping_direction_uniform: &Buffer,
        pong_direction_uniform: &Buffer,
        ping: &TextureView,
        pong: &TextureView,
        sampler: &Sampler,
    ) -> [BindGroup; 2] {
        [
            device.create_bind_group(&bind_group! {
                label: "[BlurPipeline] Ping bind group",
                layout: layout,
                entries: [
                    // Direction uniform
                    { binding: 0, resource: Buffer(ping_direction_uniform) },
                    // Texture
                    { binding: 1, resource: TextureView(ping) },
                    // Sampler
                    { binding: 2, resource: Sampler(sampler) },
                ],
            }),
            device.create_bind_group(&bind_group! {
                label: "[BlurPipeline] Pong bind group",
                layout: layout,
                entries: [
                    // Direction uniform
                    { binding: 0, resource: Buffer(pong_direction_uniform) },
                    // Texture
                    { binding: 1, resource: TextureView(pong) },
                    // Sampler
                    { binding: 2, resource: Sampler(sampler) },
                ],
            }),
        ]
    }
}
