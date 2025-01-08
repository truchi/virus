mod atlas;
mod pipeline;

use crate::{
    color::{Rgb, Rgba},
    geom::{Position, Rectangle, Size},
    gpu::{
        atlas::{Atlas, AtlasError},
        pipeline::Pipeline,
    },
    muck::WithAttributes,
    text::{Fonts, GlyphKey, Glyphs},
};
use std::{collections::HashMap, hash::Hash, num::NonZeroU64, ops::Range, sync::Arc};
use swash::{
    scale::{
        image::{Content, Image},
        ScaleContext,
    },
    zeno::Placement,
};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Extent3d, Features,
    FragmentState, ImageCopyTexture, ImageDataLayout, Instance as WGpuInstance, Limits, LoadOp,
    Operations, Origin3d, PipelineLayoutDescriptor, PresentMode, PrimitiveState, PrimitiveTopology,
    PushConstantRange, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType,
    ShaderModule, ShaderStages, StoreOp, Surface, SurfaceConfiguration, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDimension, VertexState,
};
use winit::window::Window;

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
//                                              Gpu                                               //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// WebGPU drawing pipelines.
pub struct Gpu {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    pipeline: Pipeline,
}

impl Gpu {
    /// Creates a new `Gpu`.
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = WGpuInstance::new(Default::default());
        let surface = instance
            .create_surface(window)
            .expect("Cannot create surface");
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
                memory_hints: Default::default(),
            },
            None,
        ))
        .unwrap();
        let config = {
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
        let pipeline = Pipeline::new(&device, &config);

        Self {
            surface,
            config,
            device,
            queue,
            pipeline,
        }
    }

    /// Resizes the surface to the window's logical size.
    pub fn resize(&mut self, window: &Window) {
        let size = window.inner_size();

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.pipeline.resize(&self.device, &self.config);
    }

    /// Returns the size of the surface.
    pub fn size(&self) -> Size {
        Size::new(self.config.width, self.config.height)
    }

    /// Returns the drawing API.
    pub fn draw(&mut self, region: Rectangle) -> Draw {
        Draw {
            region,
            queue: &self.queue,
            pipeline: &mut self.pipeline,
        }
    }

    /// Clears and draws to the screen.
    pub fn render(&mut self, clear: Rgb) {
        // Get output texture from surface
        let output = self.surface.get_current_texture().unwrap();
        let output_texture = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        // Draw pipelines in output texture
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &output_texture,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: (clear.r as f64 / 255.0).powf(2.2),
                        g: (clear.g as f64 / 255.0).powf(2.2),
                        b: (clear.b as f64 / 255.0).powf(2.2),
                        a: 255.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Draw primitives
        self.pipeline.draw(&mut render_pass);

        // Flush
        drop(render_pass); // Runtime error if we don't drop the render pass
        self.queue.submit([encoder.finish()]);
        output.present();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Draw                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

/// Drawing API.
pub struct Draw<'gpu> {
    region: Rectangle,
    queue: &'gpu Queue,
    pipeline: &'gpu mut Pipeline,
}

impl<'gpu> Draw<'gpu> {
    /// Returns the draw region.
    pub fn region(&self) -> Rectangle {
        self.region
    }

    /// Draws `rectangle` with `color`.
    pub fn rectangle(&mut self, rectangle: Rectangle, color: Rgba) -> &mut Self {
        self.pipeline
            .write_rectangle(self.queue, self.region, rectangle, color);

        self
    }

    /// Fills the region with `color`.
    pub fn fill(&mut self, color: Rgba) -> &mut Self {
        self.rectangle(
            Rectangle::from((Position::default(), self.region.size())),
            color,
        );

        self
    }

    /// Draws a glyph at `position` with `color`.
    pub fn glyph<F: FnOnce() -> Image>(
        &mut self,
        position: Position,
        key: GlyphKey,
        color: Rgba,
        image: F,
    ) -> &mut Self {
        self.pipeline
            .write_glyph(self.queue, self.region, position, key, color, image);

        self
    }

    /// Draws glyphs at `position`.
    pub fn glyphs(
        &mut self,
        fonts: &Fonts,
        scale: &mut ScaleContext,
        position: Position,
        line_height: u32,
        glyphs: &Glyphs,
    ) -> &mut Self {
        //
        // Add backgrounds
        //

        for (Range { start, end }, background) in glyphs.backgrounds() {
            self.rectangle(
                Rectangle::new(
                    position.top,
                    position.left + start.round() as i32,
                    (end - start).round() as u32,
                    line_height,
                ),
                background,
            );
        }

        //
        // Add glyphs
        //

        let mut scaler = Glyphs::scaler(fonts, scale);

        for glyph in glyphs.glyphs() {
            self.glyph(
                Position::new(position.top, position.left + glyph.offset.round() as i32),
                glyph.key(),
                glyph.styles.foreground,
                || scaler.render(&glyph),
            );
        }

        self
    }
}
