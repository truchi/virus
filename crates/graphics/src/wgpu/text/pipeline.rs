use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           TextPipeline                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const MAX_RECTANGLES: usize = 1_000;
const MAX_SHADOWS: usize = 10_000;
const MAX_GLYPHS: usize = 10_000;
const RECTANGLE_VERTEX_SIZE: usize = size_of::<RectangleVertex>();
const SHADOW_VERTEX_SIZE: usize = size_of::<ShadowVertex>();
const GLYPH_VERTEX_SIZE: usize = size_of::<GlyphVertex>();
const INDEX_SIZE: usize = size_of::<Index>();

type Size = [[f32; 2]; 2];

/// Text pipeline.
#[derive(Debug)]
pub struct TextPipeline {
    size: Size,
    size_uniform: Buffer,

    rectangle_vertices: Vec<RectangleVertex>,
    rectangle_indices: Vec<Index>,
    rectangle_vertex_buffer: Buffer,
    rectangle_index_buffer: Buffer,
    shadow_vertices: Vec<ShadowVertex>,
    shadow_indices: Vec<Index>,
    shadow_vertex_buffer: Buffer,
    shadow_index_buffer: Buffer,
    glyph_vertices: Vec<GlyphVertex>,
    glyph_indices: Vec<Index>,
    glyph_vertex_buffer: Buffer,
    glyph_index_buffer: Buffer,

    mask_atlas: Atlas<GlyphKey, Placement>,
    color_atlas: Atlas<GlyphKey, Placement>,
    animated_atlas: Atlas<AnimatedGlyphKey, Placement>,
    mask_texture: Texture,
    color_texture: Texture,
    animated_texture: Texture,

    bind_group: BindGroup,
    rectangle_pipeline: RenderPipeline,
    shadow_pipeline: RenderPipeline,
    glyph_pipeline: RenderPipeline,
}

// ping_direction_uniform: Buffer,
// pong_direction_uniform: Buffer,

// shadow_vertices: Vec<ShadowVertex>,
// shadow_indices: Vec<Indice>,
// shadow_vertex_buffer: Buffer,
// shadow_index_buffer: Buffer,

// shadow_atlas: Atlas<GlyphKey, Placement>,
// shadow_ping_texture: Texture,
// shadow_pong_texture: Texture,

// pass_bind_group: BindGroup,
// compose_bind_group_layout: BindGroupLayout,
// compose_bind_group: BindGroup,
// compose_pipeline: RenderPipeline,

impl TextPipeline {
    pub const ALTAS_ROW_HEIGHT: u32 = 400;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let limits = device.limits();
        let max_buffer_size = limits.max_buffer_size as usize;
        let max_texture_dimension = limits.max_texture_dimension_2d;

        //
        // Buffers
        //

        let size = [
            [config.width as f32, config.height as f32],
            [max_texture_dimension as f32, max_texture_dimension as f32],
        ];
        let size_uniform = device.create_buffer(&buffer! {
            label: "[TextPipeline] Size uniform",
            size: size_of::<Size>(),
            usage: UNIFORM | COPY_DST,
        });

        let rectangle_vertex_buffer_size = 4 * MAX_RECTANGLES * RECTANGLE_VERTEX_SIZE;
        let rectangle_index_buffer_size = 6 * MAX_RECTANGLES * INDEX_SIZE;
        let shadow_vertex_buffer_size = 4 * MAX_SHADOWS * SHADOW_VERTEX_SIZE;
        let shadow_index_buffer_size = 6 * MAX_SHADOWS * INDEX_SIZE;
        let glyph_vertex_buffer_size = 4 * MAX_GLYPHS * GLYPH_VERTEX_SIZE;
        let glyph_index_buffer_size = 6 * MAX_GLYPHS * INDEX_SIZE;
        assert!(rectangle_vertex_buffer_size <= max_buffer_size);
        assert!(rectangle_index_buffer_size <= max_buffer_size);
        assert!(shadow_vertex_buffer_size <= max_buffer_size);
        assert!(shadow_index_buffer_size <= max_buffer_size);
        assert!(glyph_vertex_buffer_size <= max_buffer_size);
        assert!(glyph_index_buffer_size <= max_buffer_size);

