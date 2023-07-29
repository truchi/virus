use super::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
struct Layer {
    rectangle: LayerInner<RectangleVertex>,
    shadow: LayerInner<ShadowVertex>,
    glyph: LayerInner<GlyphVertex>,
    blur: LayerInner<BlurVertex>,
}

#[derive(Debug)]
struct LayerInner<T> {
    vertices: Vec<T>,
    indices: Vec<Index>,
    start: u32,
}

impl<T> Default for LayerInner<T> {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
            start: 0,
        }
    }
}

impl<T> LayerInner<T> {
    fn push(&mut self, quad: Quad<T>) {
        let i = self.vertices.len() as u32;

        self.vertices.extend([
            quad.top_left,
            quad.top_right,
            quad.bottom_left,
            quad.bottom_right,
        ]);

        let top_left = i;
        let top_right = i + 1;
        let bottom_left = i + 2;
        let bottom_right = i + 3;

        self.indices.extend_from_slice(&[
            top_left,
            bottom_right,
            top_right,
            top_left,
            bottom_left,
            bottom_right,
        ]);
    }

    pub fn write(
        &mut self,
        queue: &Queue,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        vertices_offset: &mut BufferAddress,
        indices_offset: &mut BufferAddress,
        start: &mut u32,
    ) where
        T: bytemuck::NoUninit,
    {
        let vertices = bytemuck::cast_slice(&self.vertices);
        let indices = bytemuck::cast_slice(&self.indices);

        queue.write_buffer(vertex_buffer, *vertices_offset, vertices);
        queue.write_buffer(index_buffer, *indices_offset, indices);

        self.start = *start;

        *vertices_offset += vertices.len() as BufferAddress;
        *indices_offset += indices.len() as BufferAddress;
        *start += self.indices.len() as u32;
    }

    pub fn render<'pass>(
        &'pass self,
        render_pass: &mut RenderPass<'pass>,
        constants: &Constants,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
        vertex_buffer: &'pass Buffer,
        index_buffer: &'pass Buffer,
    ) {
        if !self.indices.is_empty() {
            let constants = constants.as_array();
            let constants = bytemuck::cast_slice(&constants);
            let indices = self.start..self.start + self.indices.len() as u32;

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_push_constants(Constants::STAGES, 0, constants);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), INDEX_FORMAT);
            render_pass.draw_indexed(indices, 0, 0..1);
        }
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.start = 0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Buffers                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Buffers {
    rectangle_vertices: Buffer,
    rectangle_indices: Buffer,
    shadow_vertices: Buffer,
    shadow_indices: Buffer,
    glyph_vertices: Buffer,
    glyph_indices: Buffer,
    blur_vertices: Buffer,
    blur_indices: Buffer,
    layers: HashMap<u32, Layer>,
}

impl Buffers {
    pub fn new(
        rectangle_vertices: Buffer,
        rectangle_indices: Buffer,
        shadow_vertices: Buffer,
        shadow_indices: Buffer,
        glyph_vertices: Buffer,
        glyph_indices: Buffer,
        blur_vertices: Buffer,
        blur_indices: Buffer,
    ) -> Self {
        Self {
            rectangle_vertices,
            rectangle_indices,
            shadow_vertices,
            shadow_indices,
            glyph_vertices,
            glyph_indices,
            blur_vertices,
            blur_indices,
            layers: Default::default(),
        }
    }

    pub fn push_rectangle(&mut self, layer: u32, quad: Quad<RectangleVertex>) {
        self.layers.entry(layer).or_default().rectangle.push(quad);
    }

    pub fn push_shadow(&mut self, layer: u32, quad: Quad<ShadowVertex>) {
        self.layers.entry(layer).or_default().shadow.push(quad);
    }

    pub fn push_glyph(&mut self, layer: u32, quad: Quad<GlyphVertex>) {
        self.layers.entry(layer).or_default().glyph.push(quad);
    }

    pub fn push_blur(&mut self, layer: u32, quad: Quad<BlurVertex>) {
        self.layers.entry(layer).or_default().blur.push(quad);
    }

    pub fn pre_render(&mut self, queue: &Queue) {
        let mut rectangle_vertices_offset = 0;
        let mut rectangle_indices_offset = 0;
        let mut rectangle_start = 0;

        let mut shadow_vertices_offset = 0;
        let mut shadow_indices_offset = 0;
        let mut shadow_start = 0;

        let mut glyph_vertices_offset = 0;
        let mut glyph_indices_offset = 0;
        let mut glyph_start = 0;

        let mut blur_vertices_offset = 0;
        let mut blur_indices_offset = 0;
        let mut blur_start = 0;

        for layer in self.layers.values_mut() {
            layer.rectangle.write(
                queue,
                &self.rectangle_vertices,
                &self.rectangle_indices,
                &mut rectangle_vertices_offset,
                &mut rectangle_indices_offset,
                &mut rectangle_start,
            );
            layer.shadow.write(
                queue,
                &self.shadow_vertices,
                &self.shadow_indices,
                &mut shadow_vertices_offset,
                &mut shadow_indices_offset,
                &mut shadow_start,
            );
            layer.glyph.write(
                queue,
                &self.glyph_vertices,
                &self.glyph_indices,
                &mut glyph_vertices_offset,
                &mut glyph_indices_offset,
                &mut glyph_start,
            );
            layer.blur.write(
                queue,
                &self.blur_vertices,
                &self.blur_indices,
                &mut blur_vertices_offset,
                &mut blur_indices_offset,
                &mut blur_start,
            );
        }
    }

    pub fn render_rectangles<'pass>(
        &'pass self,
        layer: u32,
        render_pass: &mut RenderPass<'pass>,
        constants: &Constants,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
    ) {
        self.layers.get(&layer).map(|layer| {
            layer.rectangle.render(
                render_pass,
                constants,
                bind_group,
                pipeline,
                &self.rectangle_vertices,
                &self.rectangle_indices,
            )
        });
    }

    pub fn render_shadows<'pass>(
        &'pass self,
        layer: u32,
        render_pass: &mut RenderPass<'pass>,
        constants: &Constants,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
    ) {
        self.layers.get(&layer).map(|layer| {
            layer.shadow.render(
                render_pass,
                constants,
                bind_group,
                pipeline,
                &self.shadow_vertices,
                &self.shadow_indices,
            )
        });
    }

    pub fn render_glyphs<'pass>(
        &'pass self,
        layer: u32,
        render_pass: &mut RenderPass<'pass>,
        constants: &Constants,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
    ) {
        self.layers.get(&layer).map(|layer| {
            layer.glyph.render(
                render_pass,
                constants,
                bind_group,
                pipeline,
                &self.glyph_vertices,
                &self.glyph_indices,
            )
        });
    }

    pub fn render_blurs<'pass>(
        &'pass self,
        layer: u32,
        render_pass: &mut RenderPass<'pass>,
        constants: &Constants,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
    ) {
        self.layers.get(&layer).map(|layer| {
            layer.blur.render(
                render_pass,
                constants,
                bind_group,
                pipeline,
                &self.blur_vertices,
                &self.blur_indices,
            )
        });
    }

    pub fn clear(&mut self) {
        for layer in self.layers.values_mut() {
            layer.rectangle.clear();
            layer.shadow.clear();
            layer.glyph.clear();
            layer.blur.clear();
        }
    }
}
