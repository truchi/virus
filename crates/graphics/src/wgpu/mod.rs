#![allow(unused)]

mod atlas;

use self::atlas::Atlas;
use crate::{
    colors::Rgba,
    text::{Context, FontKey, FontSize, Line, LineHeight},
};
use swash::{scale::image::Content, GlyphId};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BlendState, Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoderDescriptor, Device, Extent3d, Face, FilterMode, FragmentState,
    FrontFace, ImageCopyTexture, ImageDataLayout, IndexFormat, Instance, LoadOp, Operations,
    Origin3d, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue,
    RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType, SamplerDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderStages, Surface, SurfaceConfiguration, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                                Wgpu                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Wgpu {
    window: Window,
    surface: Surface,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
}

impl Wgpu {
    pub fn new(window: Window) -> Self {
        let size = window.inner_size();

        // WGPU instance
        let instance = Instance::new(Default::default());

        // Surface (window/canvas)
        //
        // SAFETY:
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // Request adapter (device handle), device (gpu connection) and queue (handle to command queue)
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default(), None)).unwrap();

        // Configure surface
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        assert!(config.format == TextureFormat::Rgba8UnormSrgb);
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            config,
            device,
            queue,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Graphics                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Graphics {
    wgpu: Wgpu,
    text_pipeline: TextPipeline,
}

