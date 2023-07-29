use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Init                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Init<'a>(pub &'a Device);

impl<'a> Init<'a> {
    pub fn buffers(&self, max_buffer_size: usize) -> Buffers {
        fn size<const MAX_QUADS: usize, Vertex>() -> [usize; 2] {
            [
                size_of::<Vertex>() * VERTICES_PER_QUAD * MAX_QUADS,
                size_of::<Index>() * INDICES_PER_QUAD * MAX_QUADS,
            ]
        }

        let [rectangle_vertex_buffer_size, rectangle_index_buffer_size] =
            size::<MAX_RECTANGLES, RectangleVertex>();
        let [shadow_vertex_buffer_size, shadow_index_buffer_size] =
            size::<MAX_SHADOWS, ShadowVertex>();
        let [glyph_vertex_buffer_size, glyph_index_buffer_size] = size::<MAX_GLYPHS, GlyphVertex>();
        let [blur_vertex_buffer_size, blur_index_buffer_size] = size::<MAX_BLURS, BlurVertex>();

        assert!(rectangle_vertex_buffer_size <= max_buffer_size);
        assert!(rectangle_index_buffer_size <= max_buffer_size);
        assert!(shadow_vertex_buffer_size <= max_buffer_size);
        assert!(shadow_index_buffer_size <= max_buffer_size);
        assert!(glyph_vertex_buffer_size <= max_buffer_size);
        assert!(glyph_index_buffer_size <= max_buffer_size);
        assert!(blur_vertex_buffer_size <= max_buffer_size);
        assert!(blur_index_buffer_size <= max_buffer_size);

        Buffers::new(
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Rectangle vertex buffer",
                size: rectangle_vertex_buffer_size,
                usage: VERTEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Rectangle index buffer",
                size: rectangle_index_buffer_size,
                usage: INDEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Shadow vertex buffer",
                size: shadow_vertex_buffer_size,
                usage: VERTEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Shadow index buffer",
                size: shadow_index_buffer_size,
                usage: INDEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Glyph vertex buffer",
                size: glyph_vertex_buffer_size,
                usage: VERTEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Glyph index buffer",
                size: glyph_index_buffer_size,
                usage: INDEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Blur vertex buffer",
                size: blur_vertex_buffer_size,
                usage: VERTEX | COPY_DST,
            }),
            self.0.create_buffer(&buffer! {
                label: "[TextPipeline] Blur index buffer",
                size: blur_index_buffer_size,
                usage: INDEX | COPY_DST,
            }),
        )
    }

    pub fn mask_texture(&self, [width, height]: [u32; 2]) -> Texture {
        self.0.create_texture(&texture! {
            label: "[TextPipeline] Mask glyphs texture",
            size: [width, height],
            format: TextureFormat::R8Unorm,
            usage: TEXTURE_BINDING | COPY_DST,
        })
    }

    pub fn color_texture(&self, [width, height]: [u32; 2]) -> Texture {
        self.0.create_texture(&texture! {
            label: "[TextPipeline] Color glyphs texture",
            size: [width, height],
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TEXTURE_BINDING | COPY_DST,
        })
    }

    pub fn blur_textures(&self, [width, height]: [u32; 2]) -> [Texture; 2] {
        [
            self.0.create_texture(&texture! {
                label: "[TextPipeline] Blur ping texture",
                size: [width, height],
                format: TextureFormat::R8Unorm,
                usage: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_DST,
            }),
            self.0.create_texture(&texture! {
                label: "[TextPipeline] Blur pong texture",
                size: [width, height],
                format: TextureFormat::R8Unorm,
                usage: RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_DST,
            }),
        ]
    }

    pub fn bind_group_layout(&self) -> BindGroupLayout {
        self.0.create_bind_group_layout(&bind_group_layout! {
            label: "[TextPipeline] Bind group layout",
            entries: [
                // Mask texture
                { binding: 0, visibility: VERTEX | FRAGMENT, ty: Texture },
                // Color texture
                { binding: 1, visibility: VERTEX | FRAGMENT, ty: Texture },
                // Blur texture
                { binding: 2, visibility: VERTEX | FRAGMENT, ty: Texture },
                // Sampler
                { binding: 3, visibility: FRAGMENT, ty: Sampler(Filtering) },
            ],
        })
    }

    pub fn bind_groups(
        &self,
        bind_group_layout: &BindGroupLayout,
        mask_texture: &Texture,
        color_texture: &Texture,
        blur_ping_texture: &Texture,
        blur_pong_texture: &Texture,
    ) -> [BindGroup; 2] {
        [
            self.0.create_bind_group(&bind_group! {
                label: "[TextPipeline] Ping bind group",
                layout: bind_group_layout,
                entries: [
                    // Mask texture
                    { binding: 0, resource: Texture(mask_texture) },
                    // Color texture
                    { binding: 1, resource: Texture(color_texture) },
                    // Ping blur texture
                    { binding: 2, resource: Texture(blur_ping_texture) },
                    // Sampler
                    { binding: 3, resource: Sampler(self.0.create_sampler(&Default::default())) },
                ],
            }),
            self.0.create_bind_group(&bind_group! {
                label: "[TextPipeline] Pong bind group",
                layout: bind_group_layout,
                entries: [
                    // Mask texture
                    { binding: 0, resource: Texture(mask_texture) },
                    // Color texture
                    { binding: 1, resource: Texture(color_texture) },
                    // Pong blur texture
                    { binding: 2, resource: Texture(blur_pong_texture) },
                    // Sampler
                    { binding: 3, resource: Sampler(self.0.create_sampler(&Default::default())) },
                ],
            }),
        ]
    }

    pub fn pipelines(
        &self,
        config: &SurfaceConfiguration,
        bind_group_layout: &BindGroupLayout,
    ) -> [RenderPipeline; 5] {
        let config_target = [Some(ColorTargetState {
            format: config.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        let r8_target = [Some(ColorTargetState {
            format: TextureFormat::R8Unorm,
            blend: None,
            write_mask: ColorWrites::RED,
        })];
        let layout = self.0.create_pipeline_layout(&pipeline_layout! {
            label: "[TextPipeline] Pipeline layout",
            bind_group_layouts: [bind_group_layout],
            push_constant_ranges: [(VERTEX_FRAGMENT, 0..Constants::SIZE)],
        });
        let module = self.0.create_shader_module(include_wgsl!("shader.wgsl"));

        [
            self.0.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Rectangle pipeline",
                layout: layout,
                module: module,
                vertex: "rectangle_vertex",
                buffers: [RectangleVertex::buffer_layout()],
                fragment: "rectangle_fragment",
                targets: config_target,
                topology: TriangleList,
            }),
            self.0.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Shadow pipeline",
                layout: layout,
                module: module,
                vertex: "shadow_vertex",
                buffers: [ShadowVertex::buffer_layout()],
                fragment: "shadow_fragment",
                targets: r8_target,
                topology: TriangleList,
            }),
            self.0.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Glyph pipeline",
                layout: layout,
                module: module,
                vertex: "glyph_vertex",
                buffers: [GlyphVertex::buffer_layout()],
                fragment: "glyph_fragment",
                targets: config_target,
                topology: TriangleList,
            }),
            self.0.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Blur ping pipeline",
                layout: layout,
                module: module,
                vertex: "blur_ping_vertex",
                buffers: [BlurVertex::buffer_layout()],
                fragment: "blur_ping_fragment",
                targets: r8_target,
                topology: TriangleList,
            }),
            self.0.create_render_pipeline(&render_pipeline! {
                label: "[TextPipeline] Blur pong pipeline",
                layout: layout,
                module: module,
                vertex: "blur_pong_vertex",
                buffers: [BlurVertex::buffer_layout()],
                fragment: "blur_pong_fragment",
                targets: config_target,
                topology: TriangleList,
            }),
        ]
    }
}
