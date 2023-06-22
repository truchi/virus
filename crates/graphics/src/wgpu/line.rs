use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Vertex                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    /// Region `[top, left]` world coordinates.
    region_position: [i32; 2],
    /// Region `[width, height]` size.
    region_size: [u32; 2],
    /// Vertex `[top, left]` coordinates in region.
    position: [i32; 2],
    /// sRGBA color.
    color: [u32; 4],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 4] = vertex_attr_array![
        0 => Sint32x2, // region position
        1 => Uint32x2, // region size
        2 => Sint32x2, // position
        3 => Uint32x4, // color
    ];

    fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    fn new(
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        color: Rgba,
    ) -> Self {
        Self {
            region_position: [region_top, region_left],
            region_size: [region_width, region_height],
            position: [top, left],
            color: [
                color.r as u32,
                color.g as u32,
                color.b as u32,
                color.a as u32,
            ],
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Pipeline                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type Sizes = [[u32; 2]; 1];

/// Line pipeline.
pub struct LinePipeline {
    sizes: Sizes,
    vertices: Vec<Vertex>,
    size_uniform: Buffer,
    vertex_buffer: Buffer,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl LinePipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();

        //
        // Buffers
        //

        let size_uniform = device.create_buffer(&buffer! {
            label: "[LinePipeline] Size uniform",
            size: size_of::<Sizes>(),
            usage: UNIFORM | COPY_DST,
        });
        let vertex_buffer = device.create_buffer(&buffer! {
            label: "[LinePipeline] Vertex buffer",
            size: limits.max_buffer_size  / 2,
            usage: VERTEX | COPY_DST,
        });

        //
        // Bind group
        //

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout! {
            label: "[LinePipeline] Texture bind group layout",
            entries: [
                // Size uniform
                { binding: 0, visibility: VERTEX, ty: Uniform },
            ],
        });
        let bind_group = device.create_bind_group(&bind_group! {
            label: "[LinePipeline] Texture bind group",
            layout: bind_group_layout,
            entries: [
                // Size uniform
                { binding: 0, resource: Buffer(size_uniform) },
            ],
        });

        //
        // Pipeline
        //

        let module = device.create_shader_module(include_wgsl!("line.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout! {
            label: "[LinePipeline] Pipeline layout",
            bind_group_layouts: [bind_group_layout],
        });
        let pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[LinePipeline] Pipeline",
            layout: pipeline_layout,
            module: module,
            vertex: "vertex",
            buffers: [Vertex::vertex_buffer_layout()],
            fragment: "fragment",
            targets: [Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            topology: LineList,
        });

        Self {
            sizes: [[config.width, config.height]],
            vertices: Vec::with_capacity(1_024),
            size_uniform,
            vertex_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.sizes[0] = [size.width, size.height];
    }

    pub fn polyline<T: IntoIterator<Item = ([i32; 2], Rgba)>>(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        points: T,
    ) {
        let region = ([region_top, region_left], [region_width, region_height]);

        let mut points = points.into_iter();
        let mut prev = if let Some(prev) = points.next() {
            prev
        } else {
            debug_assert!(false);
            return;
        };

        for curr in points {
            self.vertices.push(Vertex::new(region, prev.0, prev.1));
            self.vertices.push(Vertex::new(region, curr.0, curr.1));
            prev = curr;
        }
    }

    pub fn render<'pass>(&'pass mut self, queue: &Queue, render_pass: &mut RenderPass<'pass>) {
        queue.write_buffer(&self.size_uniform, 0, bytemuck::cast_slice(&self.sizes));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertices.len() as u32, 0..1);

        self.vertices.clear();
    }
}
