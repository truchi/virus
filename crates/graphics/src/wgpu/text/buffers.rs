use super::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Buffers                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Debug)]
pub struct Buffers<T> {
    vertices: Vec<T>,
    indices: Vec<Index>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl<T> Buffers<T> {
    pub fn with_capacity(vertex_buffer: Buffer, index_buffer: Buffer, capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(4 * capacity),
            indices: Vec::with_capacity(6 * capacity),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn push(&mut self, [top_left, top_right, bottom_left, bottom_right]: [T; 4])
    where
        T: Clone,
    {
        let i = self.vertices.len() as u32;

        self.vertices
            .extend_from_slice(&[top_left, top_right, bottom_left, bottom_right]);

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

    pub fn write(&self, queue: &Queue)
    where
        T: bytemuck::Pod,
    {
        if !self.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }
    }

    pub fn render<'pass>(
        &'pass self,
        render_pass: &mut RenderPass<'pass>,
        constants: &Constants,
        bind_group: &'pass BindGroup,
        pipeline: &'pass RenderPipeline,
    ) {
        if !self.is_empty() {
            let constants = constants.as_array();
            let constants = bytemuck::cast_slice(&constants);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_push_constants(Constants::STAGES, 0, constants);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);
            render_pass.draw_indexed(0..self.len(), 0, 0..1);
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}
