// TODO ShaderStages: $(|)+ and remove ::empty()

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         BufferDescriptor                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! buffer {
    (
        label: $label:expr,
        size: $size:expr,
        usage: $($usage:ident)|*
        $(,)?
    ) => {
        BufferDescriptor {
            label: Some($label),
            size: $size as BufferAddress,
            usage: $(BufferUsages::$usage|)* BufferUsages::empty(),
            mapped_at_creation: false,
        }
    };
}

pub(crate) use buffer;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         TextureDescriptor                                      //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! texture {
    (
        label: $label:expr,
        size: $size:expr,
        format: $format:expr,
        usage: $($usage:ident)|*
        $(,)?
    ) => {{
        let [width, height] = $size;

        TextureDescriptor {
            label: Some($label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: $format,
            usage: $(TextureUsages::$usage|)* TextureUsages::empty(),
            view_formats: &[],
        }
    }};
}

pub(crate) use texture;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       BindGroupLayoutEntry                                     //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! bind_group_layout_entry {
    (
        binding: $binding:expr,
        visibility: $($visibility:ident)|*,
        ty: Uniform
        $(,)?
    ) => {
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: $(ShaderStages::$visibility|)* ShaderStages::empty(),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    };
    (
        binding: $binding:expr,
        visibility: $($visibility:ident)|*,
        ty: Texture
        $(,)?
    ) => {
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: $(ShaderStages::$visibility|)* ShaderStages::empty(),
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        }
    };
    (
        binding: $binding:expr,
        visibility: $($visibility:ident)|*,
        ty: StorageTexture($format:expr)
        $(,)?
    ) => {
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: $(ShaderStages::$visibility|)* ShaderStages::empty(),
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: $format,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }
    };
    (
        binding: $binding:expr,
        visibility: $($visibility:ident)|*,
        ty: Sampler($sampler_binding_type:ident)
        $(,)?
    ) => {
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: $(ShaderStages::$visibility|)* ShaderStages::empty(),
            ty: BindingType::Sampler(SamplerBindingType::$sampler_binding_type),
            count: None,
        }
    };
}

pub(crate) use bind_group_layout_entry;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                     BindGroupLayoutDescriptor                                  //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! bind_group_layout {
    (
        label: $label:expr,
        entries: [$({$($entry:tt)*}),* $(,)?]
        $(,)?
    ) => {
        BindGroupLayoutDescriptor {
            label: Some($label),
            entries: &[$(bind_group_layout_entry!($($entry)*)),*],
        }
    };
}

pub(crate) use bind_group_layout;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          BindGroupEntry                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! bind_group_entry {
    (
        binding: $binding:expr,
        resource: Buffer($buffer:expr)
        $(,)?
    ) => {
        BindGroupEntry {
            binding: $binding,
            resource: BindingResource::Buffer($buffer.as_entire_buffer_binding()),
        }
    };
    (
        binding: $binding:expr,
        resource: Texture($texture:expr)
        $(,)?
    ) => {
        BindGroupEntry {
            binding: $binding,
            resource: BindingResource::TextureView(&$texture.create_view(&Default::default())),
        }
    };
    (
        binding: $binding:expr,
        resource: TextureView($view:expr)
        $(,)?
    ) => {
        BindGroupEntry {
            binding: $binding,
            resource: BindingResource::TextureView(&$view),
        }
    };
    (
        binding: $binding:expr,
        resource: Sampler($sampler:expr)
        $(,)?
    ) => {
        BindGroupEntry {
            binding: $binding,
            resource: BindingResource::Sampler(&$sampler),
        }
    };
}

pub(crate) use bind_group_entry;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        BindGroupDescriptor                                     //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! bind_group {
    (
        label: $label:expr,
        layout: $layout:expr,
        entries: [$({$($entry:tt)*}),* $(,)?]
        $(,)?
    ) => {
        BindGroupDescriptor {
            label: Some($label),
            layout: &$layout,
            entries: &[$(bind_group_entry!($($entry)*)),*],
        }
    };
}

pub(crate) use bind_group;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                     PipelineLayoutDescriptor                                   //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! pipeline_layout {
    (
        label: $label:expr,
        bind_group_layouts: [$($bind_group_layout:expr),*]
        $(,)?
    ) => {
        PipelineLayoutDescriptor {
            label: Some($label),
            bind_group_layouts: &[$(&$bind_group_layout),*],
            push_constant_ranges: &[],
        }
    };
    (
        label: $label:expr,
        bind_group_layouts: [$($bind_group_layout:expr),*],
        push_constant_ranges: [$(($($stage:ident)|*, $range:expr)),*]
        $(,)?
    ) => {
        PipelineLayoutDescriptor {
            label: Some($label),
            bind_group_layouts: &[$(&$bind_group_layout),*],
            push_constant_ranges: &[$(PushConstantRange {
                stages: $(ShaderStages::$stage|)* ShaderStages::empty(),
                range: $range,
            }),*],
        }
    };
}

