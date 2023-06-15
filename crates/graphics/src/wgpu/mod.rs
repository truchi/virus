mod atlas;
mod line;
mod text;

use crate::{
    colors::Rgba,
    text::{Context, FontKey, FontSize, Line, LineHeight},
};
use atlas::Atlas;
use line::LinePipeline;
use std::ops::Range;
use swash::{scale::image::Content, GlyphId};
use text::TextPipeline;
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, Extent3d, FragmentState,
    ImageCopyTexture, ImageDataLayout, IndexFormat, Instance, LoadOp, Operations, Origin3d,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, SamplerBindingType, ShaderStages, Surface, SurfaceConfiguration,
    Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexState,
    VertexStepMode,
};
use winit::{dpi::PhysicalSize, window::Window};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Draw                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// A drawing API for [`Graphics`].
pub struct Draw<'a> {
    graphics: &'a mut Graphics,
    region: ([i32; 2], [u32; 2]),
}

impl<'a> Draw<'a> {
    /// Returns the top coordinate of the drawing region.
    pub fn top(&self) -> i32 {
        self.region.0[0]
    }

    /// Returns the left coordinate of the drawing region.
    pub fn left(&self) -> i32 {
        self.region.0[1]
    }

    /// Returns the width of the drawing region.
    pub fn width(&self) -> u32 {
        self.region.1[0]
    }

    /// Returns the height of the drawing region.
    pub fn height(&self) -> u32 {
        self.region.1[1]
    }

    /// Draws glyphs.
    pub fn glyphs(
        &mut self,
        context: &mut Context,
        [top, left]: [i32; 2],
        line: &Line,
        line_height: LineHeight,
    ) {
        self.graphics.text_pipeline.glyphs(
            &self.graphics.queue,
            context,
            self.region,
            [top, left],
            line,
            line_height,
        );
    }

    /// Draws a rectange.
    pub fn rectangle(&mut self, ([top, left], [width, height]): ([i32; 2], [u32; 2]), color: Rgba) {
        self.graphics
            .text_pipeline
            .rectangle(self.region, ([top, left], [width, height]), color)
    }

    /// Draws a polyline.
    pub fn polyline<T: IntoIterator<Item = ([i32; 2], Rgba)>>(&mut self, points: T) {
        self.graphics.line_pipeline.polyline(self.region, points);
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
    text_pipeline: TextPipeline,
    line_pipeline: LinePipeline,
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
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default(), None)).unwrap();

        // Configure surface
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        assert!(config.format == TextureFormat::Bgra8UnormSrgb);
        surface.configure(&device, &config);

        // Pipelines
        let text_pipeline = TextPipeline::new(&device, &config);
        let line_pipeline = LinePipeline::new(&device, &config);

        Self {
            window,
            surface,
            config,
            device,
            queue,
            text_pipeline,
            line_pipeline,
        }
    }

    /// Returns the `Window`.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Resizes the surface to the window's logical size.
    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.text_pipeline.resize(size);
        self.line_pipeline.resize(size);
    }

    /// Returns the drawing API.
    pub fn draw(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
    ) -> Draw {
        Draw {
            graphics: self,
            region: ([region_top, region_left], [region_width, region_height]),
        }
    }

    /// Renders to the screen.
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
            self.text_pipeline.render(&self.queue, &mut render_pass);
            self.line_pipeline.render(&self.queue, &mut render_pass);
        }

        self.queue.submit([encoder.finish()]);
        output.present();
    }
}
