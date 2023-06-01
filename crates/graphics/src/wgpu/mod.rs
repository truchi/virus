#![allow(unused)]

mod atlas;

use crate::{colors::Rgba, text::Context};
use std::{collections::HashMap, hash::Hash, num::NonZeroU32};
use swash::scale::image::{Content, Image};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferAddress,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, Extent3d,
    Face, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, Instance, LoadOp,
    Operations, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerBindingType, ShaderStages, Surface, SurfaceConfiguration,
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

        dbg!(image.placement);
        dbg!(image.data.len());
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

        let shader_module = device.create_shader_module(include_wgsl!("shader.wgsl"));
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
}

impl Graphics {
    pub fn new(window: Window, image: &Image) -> Self {
        Self {
            wgpu: Wgpu::new(window, image),
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
    position: [f32; 3],
    is_mask: bool,
    texture: [f32; 2],
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

#[derive(Default, Debug)]
pub struct TextPipelineData {
    mask_texture: (),
    color_texture: (),
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,
}

impl TextPipelineData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, [x, y, z]: [u32; 3], image: &Image, color: Rgba) {}

    pub fn clear(&mut self) {}
}