pub(crate) use pipeline_layout;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                     RenderPipelineDescriptor                                   //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! render_pipeline {
    (
        label: $label:expr,
        layout: $layout:expr,
        module: $module:expr,
        vertex: $vertex:expr,
        buffers: $buffers:expr,
        fragment: $fragment:expr,
        targets: $targets:expr,
        topology: $topology:ident
        $(,)?
    ) => {{
        let module = &$module;

        RenderPipelineDescriptor {
            label: Some($label),
            layout: Some(&$layout),
            vertex: VertexState {
                module,
                entry_point: $vertex,
                buffers: &$buffers,
            },
            fragment: Some(FragmentState {
                module,
                entry_point: $fragment,
                targets: &$targets,
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::$topology,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        }
    }};
}

pub(crate) use render_pipeline;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       RenderPassDescriptor                                     //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

macro_rules! render_pass {
    (
        label: $label:expr,
        view: $view:expr,
        load: $load:ident$(($color:ident))?,
        store: $store:expr
        $(,)?
    ) => {
        RenderPassDescriptor {
            label: Some($label),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &$view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::$load$((Color::$color))?,
                    store: $store,
                },
            })],
            depth_stencil_attachment: None,
        }
    };
}

pub(crate) use render_pass;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Tests                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[cfg(test)]
mod tests {
    use wgpu::*;

    #[test]
    #[ignore = "Compile-time test"]
    fn test() {
        fn _test(
            buffer: Buffer,
            texture: Texture,
            sampler: Sampler,
            bind_group_layout: BindGroupLayout,
            pipeline_layout: PipelineLayout,
            module: ShaderModule,
            buffer_layout: VertexBufferLayout,
            target: ColorTargetState,
            view: TextureView,
        ) {
            // Buffer
            let _ = buffer! {
                label: "Buffer",
                size: 12,
                usage: UNIFORM | COPY_DST,
            };

            // Texture
            let _ = texture! {
                label: "Texture",
                size: [12, 12],
                format: TextureFormat::R8Uint,
                usage: TEXTURE_BINDING | COPY_DST,
            };

            // Bind group layout entry
            let _ = bind_group_layout_entry! {
                binding: 0,
                visibility: FRAGMENT,
                ty: Uniform,
            };
            let _ = bind_group_layout_entry! {
                binding: 0,
                visibility: FRAGMENT,
                ty: Texture,
            };
            let _ = bind_group_layout_entry! {
                binding: 0,
                visibility: FRAGMENT,
                ty: Sampler(Filtering),
            };

            // Bind group layout
            let _ = bind_group_layout! {
                label: "Bind group layout",
                entries: [{
                    binding: 0,
                    visibility: FRAGMENT,
                    ty: Uniform,
                }, {
                    binding: 1,
                    visibility: FRAGMENT,
                    ty: Texture,
                }],
            };

            // Bind group entry
            let _ = bind_group_entry! {
                binding: 0,
                resource: Buffer(buffer),
            };
            let _ = bind_group_entry! {
                binding: 0,
                resource: Texture(texture),
            };
            let _ = bind_group_entry! {
                binding: 0,
                resource: TextureView(texture.create_view(&Default::default())),
            };
            let _ = bind_group_entry! {
                binding: 0,
                resource: Sampler(sampler),
            };

            // Bind group
            let _ = bind_group! {
                label: "Bind group",
                layout: bind_group_layout,
                entries: [{
                    binding: 0,
                    resource: Buffer(buffer),
                },
                {
                    binding: 1,
                    resource: Texture(texture),
                },
                {
                    binding: 2,
                    resource: TextureView(texture.create_view(&Default::default())),
                },
                {
                    binding: 3,
                    resource: Sampler(sampler),
                }],
            };

            // Pipeline layout
            let _ = pipeline_layout! {
                label: "Pipeline layout",
                bind_group_layouts: [bind_group_layout],
            };
            let _ = pipeline_layout! {
                label: "Pipeline layout",
                bind_group_layouts: [bind_group_layout],
                push_constant_ranges: [(VERTEX | FRAGMENT, 0..128)],
            };

            // Render pipeline
            let _ = render_pipeline! {
                label: "Render pipeline",
                layout: pipeline_layout,
                module: module,
                vertex: "vertex",
                buffers: [buffer_layout],
                fragment: "fragment",
                targets: [Some(target)],
                topology: TriangleList,
            };

            // Render pass
            let _ = render_pass! {
                label: "Render pipeline",
                view: view,
                load: Load,
                store: true,
            };
            let _ = render_pass! {
                label: "Render pipeline",
                view: view,
                load: Clear(BLACK),
                store: true,
            };
        }
    }
}
