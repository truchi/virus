#![allow(unused)]

mod atlas;

use self::atlas::Atlas;
use crate::{
    colors::Rgba,
    text::{Context, FontKey, FontSize, Line, LineHeight},
};
use std::{collections::HashMap, hash::Hash, num::NonZeroU32};
use swash::{
    scale::image::{Content, Image},
    GlyphId,
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferAddress, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, Extent3d, Face, FragmentState, FrontFace, ImageCopyTexture,
    ImageDataLayout, Instance, LoadOp, Operations, PipelineLayout, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType,
    ShaderModule, ShaderModuleDescriptor, ShaderStages, Surface, SurfaceConfiguration, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
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
    pipeline: RenderPipeline,
    texture_bind_group: BindGroup,
    vertex_buffer: Buffer,
}

impl Wgpu {
    pub fn new(window: Window, image: &Image) -> Self {
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

        //
        // TEXTURE
        //

        assert!(image.content == Content::Mask);

        let texture = device.create_texture_with_data(
            &queue,
            &TextureDescriptor {
                label: Some("Texture"),
                size: Extent3d {
                    width: image.placement.width,
                    height: image.placement.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &image.data,
        );
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    // Texture
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
                    // Sampler
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                // Texture
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &texture.create_view(&Default::default()),
                    ),
                },
                // Sampler
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&device.create_sampler(&Default::default())),
                },
            ],
        });

        //
        // PIPELINE
        //

        let shader_module = device.create_shader_module(include_wgsl!("text.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vertex",
                buffers: &[VertexBufferLayout {
                    array_stride: 5 * std::mem::size_of::<f32>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: Default::default(),
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        //
        // VERTICES
        //

        const VERTICES: &[f32] = &[
            /* FIRST TRIANGLE */
            /* Top left */
            -1.0, 1.0, 0.0, // position
            0.0, 0.0, // texture coordinates
            /* Top right */
            1.0, 1.0, 0.0, // position
            1.0, 0.0, // texture coordinates
            /* Bottom right */
            1.0, -1.0, 0.0, // position
            1.0, 1.0, // texture coordinates
            /* SECOND TRIANGLE */
            /* Top left */
            -1.0, 1.0, 0.0, // position
            0.0, 0.0, // texture coordinates
            /* Bottom right */
            1.0, -1.0, 0.0, // position
            1.0, 1.0, // texture coordinates
            /* Bottom left */
            -1.0, -1.0, 0.0, // position
            0.0, 1.0, // texture coordinates
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        Self {
            window,
            surface,
            config,
            device,
            queue,
            pipeline,
            texture_bind_group,
            vertex_buffer,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self
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
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        output.present();
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
    pub fn new(window: Window, image: &Image) -> Self {
        let wgpu = Wgpu::new(window, image);
        dbg!(wgpu.device.limits());
        panic!("OK");

        let text_pipeline = TextPipeline::new(&wgpu.device, &wgpu.queue, wgpu.config.format);

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

    pub fn render(&mut self) {
        self.wgpu.render();
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
    color: [u8; 4],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Vertex {
    pub const BACKGROUND_RECTANGLE_TYPE: u32 = 0;
    pub const MASK_GLYPH_TYPE: u32 = 1;
    pub const COLOR_GLYPH_TYPE: u32 = 2;

    // TODO
    pub fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }

    pub fn new(ty: u32, position: [i32; 3], texture: [u32; 2], color: Rgba) -> Self {
        Self {
            ty,
            position,
            texture,
            color: [color.r, color.g, color.b, color.a],
        }
    }

    pub fn quad(
        ty: u32,
        [top, left]: [i32; 2],
        [width, height]: [u32; 2],
        depth: i32,
        texture: [u32; 2],
        color: Rgba,
    ) -> [Self; 4] {
        let right = left + width as i32;
        let bottom = top + height as i32;

        [
            Vertex::new(ty, [top, left, depth], texture, color),
            Vertex::new(ty, [top, right, depth], texture, color),
            Vertex::new(ty, [bottom, left, depth], texture, color),
            Vertex::new(ty, [bottom, right, depth], texture, color),
        ]
    }
}

#[derive(Debug)]
pub struct TextPipeline {
    pipeline: RenderPipeline,
    mask_texture: Atlas<(FontKey, GlyphId, FontSize)>,
    color_texture: Atlas<(FontKey, GlyphId, FontSize)>,
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u32>,
}

impl TextPipeline {
    const ATLAS_WIDTH: usize = 256;

    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("text.wgsl"));

        // let texture = device.create_texture_with_data(
        //     &queue,
        //     &TextureDescriptor {
        //         label: Some("TODO"),
        //         size: Extent3d {
        //             width: image.placement.width,
        //             height: image.placement.height,
        //             depth_or_array_layers: 1,
        //         },
        //         mip_level_count: 1,
        //         sample_count: 1,
        //         dimension: TextureDimension::D2,
        //         format: TextureFormat::R8Unorm,
        //         usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        //         view_formats: &[],
        //     },
        //     &image.data,
        // );

        // let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        //     label: Some("[TextPipeline] texture bind group layout"),
        //     entries: &[
        //         // Texture
        //         BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: BindingType::Texture {
        //                 multisampled: false,
        //                 view_dimension: TextureViewDimension::D2,
        //                 sample_type: TextureSampleType::Float { filterable: true },
        //             },
        //             count: None,
        //         },
        //         // Sampler
        //         BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: BindingType::Sampler(SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ],
        // });
        // let bind_group = device.create_bind_group(&BindGroupDescriptor {
        //     label: Some("Texture bind group"),
        //     layout: &bind_group_layout,
        //     entries: &[
        //         // Texture
        //         BindGroupEntry {
        //             binding: 0,
        //             resource: BindingResource::TextureView(
        //                 &texture.create_view(&Default::default()),
        //             ),
        //         },
        //         // Sampler
        //         BindGroupEntry {
        //             binding: 1,
        //             resource: BindingResource::Sampler(&device.create_sampler(&Default::default())),
        //         },
        //     ],
        // });

        // let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        //     label: Some("[TextPipeline] pipeline layout"),
        //     bind_group_layouts: &[&bind_group_layout],
        //     push_constant_ranges: &[],
        // });
        // let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        //     label: Some("[TextPipeline] pipeline"),
        //     layout: Some(&pipeline_layout),
        //     vertex: VertexState {
        //         module: &shader_module,
        //         entry_point: "vertex",
        //         buffers: &[Vertex::vertex_buffer_layout()],
        //     },
        //     fragment: Some(FragmentState {
        //         module: &shader_module,
        //         entry_point: "fragment",
        //         targets: &[Some(format.into())],
        //     }),
        //     primitive: Default::default(),
        //     depth_stencil: None,
        //     multisample: Default::default(),
        //     multiview: None,
        // });

        Self {
            pipeline: todo!(),
            mask_texture: Atlas::new(Self::ATLAS_WIDTH),
            color_texture: Atlas::new(Self::ATLAS_WIDTH),
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
        }
    }

    pub fn insert(
        &mut self,
        context: &mut Context,
        top: i32,
        left: i32,
        depth: i32,
        line: &Line,
        height: LineHeight,
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
                [glyph.advance as u32, height],
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

            let (ty, texture) = match image.content {
                Content::Mask => (
                    Vertex::MASK_GLYPH_TYPE,
                    self.mask_texture
                        .set(key, [width as usize, height as usize], &image.data)
                        .unwrap(),
                ),
                Content::Color => (
                    Vertex::COLOR_GLYPH_TYPE,
                    self.mask_texture
                        .set(key, [4 * width as usize, height as usize], &image.data)
                        .unwrap(),
                ),
                Content::SubpixelMask => unreachable!(),
            };

            self.insert_quad(Vertex::quad(
                ty,
                [top, left],
                [width, height],
                depth,
                [texture[0] as u32, texture[1] as u32],
                glyph.styles.foreground,
            ));
        }
    }

    pub fn clear(&mut self) {
        self.mask_texture.clear();
        self.color_texture.clear();
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }

    fn insert_quad(&mut self, [top_left, top_right, bottom_left, bottom_right]: [Vertex; 4]) {
        self.vertex_buffer.reserve(4);
        self.index_buffer.reserve(6);

        let i = self.vertex_buffer.len() as u32;

        self.vertex_buffer.push(top_left);
        self.vertex_buffer.push(top_right);
        self.vertex_buffer.push(bottom_left);
        self.vertex_buffer.push(bottom_right);

        let top_left = i;
        let top_right = i + 1;
        let bottom_left = i + 2;
        let bottom_right = i + 3;

        self.index_buffer.push(top_left);
        self.index_buffer.push(bottom_right);
        self.index_buffer.push(top_right);

        self.index_buffer.push(top_left);
        self.index_buffer.push(bottom_left);
        self.index_buffer.push(bottom_right);
    }
}
