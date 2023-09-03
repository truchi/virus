#![allow(unused)]

mod rectangle;

use super::atlas::{Allocator, Horizontal};
use crate::text::{
    AnimatedGlyphKey, Context, FontSize, FrameIndex, Glyph, GlyphKey, Line, LineHeight, LineScaler,
    Shadow,
};
use std::{mem::size_of, ops::Range, time::Duration};
use swash::{scale::image::Content, zeno::Placement};
use virus_common::{muck, Position, Rectangle, Rgba, Size, WithAttributes};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BlendState, Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Extent3d,
    Features, FragmentState, ImageCopyTexture, ImageDataLayout, IndexFormat, Instance, Limits,
    LoadOp, Operations, Origin3d, PipelineLayoutDescriptor, PresentMode, PrimitiveState,
    PrimitiveTopology, PushConstantRange, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    SamplerBindingType, ShaderStages, Surface, SurfaceConfiguration, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexState,
    VertexStepMode,
};
use winit::{dpi::PhysicalSize, window::Window};

const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

type Index = u32;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Constants                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Constants {
    pub surface: [f32; 2],
}

impl Constants {
    // /!\
    pub const SIZE: u32 = 2 * size_of::<f32>() as u32;
    pub const STAGES: ShaderStages = ShaderStages::VERTEX_FRAGMENT;

    pub fn as_array(&self) -> [f32; 2] {
        [self.surface[0], self.surface[1]]
    }

    pub fn resize(&mut self, config: &SurfaceConfiguration) {
        self.surface = [config.width as f32, config.height as f32];
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Graphics                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// WebGpu graphics.
pub struct Graphics {
    window: Window,   // Window and Surface MUST live as long as one another
    surface: Surface, // That's why they are both here, to make sure its safe
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    rectangle: rectangle::Pipeline,
}

impl Graphics {
    /// Creates a new `Graphics`.
    pub fn new(window: Window) -> Self {
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
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Device descriptor"),
                features: Features::PUSH_CONSTANTS,
                limits: Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
            },
            None,
        ))
        .unwrap();

        // Configure surface
        let config = {
            let size = window.inner_size();
            let capabilities = surface.get_capabilities(&adapter);
            SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: capabilities.formats[0],
                width: size.width,
                height: size.height,
                present_mode: PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            }
        };
        surface.configure(&device, &config);

        // Pipelines
        let rectangle = rectangle::Pipeline::new(&device, &config);

        Self {
            window,
            surface,
            config,
            device,
            queue,
            rectangle,
        }
    }

    /// Returns the `Window`.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Returns the size of the window/surface.
    pub fn size(&self) -> Size {
        Size {
            width: self.config.width,
            height: self.config.height,
        }
    }

    /// Resizes the surface to the window's logical size.
    pub fn resize(&mut self) {
        let size = self.window.inner_size();

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Returns the `Draw`ing API.
    pub fn draw(&mut self, layer: u32, region: Rectangle) -> Draw {
        Draw {
            graphics: self,
            layer,
            region,
        }
    }

    /// Renders to the screen.
    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let output_texture = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        // Flush
        self.queue.submit([encoder.finish()]);
        output.present();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Draw                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A drawing API for [`Graphics`].
pub struct Draw<'a> {
    graphics: &'a mut Graphics,
    layer: u32,
    region: Rectangle,
}

impl<'a> Draw<'a> {
    /// Further restricts the current region.
    pub fn region(&mut self, region: Rectangle) -> Option<Draw> {
        self.region.region(region).map(|region| Draw {
            graphics: self.graphics,
            layer: self.layer,
            region,
        })
    }

    /// Draws a rectange.
    pub fn rectangle(&mut self, rectangle: Rectangle, color: Rgba) {
        self.graphics
            .rectangle
            .push(self.layer, self.region, rectangle, color);
    }

    /// Draws glyphs.
    pub fn glyphs(
        &mut self,
        context: &mut Context,
        position: Position,
        line: &Line,
        line_height: LineHeight,
        time: Duration,
    ) {
    }
}