        let rectangle_vertices = Vec::with_capacity(4 * MAX_RECTANGLES);
        let rectangle_indices = Vec::with_capacity(6 * MAX_RECTANGLES);
        let rectangle_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Rectangle vertex buffer",
            size: rectangle_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let rectangle_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Rectangle index buffer",
            size: rectangle_index_buffer_size,
            usage: INDEX | COPY_DST,
        });
        let shadow_vertices = Vec::with_capacity(4 * MAX_SHADOWS);
        let shadow_indices = Vec::with_capacity(6 * MAX_SHADOWS);
        let shadow_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Shadow vertex buffer",
            size: shadow_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let shadow_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Shadow index buffer",
            size: shadow_index_buffer_size,
            usage: INDEX | COPY_DST,
        });
        let glyph_vertices = Vec::with_capacity(4 * MAX_GLYPHS);
        let glyph_indices = Vec::with_capacity(6 * MAX_GLYPHS);
        let glyph_vertex_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Glyph vertex buffer",
            size: glyph_vertex_buffer_size,
            usage: VERTEX | COPY_DST,
        });
        let glyph_index_buffer = device.create_buffer(&buffer! {
            label: "[TextPipeline] Glyph index buffer",
            size: glyph_index_buffer_size,
            usage: INDEX | COPY_DST,
        });

        //
        // Atlases and textures
        //

        // FIXME do we use atlas/texture sizes properly?
        let mask_dimension = max_texture_dimension;
        let color_dimension = max_texture_dimension;
        let animated_dimension = max_texture_dimension;

        let mut mask_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, mask_dimension, mask_dimension);
        let mut color_atlas = Atlas::new(Self::ALTAS_ROW_HEIGHT, color_dimension, color_dimension);
        let mut animated_atlas = Atlas::new(
            Self::ALTAS_ROW_HEIGHT,
            animated_dimension,
            animated_dimension,
        );

        mask_atlas.next_frame();
        color_atlas.next_frame();
        animated_atlas.next_frame();

        let mask_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Mask glyphs texture",
            size: [mask_dimension, mask_dimension],
            format: TextureFormat::R8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let color_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Color glyphs texture",
            size: [color_dimension, color_dimension],
            format: TextureFormat::Rgba8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });
        let animated_texture = device.create_texture(&texture! {
            label: "[TextPipeline] Animated glyphs texture",
            size: [animated_dimension, animated_dimension],
            format: TextureFormat::Rgba8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        });

        //
        // Bind group
        //

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout! {
            label: "[TextPipeline] Bind group layout",
            entries: [
                // Size uniform
                { binding: 0, visibility: VERTEX, ty: Uniform },
                // Mask texture
                { binding: 1, visibility: FRAGMENT, ty: Texture },
                // Color texture
                { binding: 2, visibility: FRAGMENT, ty: Texture },
                // Animated texture
                { binding: 3, visibility: FRAGMENT, ty: Texture },
                // Sampler
                { binding: 4, visibility: FRAGMENT, ty: Sampler(Filtering) },
            ],
        });
        let bind_group = device.create_bind_group(&bind_group! {
            label: "[TextPipeline] Bind group",
            layout: bind_group_layout,
            entries: [
                // Size uniform
                { binding: 0, resource: Buffer(size_uniform) },
                // Mask texture
                { binding: 1, resource: Texture(mask_texture) },
                // Color texture
                { binding: 2, resource: Texture(color_texture) },
                // Animated texture
                { binding: 3, resource: Texture(animated_texture) },
                // Sampler
                { binding: 4, resource: Sampler(device.create_sampler(&Default::default())) },
            ],
        });

        //
        // Pipeline
        //

        let targets = [Some(ColorTargetState {
            format: config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        let layout = device.create_pipeline_layout(&pipeline_layout! {
            label: "[TextPipeline] Pipeline layout",
            bind_group_layouts: [bind_group_layout],
        });
        let module = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));
        let rectangle_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Rectangle pipeline",
            layout: layout,
            module: module,
            vertex: "rectangle_vertex",
            buffers: [RectangleVertex::buffer_layout()],
            fragment: "rectangle_fragment",
            targets: targets,
            topology: TriangleList,
        });
        let shadow_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Shadow pipeline",
            layout: layout,
            module: module,
            vertex: "shadow_vertex",
            buffers: [ShadowVertex::buffer_layout()],
            fragment: "shadow_fragment",
            targets: targets,
            topology: TriangleList,
        });
        let glyph_pipeline = device.create_render_pipeline(&render_pipeline! {
            label: "[TextPipeline] Glyph pipeline",
            layout: layout,
            module: module,
            vertex: "glyph_vertex",
            buffers: [GlyphVertex::buffer_layout()],
            fragment: "glyph_fragment",
            targets: targets,
            topology: TriangleList,
        });

        Self {
            size,
            size_uniform,
            rectangle_vertices,
            rectangle_indices,
            rectangle_vertex_buffer,
            rectangle_index_buffer,
            shadow_vertices,
            shadow_indices,
            shadow_vertex_buffer,
            shadow_index_buffer,
            glyph_vertices,
            glyph_indices,
            glyph_vertex_buffer,
            glyph_index_buffer,
            mask_atlas,
            color_atlas,
            animated_atlas,
            mask_texture,
            color_texture,
            animated_texture,
            bind_group,
            rectangle_pipeline,
            shadow_pipeline,
            glyph_pipeline,
        }
    }

    pub fn resize(&mut self, config: &SurfaceConfiguration) {
        self.size[0] = [config.width as f32, config.height as f32];
    }

    pub fn rectangle(
        &mut self,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        ([top, left], [width, height]): ([i32; 2], [u32; 2]),
        color: Rgba,
    ) {
        Self::push_quad(
            (&mut self.rectangle_vertices, &mut self.rectangle_indices),
            RectangleVertex::quad(
                ([region_top, region_left], [region_width, region_height]),
                ([top, left], [width, height]),
                color,
            ),
        );
    }

    pub fn glyphs(
        &mut self,
        queue: &Queue,
        context: &mut Context,
        ([region_top, region_left], [region_width, region_height]): ([i32; 2], [u32; 2]),
        [top, left]: [i32; 2],
        line: &Line,
        line_height: LineHeight,
        time: Duration,
    ) {
        let region = ([region_top, region_left], [region_width, region_height]);

        // Discard when outside region. This suppposes that:
        // - glyphs are not bigger that line height (~ font size < line height)
        // - glyphs outside do not affect what's inside (~ no shadow FIXME)
        // - no further transforms are applied in the shader
        // Of course the GPU would have done that for us. Don't fear to remove if necessary.
        {
            let above = top + (line_height as i32) < 0;
            let below = top >= region_height as i32;

            if above || below {
                return;
            }
        }

        //
        // Add backgrounds
        //

        for (Range { start, end }, background) in line.backgrounds() {
            if background.a != 0 {
                let left = left + start as i32;
                let width = (end - start) as u32;

                Self::push_quad(
                    (&mut self.rectangle_vertices, &mut self.rectangle_indices),
                    RectangleVertex::quad(
                        ([region_top, region_left], [region_width, region_height]),
                        ([top, left], [width, line_height]),
                        background,
                    ),
                );
            }
        }

        //
        // Add glyphs
        //

        let mut scaler = line.scaler(context);

        for glyph in line.glyphs() {
            let (ty, ([top, left], [width, height]), [u, v]) = if glyph.is_animated() {
                if let Some((ty, [u, v], placement)) =
                    self.insert_animated_glyph(queue, &mut scaler, glyph, time)
                {
                    debug_assert!(placement.top == 0 && placement.left == 0);

                    // Centering vertically by hand
                    let top =
                        top + ((line_height as f32 - placement.width as f32) / 2.0).round() as i32;
                    let left = left + glyph.offset.round() as i32;
                    let width = placement.width;
                    let height = placement.height;

                    (ty, ([top, left], [width, height]), [u, v])
                } else {
                    continue;
                }
            } else {
                if let Some((ty, [u, v], placement)) = self.insert_glyph(queue, &mut scaler, glyph)
                {
                    // Swash image has placement (vertical up from baseline)
                    let top = top + line.size() as i32 - placement.top;
                    let left = left + glyph.offset.round() as i32 + placement.left;
                    let width = placement.width;
                    let height = placement.height;

                    (ty, ([top, left], [width, height]), [u, v])
                } else {
                    continue;
                }
            };

            if let Some(_shadow) = glyph
                .styles
                .shadow
                .filter(|shadow| shadow.radius > 0 && shadow.color.a != 0)
            {
                Self::push_quad(
                    (&mut self.shadow_vertices, &mut self.shadow_indices),
                    ShadowVertex::quad(ty, region, ([top, left], [width, height]), [u, v]),
                );
            }

            Self::push_quad(
                (&mut self.glyph_vertices, &mut self.glyph_indices),
                GlyphVertex::quad(
                    ty,
                    region,
                    ([top, left], [width, height]),
                    [u, v],
                    glyph.styles.foreground,
                ),
            );
        }
    }

    pub fn pre_render(&mut self, queue: &Queue) {
        queue.write_buffer(&self.size_uniform, 0, bytemuck::cast_slice(&self.size));
        queue.write_buffer(
            &self.rectangle_vertex_buffer,
            0,
            bytemuck::cast_slice(&self.rectangle_vertices),
        );
        queue.write_buffer(
            &self.rectangle_index_buffer,
            0,
            bytemuck::cast_slice(&self.rectangle_indices),
        );
        queue.write_buffer(
            &self.shadow_vertex_buffer,
            0,
            bytemuck::cast_slice(&self.shadow_vertices),
        );
        queue.write_buffer(
            &self.shadow_index_buffer,
            0,
            bytemuck::cast_slice(&self.shadow_indices),
        );
        queue.write_buffer(
            &self.glyph_vertex_buffer,
            0,
            bytemuck::cast_slice(&self.glyph_vertices),
        );
        queue.write_buffer(
            &self.glyph_index_buffer,
            0,
            bytemuck::cast_slice(&self.glyph_indices),
        );
    }

    pub fn render_rectangles<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.rectangle_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.rectangle_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.rectangle_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.rectangle_indices.len() as u32, 0, 0..1);
    }

    pub fn render_shadows<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.shadow_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.shadow_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.shadow_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.shadow_indices.len() as u32, 0, 0..1);
    }

    pub fn render_glyphs<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.glyph_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.glyph_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.glyph_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.glyph_indices.len() as u32, 0, 0..1);
    }

    pub fn post_render(&mut self) {
        self.rectangle_vertices.clear();
        self.rectangle_indices.clear();
        self.shadow_vertices.clear();
        self.shadow_indices.clear();
        self.glyph_vertices.clear();
        self.glyph_indices.clear();

        self.mask_atlas.next_frame();
        self.color_atlas.next_frame();
        self.animated_atlas.next_frame();
    }
}

