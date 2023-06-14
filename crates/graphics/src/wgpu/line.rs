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
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
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

        let size_uniform = device.create_buffer(&BufferDescriptor {
            label: Some("[LinePipeline] Size uniform"),
            size: std::mem::size_of::<Sizes>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[LinePipeline] Vertex buffer"),
            size: limits.max_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //
        // Bind group
        //

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("[LinePipeline] Texture bind group layout"),
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
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("[LinePipeline] Texture bind group"),
            layout: &bind_group_layout,
            entries: &[
                // Size uniform
                BindGroupEntry {
                    binding: 0,
                    resource: size_uniform.as_entire_binding(),
                },
            ],
        });

        //
        // Pipeline
        //

        let shader_module = device.create_shader_module(include_wgsl!("line.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[LinePipeline] Pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[LinePipeline] Pipeline"),
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
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
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

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        [width, height]: [u32; 2],
        thickness: u32,
        radius: u32,
        color: Rgba,
    ) {
        debug_assert!(2 * (thickness + radius) <= width);
        debug_assert!(2 * (thickness + radius) <= height);

        let region = ([region_top, region_left], [region_width, region_height]);
        let width = width as i32;
        let height = height as i32;
        let thickness = thickness as i32;
        let radius = radius as i32;

        for i in 0..thickness {
            let top = top + i;
            let left = left + i;
            let width = width - 2 * i;
            let height = height - 2 * i;
            let radius = radius - i;

            //
            // Corners
            //

            let andres = Andres(radius as u32);
            let translate = |translate_top, translate_left| {
                move |(t, l)| ([top + translate_top + t, left + translate_left + l], color)
            };

            // Top right
            let top_right = translate(radius, width - radius);
            self.polyline(region, andres.o1().map(top_right));
            self.polyline(region, andres.o2().map(top_right));

            // Top left
            let top_left = translate(radius, radius);
            self.polyline(region, andres.o3().map(top_left));
            self.polyline(region, andres.o4().map(top_left));

            // Bottom left
            let bottom_left = translate(height - radius, radius);
            self.polyline(region, andres.o5().map(bottom_left));
            self.polyline(region, andres.o6().map(bottom_left));

            // Bottom right
            let bottom_right = translate(height - radius, width - radius);
            self.polyline(region, andres.o7().map(bottom_right));
            self.polyline(region, andres.o8().map(bottom_right));

            //
            // Sides
            //

            let translate =
                |translate_top, translate_left| [top + translate_top, left + translate_left];

            self.vertices.extend_from_slice(&[
                // Top
                Vertex::new(region, translate(0, radius), color),
                Vertex::new(region, translate(0, width - radius), color),
                // Right
                Vertex::new(region, translate(radius, width), color),
                Vertex::new(region, translate(height - radius, width), color),
                // Bottom
                Vertex::new(region, translate(height, radius), color),
                Vertex::new(region, translate(height, width - radius), color),
                // Left
                Vertex::new(region, translate(radius, 0), color),
                Vertex::new(region, translate(height - radius, 0), color),
            ]);
        }
    }

    pub fn top_right(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [center_top, center_left]: [i32; 2],
        thickness: u32,
        radius: u32,
        color: Rgba,
    ) {
        self.corner(
            ([region_top, region_left], [region_width, region_height]),
            [center_top, center_left],
            thickness,
            radius,
            color,
            Andres::o1,
            Andres::o2,
        );
    }

    pub fn top_left(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [center_top, center_left]: [i32; 2],
        thickness: u32,
        radius: u32,
        color: Rgba,
    ) {
        self.corner(
            ([region_top, region_left], [region_width, region_height]),
            [center_top, center_left],
            thickness,
            radius,
            color,
            Andres::o3,
            Andres::o4,
        );
    }

    pub fn bottom_left(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [center_top, center_left]: [i32; 2],
        thickness: u32,
        radius: u32,
        color: Rgba,
    ) {
        self.corner(
            ([region_top, region_left], [region_width, region_height]),
            [center_top, center_left],
            thickness,
            radius,
            color,
            Andres::o5,
            Andres::o6,
        );
    }

    pub fn bottom_right(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [center_top, center_left]: [i32; 2],
        thickness: u32,
        radius: u32,
        color: Rgba,
    ) {
        self.corner(
            ([region_top, region_left], [region_width, region_height]),
            [center_top, center_left],
            thickness,
            radius,
            color,
            Andres::o7,
            Andres::o8,
        );
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

/// Private.
impl LinePipeline {
    fn corner<O1, O2>(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [center_top, center_left]: [i32; 2],
        thickness: u32,
        radius: u32,
        color: Rgba,
        o1: fn(&Andres) -> O1,
        o2: fn(&Andres) -> O2,
    ) where
        O1: Iterator<Item = (i32, i32)>,
        O2: Iterator<Item = (i32, i32)>,
    {
        let region = ([region_top, region_left], [region_width, region_height]);
        let thickness = thickness as i32;
        let radius = radius as i32;

        for i in 0..thickness {
            let center_top = center_top + i;
            let center_left = center_left + i;
            let radius = radius - i;

            let andres = Andres(radius as u32);
            let translate = |(t, l)| ([center_top + t, center_left + l], color);

            self.polyline(region, o1(&andres).map(translate));
            self.polyline(region, o2(&andres).map(translate));
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Andres                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO fatorize
struct Andres(/* radius */ u32);

impl Andres {
    fn o1(&self) -> impl Iterator<Item = (i32, i32)> {
        let r = self.0 as i32;

        let mut x = r;
        let mut y = 0;
        let mut d = r - 1;

        std::iter::from_fn(move || {
            if x < y {
                None
            } else {
                let top_left = (-y, x);

                if d >= 2 * y {
                    d -= 2 * y + 1;
                    y += 1;
                } else if d < 2 * (r - x) {
                    d += 2 * x - 1;
                    x -= 1;
                } else {
                    d += 2 * (x - y + 1);
                    x -= 1;
                    y += 1;
                }

                Some(top_left)
            }
        })
    }

    fn o2(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (-left, -top))
    }

    fn o3(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (-left, top))
    }

    fn o4(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (top, -left))
    }

    fn o5(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (-top, -left))
    }

    fn o6(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (left, top))
    }

    fn o7(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (left, -top))
    }

    fn o8(&self) -> impl Iterator<Item = (i32, i32)> {
        self.o1().map(|(top, left)| (-top, left))
    }
}