impl Graphics {
    pub fn new(window: Window) -> Self {
        let wgpu = Wgpu::new(window);
        let text_pipeline = TextPipeline::new(&wgpu.device, wgpu.config.format);

        Self {
            wgpu,
            text_pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.wgpu.window
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.wgpu.resize(size)
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn add_line(
        &mut self,
        context: &mut Context,
        top: i32,
        left: i32,
        depth: i32,
        line: &Line,
        line_height: LineHeight,
    ) {
        self.text_pipeline.insert(
            &self.wgpu.queue,
            context,
            top,
            left,
            depth,
            line,
            line_height,
        );
    }

    pub fn render(&mut self) {
        let output = self.wgpu.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self
            .wgpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.text_pipeline
                .render(&self.wgpu.queue, &mut render_pass);
        }

        self.wgpu.queue.submit([encoder.finish()]);
        output.present();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Text...                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    /// - 0: a background rectangle (use `color`),
    /// - 1: a mask glyph (use `texture` in the mask texture with `color`),
    /// - 2: a color glyph (use `texture` in the color texture),
    ty: u32,
    /// Screen coordinates.
    position: [i32; 3],
    /// Texture coordinates.
    texture: [u32; 2],
    /// Rgba color.
    color: [u32; 4],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    pub const BACKGROUND_RECTANGLE_TYPE: u32 = 0;
    pub const MASK_GLYPH_TYPE: u32 = 1;
    pub const COLOR_GLYPH_TYPE: u32 = 2;
    pub const ATTRIBUTES: [VertexAttribute; 4] =
        vertex_attr_array![0 => Uint32, 1 => Sint32x3, 2 => Uint32x2, 3 => Uint32x4];

    pub fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(ty: u32, position: [i32; 3], texture: [u32; 2], color: Rgba) -> Self {
        Self {
            ty,
            position,
            texture,
            color: [
                color.r as u32,
                color.g as u32,
                color.b as u32,
                color.a as u32,
            ],
        }
    }

    pub fn quad(
        ty: u32,
        [top, left]: [i32; 2],
        [width, height]: [u32; 2],
        depth: i32,
        [x, y]: [u32; 2],
        color: Rgba,
    ) -> [Self; 4] {
        let right = left + width as i32;
        let bottom = top + height as i32;
        let x2 = x + width;
        let y2 = y + height;

        [
            Vertex::new(ty, [left, top, depth], [x, y], color),
            Vertex::new(ty, [right, top, depth], [x2, y], color),
            Vertex::new(ty, [left, bottom, depth], [x, y2], color),
            Vertex::new(ty, [right, bottom, depth], [x2, y2], color),
        ]
    }
}

#[derive(Debug)]
pub struct TextPipeline {
    bind_group: BindGroup,
    pipeline: RenderPipeline,
    mask_atlas: Atlas<(FontKey, GlyphId, FontSize)>,
    color_atlas: Atlas<(FontKey, GlyphId, FontSize)>,
    mask_texture: Texture,
    color_texture: Texture,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl TextPipeline {
    pub const ALTAS_ROW_HEIGHT: u32 = 200;

    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let limits = device.limits();
        let [width, height] = [
            limits.max_texture_dimension_2d,
            limits.max_texture_dimension_2d,
        ];

        let mut mask_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, width, height);
        let mut color_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, width, height);
        mask_atlas.next_frame();
        color_atlas.next_frame();

        let texture_descriptor = |label, format| TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let mask_texture = device.create_texture(&texture_descriptor(
            "[TextPipeline] Mask glyphs texture",
            TextureFormat::R8Unorm,
        ));
        let color_texture = device.create_texture(&texture_descriptor(
            "[TextPipeline] Color glyphs texture",
            TextureFormat::Rgba8Unorm,
        ));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("[TextPipeline] texture bind group layout"),
            entries: &[
                // Mask texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
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
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &bind_group_layout,
            entries: &[
                // Mask texture
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &mask_texture.create_view(&Default::default()),
                    ),
                },
                // Color texture
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &color_texture.create_view(&Default::default()),
                    ),
                },
                // Sampler
                BindGroupEntry {
                    binding: 2,
                    // resource: BindingResource::Sampler(&device.create_sampler(&Default::default())),
                    resource: BindingResource::Sampler(&device.create_sampler(
                        &SamplerDescriptor {
                            label: Some("[TextPipeline] Sampler"),
                            min_filter: FilterMode::Nearest,
                            mag_filter: FilterMode::Nearest,
                            mipmap_filter: FilterMode::Nearest,
                            lod_min_clamp: 0f32,
                            lod_max_clamp: 0f32,
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        let shader_module = device.create_shader_module(include_wgsl!("text.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("[TextPipeline] pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("[TextPipeline] pipeline"),
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
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Vertex buffer"),
            size: limits.max_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("[TextPipeline] Index buffer"),
            size: limits.max_buffer_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            bind_group,
            pipeline,
            mask_atlas,
            color_atlas,
            mask_texture,
            color_texture,
            vertices: Vec::with_capacity(1_024),
            indices: Vec::with_capacity(1_024),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn insert(
        &mut self,
        queue: &Queue,
        context: &mut Context,
        top: i32,
        left: i32,
        depth: i32,
        line: &Line,
        line_height: LineHeight,
    ) {
        let mut scaler = line.scaler(context);

        while let Some((advance, glyph, image)) = scaler.next() {
            let image = if let Some(image) = image {
                image
            } else {
                continue;
            };

            let left = left + advance as i32;

            //
            // Add background
            //

            self.insert_quad(Vertex::quad(
                Vertex::BACKGROUND_RECTANGLE_TYPE,
                [top, left],
                [glyph.advance as u32, line_height],
                depth,
                Default::default(),
                glyph.styles.background,
            ));

            //
            // Add glyph
            //

            let key = (glyph.styles.font, glyph.id, line.size());
            let top = top + line.size() as i32;

            // Swash image has placement
            let top = top - image.placement.top;
            let left = left + image.placement.left;
            let width = image.placement.width;
            let height = image.placement.height;

            let (ty, ([x, y], is_new), texture, channels) = match image.content {
                Content::Mask => (
                    Vertex::MASK_GLYPH_TYPE,
                    self.mask_atlas.insert(key, [width, height]).unwrap(),
                    &self.mask_texture,
                    1,
                ),
                Content::Color => (
                    Vertex::COLOR_GLYPH_TYPE,
                    self.color_atlas.insert(key, [4 * width, height]).unwrap(),
                    &self.color_texture,
                    4,
                ),
                Content::SubpixelMask => unreachable!(),
            };

            if is_new {
                queue.write_texture(
                    ImageCopyTexture {
                        texture,
                        mip_level: 0,
                        origin: Origin3d { x, y, z: 0 },
                        aspect: TextureAspect::All,
                    },
                    &image.data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(width * channels),
                        rows_per_image: Some(height),
                    },
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            self.insert_quad(Vertex::quad(
                ty,
                [top, left],
                [width, height],
                depth,
                [x, y],
                glyph.styles.foreground,
            ));
        }
    }

    pub fn render<'pass>(&'pass mut self, queue: &Queue, render_pass: &mut RenderPass<'pass>) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);

        self.mask_atlas.next_frame();
        self.color_atlas.next_frame();
        self.vertices.clear();
        self.indices.clear();
    }

    // TODO remove?
    pub fn clear(&mut self) {
        // self.mask_texture.clear();
        // self.color_texture.clear();
        self.vertices.clear();
        self.indices.clear();
    }

    fn insert_quad(&mut self, [top_left, top_right, bottom_left, bottom_right]: [Vertex; 4]) {
        let i = self.vertices.len() as u32;

        self.vertices
            .extend_from_slice(&[top_left, top_right, bottom_left, bottom_right]);

        let top_left = i;
        let top_right = i + 1;
        let bottom_left = i + 2;
        let bottom_right = i + 3;

        self.indices.extend_from_slice(&[
            top_left,
            bottom_right,
            top_right,
            top_left,
            bottom_left,
            bottom_right,
        ]);
    }
}
