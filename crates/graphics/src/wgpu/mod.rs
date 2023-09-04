pub mod new;

mod atlas;
mod atlas2;
mod line;
mod macros;
mod text;

use crate::text::{
    AnimatedGlyphKey, Context, FontSize, FrameIndex, Glyph, GlyphKey, Line, LineHeight, LineScaler,
    Shadow,
};
use atlas::{Allocator, Horizontal};
use line::LinePipeline;
use macros::*;
use std::{mem::size_of, ops::Range, time::Duration};
use swash::{scale::image::Content, zeno::Placement};
use text::TextPipeline;
use virus_common::Rgba;
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
        self.text_pipeline.resize(&self.device, &self.config);
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
        let output_texture = output.texture.create_view(&Default::default());
        let blur_ping_texture = self.text_pipeline.blur_ping_texture_view();
        let blur_pong_texture = self.text_pipeline.blur_pong_texture_view();
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        self.text_pipeline.pre_render(&self.queue);
        self.line_pipeline.pre_render(&self.queue);

        // Render rectangles in output texture
        let mut render_pass = encoder.begin_render_pass(&render_pass! {
            label: "Rectangles render pass",
            view: output_texture,
            load: Clear(BLACK),
            store: true,
        });
        self.text_pipeline.render_rectangles(&mut render_pass);
        drop(render_pass);

        // Render shadows in ping texture
        let mut render_pass = encoder.begin_render_pass(&render_pass! {
            label: "Shadows render pass",
            view: blur_ping_texture,
            load: Clear(BLACK),
            store: true,
        });
        self.text_pipeline.render_shadows(&mut render_pass);
        drop(render_pass);

        // Blur ping texture in pong texture
        let mut render_pass = encoder.begin_render_pass(&render_pass! {
            label: "Blur ping render pass",
            view: blur_pong_texture,
            load: Clear(BLACK),
            store: true,
        });
        self.text_pipeline.blur_ping(&mut render_pass);
        drop(render_pass);

        // Blur pong texture in output texture
        let mut render_pass = encoder.begin_render_pass(&render_pass! {
            label: "Blur pong render pass",
            view: output_texture,
            load: Load,
            store: true,
        });
        self.text_pipeline.blur_pong(&mut render_pass);
        drop(render_pass);

        // Render glyphs and lines in output texture
        let mut render_pass = encoder.begin_render_pass(&render_pass! {
            label: "Glyphs and line render pass",
            view: output_texture,
            load: Load,
            store: true,
        });
        self.text_pipeline.render_glyphs(&mut render_pass);
        self.line_pipeline.render(&mut render_pass);
        drop(render_pass);

        // Flush
        self.queue.submit([encoder.finish()]);
        output.present();

        self.text_pipeline.post_render();
        self.line_pipeline.post_render();
    }
}

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

    /// Returns the bottom coordinate of the drawing region.
    pub fn bottom(&self) -> i32 {
        self.top() + self.height() as i32
    }

    /// Returns the left coordinate of the drawing region.
    pub fn left(&self) -> i32 {
        self.region.0[1]
    }

    /// Returns the right coordinate of the drawing region.
    pub fn right(&self) -> i32 {
        self.left() + self.width() as i32
    }

    /// Returns the width of the drawing region.
    pub fn width(&self) -> u32 {
        self.region.1[0]
    }

    /// Returns the height of the drawing region.
    pub fn height(&self) -> u32 {
        self.region.1[1]
    }

    /// Crops relative to the current region.
    pub fn draw(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
    ) -> Draw {
        let (top, bottom) = (
            region_top.clamp(0, self.height() as i32),
            (region_top + region_height as i32).clamp(0, self.height() as i32),
        );
        let (left, right) = (
            region_left.clamp(0, self.width() as i32),
            (region_left + region_width as i32).clamp(0, self.width() as i32),
        );

        Draw {
            region: (
                [self.top() + top, self.left() + left],
                [(right - left) as u32, (bottom - top) as u32],
            ),
            graphics: self.graphics,
        }
    }

    /// Draws glyphs.
    pub fn glyphs(
        &mut self,
        context: &mut Context,
        [top, left]: [i32; 2],
        line: &Line,
        line_height: LineHeight,
        time: Duration,
    ) {
        self.graphics.text_pipeline.glyphs(
            &self.graphics.queue,
            context,
            self.region,
            [top, left],
            line,
            line_height,
            time,
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
