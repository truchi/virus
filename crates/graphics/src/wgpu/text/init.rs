use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Init                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct Init<'a>(pub &'a Device);

impl<'a> Init<'a> {
    pub fn buffers(
        &self,
        max_buffer_size: usize,
    ) -> (
        Buffers<RectangleVertex>,
        Buffers<ShadowVertex>,
        Buffers<GlyphVertex>,
        Buffers<BlurVertex>,
    ) {
        let rectangle_vertex_buffer_size = 4 * MAX_RECTANGLES * RECTANGLE_VERTEX_SIZE;
        let rectangle_index_buffer_size = 6 * MAX_RECTANGLES * INDEX_SIZE;
        let shadow_vertex_buffer_size = 4 * MAX_SHADOWS * SHADOW_VERTEX_SIZE;
        let shadow_index_buffer_size = 6 * MAX_SHADOWS * INDEX_SIZE;
        let glyph_vertex_buffer_size = 4 * MAX_GLYPHS * GLYPH_VERTEX_SIZE;
        let glyph_index_buffer_size = 6 * MAX_GLYPHS * INDEX_SIZE;
        let blur_vertex_buffer_size = 4 * MAX_BLURS * BLUR_VERTEX_SIZE;
        let blur_index_buffer_size = 6 * MAX_BLURS * INDEX_SIZE;
        assert!(rectangle_vertex_buffer_size <= max_buffer_size);
        assert!(rectangle_index_buffer_size <= max_buffer_size);
        assert!(shadow_vertex_buffer_size <= max_buffer_size);
        assert!(shadow_index_buffer_size <= max_buffer_size);
        assert!(glyph_vertex_buffer_size <= max_buffer_size);
        assert!(glyph_index_buffer_size <= max_buffer_size);
        assert!(blur_vertex_buffer_size <= max_buffer_size);
        assert!(blur_index_buffer_size <= max_buffer_size);

        (
            Buffers::with_capacity(
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
                MAX_RECTANGLES,
            ),
            Buffers::with_capacity(
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
                MAX_SHADOWS,
            ),
            Buffers::with_capacity(
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
                MAX_GLYPHS,
            ),
            Buffers::with_capacity(
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
                MAX_BLURS,
            ),
        )
    }

    pub fn atlases(&self, size: u32) -> (Texture, Texture) {
        (
            self.0.create_texture(&texture! {
                label: "[TextPipeline] Mask glyphs texture",
                size: [size, size],
                format: TextureFormat::R8Unorm,
                usage: TEXTURE_BINDING | COPY_DST,
            }),
            self.0.create_texture(&texture! {
                label: "[TextPipeline] Color glyphs texture",
                size: [size, size],
                format: TextureFormat::Rgba8Unorm,
                usage: TEXTURE_BINDING | COPY_DST,
            }),
        )
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