/// Private.
impl TextPipeline {
    fn insert_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        glyph: &Glyph,
    ) -> Option<(u32, [u32; 2], Placement)> {
        let key = glyph.key();

        // Check atlases for glyph
        if let Some((ty, ([u, v], placement))) = {
            let in_mask = || self.mask_atlas.get(&key).map(|v| (MASK_GLYPH, v));
            let in_color = || self.color_atlas.get(&key).map(|v| (COLOR_GLYPH, v));
            in_mask().or_else(in_color)
        } {
            return Some((ty, [u, v], *placement));
        }

        // Render glyph
        let image = scaler.render(&glyph)?;
        let placement = image.placement;
        let [width, height] = [placement.width, placement.height];

        // Allocate glyph in atlas
        let (ty, atlas, texture, channels) = match image.content {
            Content::Mask => (MASK_GLYPH, &mut self.mask_atlas, &self.mask_texture, 1),
            Content::Color => (COLOR_GLYPH, &mut self.color_atlas, &self.color_texture, 4),
            Content::SubpixelMask => unreachable!(),
        };
        let ([u, v], _) = {
            atlas.insert(key, placement, [width, height]).unwrap();
            atlas.get(&key).unwrap()
        };

        // Insert glyph in atlas
        queue.write_texture(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d { x: u, y: v, z: 0 },
                aspect: TextureAspect::All,
            },
            &image.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(channels * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Some((ty, [u, v], placement))
    }

    fn insert_animated_glyph(
        &mut self,
        queue: &Queue,
        scaler: &mut LineScaler,
        glyph: &Glyph,
        time: Duration,
    ) -> Option<(u32, [u32; 2], Placement)> {
        let id = glyph.animated_id?;
        let key = (glyph.size, id, scaler.frame(glyph, time)?);

        // Check atlas for frame
        if let Some(([u, v], placement)) = self.animated_atlas.get(&key) {
            return Some((ANIMATED_GLYPH, [u, v], *placement));
        }

        // Render frames
        let frames = scaler.render_animated(&glyph)?;
        debug_assert!(FrameIndex::try_from(frames.len()).is_ok());

        for (index, frame) in frames.iter().enumerate() {
            let placement = frame.placement;
            let [width, height] = [placement.width, placement.height];

            // Allocate frame in atlas
            let ([u, v], _) = {
                let key = (glyph.size, id, index as FrameIndex);
                self.animated_atlas
                    .insert(key, placement, [width, height])
                    .unwrap();
                self.animated_atlas.get(&key).unwrap()
            };

            // Insert frame in atlas
            queue.write_texture(
                ImageCopyTexture {
                    texture: &self.animated_texture,
                    mip_level: 0,
                    origin: Origin3d { x: u, y: v, z: 0 },
                    aspect: TextureAspect::All,
                },
                &frame.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let ([u, v], placement) = self.animated_atlas.get(&key).unwrap();
        Some((ANIMATED_GLYPH, [u, v], *placement))
    }

    fn push_quad<T: Copy>(
        (vertices, indices): (&mut Vec<T>, &mut Vec<Index>),
        [top_left, top_right, bottom_left, bottom_right]: [T; 4],
    ) {
        let i = vertices.len() as u32;

        vertices.extend_from_slice(&[top_left, top_right, bottom_left, bottom_right]);

        let top_left = i;
        let top_right = i + 1;
        let bottom_left = i + 2;
        let bottom_right = i + 3;

        indices.extend_from_slice(&[
            top_left,
            bottom_right,
            top_right,
            top_left,
            bottom_left,
            bottom_right,
        ]);
    }
}
