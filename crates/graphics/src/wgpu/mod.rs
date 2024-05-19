#![allow(unused)]

mod atlas;
mod glyph;
mod line;
mod rectangle;

use crate::text::{Context, FontSize, Glyph, GlyphKey, Line, LineHeight, LineScaler};
use atlas::{Atlas, AtlasError};
use glyph::Pipeline as GlyphPipeline;
use line::Pipeline as LinePipeline;
use rectangle::Pipeline as RectanglePipeline;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    ops::Range,
    sync::Arc,
    time::Duration,
};
use swash::{
    scale::image::{Content, Image},
    zeno::Placement,
};
use virus_common::{muck, Position, Rectangle, Rgba, Size, WithAttributes};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BlendState, Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Extent3d,
    Features, FragmentState, ImageCopyTexture, ImageDataLayout, IndexFormat, Instance, Limits,
    LoadOp, Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PresentMode,
    PrimitiveState, PrimitiveTopology, PushConstantRange, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerBindingType, ShaderModule, ShaderStages, StoreOp, Surface,
    SurfaceConfiguration, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::{dpi::PhysicalSize, window::Window};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Constants                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Default, Debug)]
struct Constants {
    surface: [f32; 2],
}

impl Constants {
    const STAGES: ShaderStages = ShaderStages::VERTEX_FRAGMENT;

    fn as_array(&self) -> [f32; 2] {
        [self.surface[0], self.surface[1]]
    }

    fn resize(&mut self, config: &SurfaceConfiguration) {
        self.surface = [config.width as f32, config.height as f32];
    }

    fn size() -> u32 {
        std::mem::size_of_val(&Self::default().as_array()) as u32
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Graphics                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// WebGpu graphics.
pub struct Graphics {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    rectangle: RectanglePipeline,
    glyph: GlyphPipeline,
    line: LinePipeline,
}

impl Graphics {
    /// Creates a new `Graphics`.
    pub fn new(window: &Arc<Window>) -> Self {
        // WGPU instance
        let instance = Instance::new(Default::default());

        // Surface (window/canvas)
        let surface = instance
            .create_surface(Arc::clone(window))
            .expect("Cannot create surface");

        // Request adapter (device handle), device (gpu connection) and queue (handle to command queue)
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Device descriptor"),
                required_features: Features::PUSH_CONSTANTS,
                required_limits: Limits {
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
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            }
        };
        surface.configure(&device, &config);

        // Pipelines
        let rectangle = RectanglePipeline::new(&device, &config);
        let glyph = GlyphPipeline::new(&device, &config);
        let line = LinePipeline::new(&device, &config);

        Self {
            surface,
            config,
            device,
            queue,
            rectangle,
            glyph,
            line,
        }
    }

    /// Resizes the surface to the window's logical size.
    pub fn resize(&mut self, window: &Window) {
        let size = window.inner_size();
        self.config.width = size.width;
        self.config.height = size.height;

        self.surface.configure(&self.device, &self.config);
        self.rectangle.resize(&self.device, &self.config);
        self.glyph.resize(&self.device, &self.config);
        self.line.resize(&self.device, &self.config);
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
        // Get output texture from surface
        let output = self.surface.get_current_texture().unwrap();
        let output_texture = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        // Render pipelines in output texture
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Rectangle render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &output_texture,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear({
                        // Catppuccin latte
                        let base = Color {
                            r: 0.937,
                            g: 0.945,
                            b: 0.960,
                            a: 1.0,
                        };
                        let crust = Color {
                            r: 0.862,
                            g: 0.878,
                            b: 0.909,
                            a: 1.0,
                        };
                        crust
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.rectangle.render(0, &self.queue, &mut render_pass);
        self.glyph.render(0, &self.queue, &mut render_pass);
        self.line.render(0, &self.queue, &mut render_pass);
        drop(render_pass);

        // Flush
        self.queue.submit([encoder.finish()]);
        output.present();

        // Clear pipelines
        self.rectangle.clear();
        self.glyph.clear();
        self.line.clear();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Draw                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A drawing API for [`Graphics`].
pub struct Draw<'graphics> {
    graphics: &'graphics mut Graphics,
    layer: u32,
    region: Rectangle,
}

impl<'graphics> Draw<'graphics> {
    /// Returns the current region.
    pub fn region(&self) -> Rectangle {
        self.region
    }

    /// Returns the current layer.
    pub fn layer(&self) -> u32 {
        self.layer
    }

    /// Draws a rectange.
    pub fn rectangle(&mut self, rectangle: Rectangle, color: Rgba) {
        self.graphics
            .rectangle
            .push(self.layer, self.region, rectangle, color);
    }

    /// Draws a glyph.
    pub fn glyph<F: FnOnce() -> Image>(
        &mut self,
        position: Position,
        font_size: FontSize,
        key: GlyphKey,
        color: Rgba,
        image: F,
    ) {
        self.graphics.glyph.push(
            &self.graphics.queue,
            self.layer,
            self.region,
            position,
            font_size,
            key,
            color,
            image,
        );
    }

    /// Draws glyphs.
    pub fn glyphs(
        &mut self,
        context: &mut Context,
        position: Position,
        line: &Line,
        line_height: LineHeight,
    ) {
        //
        // Add backgrounds
        //

        for (Range { start, end }, _, background) in line.segments(|glyph| glyph.styles.background)
        {
            self.rectangle(
                Rectangle {
                    top: position.top,
                    left: position.left + start as i32,
                    width: (end - start) as u32,
                    height: line_height,
                },
                background,
            );
        }

        //
        // Add glyphs
        //

        let mut scaler = line.scaler(context);

        for glyph in line.glyphs() {
            self.glyph(
                Position {
                    top: position.top,
                    left: position.left + glyph.offset.round() as i32,
                },
                line.font_size(),
                glyph.key(),
                glyph.styles.foreground,
                || scaler.render(&glyph),
            );
        }
    }

    /// Draws a polyline.
    pub fn polyline<T: IntoIterator<Item = (Position, Rgba)>>(&mut self, points: T) {
        self.graphics
            .line
            .points(self.layer, self.region, points, false);
    }

    /// Draws a polygon.
    pub fn polygon<T: IntoIterator<Item = (Position, Rgba)>>(&mut self, points: T) {
        self.graphics
            .line
            .points(self.layer, self.region, points, true);
    }
}
