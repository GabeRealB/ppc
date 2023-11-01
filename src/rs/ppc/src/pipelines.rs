use crate::webgpu::*;

const NUM_SAMPLES: u32 = 4;

pub struct Pipelines {
    pub render_pipelines: RenderPipelines,
    pub compute_pipelines: ComputePipelines,
}

impl Pipelines {
    pub async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let render_pipelines = RenderPipelines::new(device, presentation_format).await;
        let compute_pipelines = ComputePipelines::new(device).await;

        Self {
            render_pipelines,
            compute_pipelines,
        }
    }
}

pub struct RenderPipelines {
    pub draw_lines: (BindGroupLayout, RenderPipeline),
    pub draw_value_lines: (BindGroupLayout, RenderPipeline, Sampler),
    pub draw_selections: (BindGroupLayout, RenderPipeline, Sampler),
    pub draw_curve_segments: (BindGroupLayout, RenderPipeline),
}

impl RenderPipelines {
    pub async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let draw_lines = Self::init_draw_lines_pipeline(device, presentation_format).await;
        let draw_value_lines =
            Self::init_draw_value_lines_pipeline(device, presentation_format).await;
        let draw_selections =
            Self::init_draw_selections_pipeline(device, presentation_format).await;
        let draw_curve_segments =
            Self::init_draw_curve_segments_pipeline(device, presentation_format).await;

        Self {
            draw_lines,
            draw_value_lines,
            draw_selections,
            draw_curve_segments,
        }
    }

    async fn init_draw_lines_pipeline(
        device: &Device,
        presentation_format: TextureFormat,
    ) -> (BindGroupLayout, RenderPipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("line render pipeline bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("line render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [bind_layout.clone()],
                    },
                )),
                vertex: VertexState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("lines vertex shader".into()),
                        code: include_str!("./shaders/line.vert.wgsl").into(),
                    }),
                },
                fragment: Some(FragmentState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("lines fragment shader".into()),
                        code: include_str!("./shaders/line.frag.wgsl").into(),
                    }),
                    targets: [FragmentStateTarget {
                        format: presentation_format,
                        blend: Some(FragmentStateBlend {
                            alpha: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                            color: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                        }),
                        write_mask: None,
                    }],
                }),
                multisample: Some(MultisampleState {
                    alpha_to_coverage_enabled: None,
                    count: Some(NUM_SAMPLES),
                    mask: None,
                }),
                primitive: Some(PrimitiveState {
                    cull_mode: None,
                    front_face: None,
                    strip_index_format: None,
                    topology: Some(PrimitiveTopology::TriangleList),
                    unclipped_depth: None,
                }),
            })
            .await;

        (bind_layout, pipeline)
    }

    async fn init_draw_value_lines_pipeline(
        device: &Device,
        presentation_format: TextureFormat,
    ) -> (BindGroupLayout, RenderPipeline, Sampler) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("value lines render pipeline bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Texture(TextureBindingLayout {
                        multisampled: None,
                        sample_type: Some(TextureSampleType::UnfilterableFloat),
                        view_dimension: Some(TextureViewDimension::D2),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Sampler(SamplerBindingLayout {
                        r#type: Some(SamplerBindingType::NonFiltering),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("value lines render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [bind_layout.clone()],
                    },
                )),
                vertex: VertexState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("value lines vertex shader".into()),
                        code: include_str!("./shaders/value_line.vert.wgsl").into(),
                    }),
                },
                fragment: Some(FragmentState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("value lines fragment shader".into()),
                        code: include_str!("./shaders/value_line.frag.wgsl").into(),
                    }),
                    targets: [FragmentStateTarget {
                        format: presentation_format,
                        blend: Some(FragmentStateBlend {
                            alpha: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                            color: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                        }),
                        write_mask: None,
                    }],
                }),
                multisample: Some(MultisampleState {
                    alpha_to_coverage_enabled: None,
                    count: Some(NUM_SAMPLES),
                    mask: None,
                }),
                primitive: Some(PrimitiveState {
                    cull_mode: None,
                    front_face: None,
                    strip_index_format: None,
                    topology: Some(PrimitiveTopology::TriangleList),
                    unclipped_depth: None,
                }),
            })
            .await;

        let sampler = device.create_sampler(SamplerDescriptor {
            label: Some("value lines render sampler".into()),
            address_mode_u: Some(AddressMode::Repeat),
            address_mode_v: Some(AddressMode::Repeat),
            address_mode_w: None,
            compare: None,
            lod_max_clamp: None,
            lod_min_clamp: None,
            mag_filter: None,
            max_anisotropy: None,
            min_filter: None,
            mipmap_filter: None,
        });

        (bind_layout, pipeline, sampler)
    }

    async fn init_draw_selections_pipeline(
        device: &Device,
        presentation_format: TextureFormat,
    ) -> (BindGroupLayout, RenderPipeline, Sampler) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("selection render pipeline bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Texture(TextureBindingLayout {
                        multisampled: None,
                        sample_type: Some(TextureSampleType::UnfilterableFloat),
                        view_dimension: Some(TextureViewDimension::D2Array),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Sampler(SamplerBindingLayout {
                        r#type: Some(SamplerBindingType::NonFiltering),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("selection render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [bind_layout.clone()],
                    },
                )),
                vertex: VertexState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("selection vertex shader".into()),
                        code: include_str!("./shaders/selection.vert.wgsl").into(),
                    }),
                },
                fragment: Some(FragmentState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("selection fragment shader".into()),
                        code: include_str!("./shaders/selection.frag.wgsl").into(),
                    }),
                    targets: [FragmentStateTarget {
                        format: presentation_format,
                        blend: Some(FragmentStateBlend {
                            alpha: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                            color: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                        }),
                        write_mask: None,
                    }],
                }),
                multisample: Some(MultisampleState {
                    alpha_to_coverage_enabled: None,
                    count: Some(NUM_SAMPLES),
                    mask: None,
                }),
                primitive: Some(PrimitiveState {
                    cull_mode: None,
                    front_face: None,
                    strip_index_format: None,
                    topology: Some(PrimitiveTopology::TriangleList),
                    unclipped_depth: None,
                }),
            })
            .await;

        let sampler = device.create_sampler(SamplerDescriptor {
            label: Some("selection render sampler".into()),
            address_mode_u: Some(AddressMode::Repeat),
            address_mode_v: Some(AddressMode::Repeat),
            address_mode_w: None,
            compare: None,
            lod_max_clamp: None,
            lod_min_clamp: None,
            mag_filter: None,
            max_anisotropy: None,
            min_filter: None,
            mipmap_filter: None,
        });

        (bind_layout, pipeline, sampler)
    }

    async fn init_draw_curve_segments_pipeline(
        device: &Device,
        presentation_format: TextureFormat,
    ) -> (BindGroupLayout, RenderPipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve segment render pipeline bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::Uniform),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::VERTEX,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("curve segment render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [bind_layout.clone()],
                    },
                )),
                vertex: VertexState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve segment vertex shader".into()),
                        code: include_str!("./shaders/curve_segment.vert.wgsl").into(),
                    }),
                },
                fragment: Some(FragmentState {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve segment fragment shader".into()),
                        code: include_str!("./shaders/curve_segment.frag.wgsl").into(),
                    }),
                    targets: [FragmentStateTarget {
                        format: presentation_format,
                        blend: Some(FragmentStateBlend {
                            alpha: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                            color: FragmentStateBlendEntry {
                                dst_factor: Some(BlendFactor::OneMinusSrcAlpha),
                                operation: Some(BlendOperation::Add),
                                src_factor: Some(BlendFactor::One),
                            },
                        }),
                        write_mask: None,
                    }],
                }),
                multisample: Some(MultisampleState {
                    alpha_to_coverage_enabled: None,
                    count: Some(NUM_SAMPLES),
                    mask: None,
                }),
                primitive: Some(PrimitiveState {
                    cull_mode: None,
                    front_face: None,
                    strip_index_format: None,
                    topology: Some(PrimitiveTopology::TriangleList),
                    unclipped_depth: None,
                }),
            })
            .await;

        (bind_layout, pipeline)
    }
}

pub struct ComputePipelines {
    pub sample_curves: (BindGroupLayout, ComputePipeline),
    pub create_curves: (BindGroupLayout, ComputePipeline),
    pub create_curves_segments: (BindGroupLayout, ComputePipeline),
    pub compute_probability: ProbabilityComputationPipeline,
    pub sample_color_scale: (BindGroupLayout, ComputePipeline),
    pub transform_color_scale: (BindGroupLayout, ComputePipeline),
}

pub struct ProbabilityComputationPipeline {
    pub apply_curve_bind_layout: BindGroupLayout,
    pub apply_curve_pipeline: ComputePipeline,
    pub reduce_bind_layout: BindGroupLayout,
    pub reduce_pipeline: ComputePipeline,
}

impl ComputePipelines {
    pub async fn new(device: &Device) -> Self {
        let sample_curves = Self::init_curve_sampling_pipeline(device).await;
        let create_curves = Self::init_curve_creation_pipeline(device).await;
        let create_curves_segments = Self::init_curve_segment_creation_pipeline(device).await;
        let compute_probability = Self::init_probability_computation_pipeline(device).await;
        let sample_color_scale = Self::init_color_scale_sampling_pipeline(device).await;
        let transform_color_scale = Self::init_color_scale_transformation_pipeline(device).await;

        Self {
            sample_curves,
            create_curves,
            create_curves_segments,
            compute_probability,
            sample_color_scale,
            transform_color_scale,
        }
    }

    async fn init_curve_sampling_pipeline(device: &Device) -> (BindGroupLayout, ComputePipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve spline sampling bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::StorageTexture(
                        StorageTextureBindingLayout {
                            access: Some(StorageTextureAccess::WriteOnly),
                            format: TextureFormat::R32float,
                            view_dimension: Some(TextureViewDimension::D2),
                        },
                    ),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("curve spline sampling compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("curve sampling pipeline layout".into()),
                        layouts: [bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve sampling compute shader".into()),
                        code: include_str!("./shaders/sample_spline.comp.wgsl").into(),
                    }),
                },
            })
            .await;

        (bind_layout, pipeline)
    }

    async fn init_curve_creation_pipeline(device: &Device) -> (BindGroupLayout, ComputePipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve creation bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Storage),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Texture(TextureBindingLayout {
                        multisampled: None,
                        sample_type: Some(TextureSampleType::UnfilterableFloat),
                        view_dimension: Some(TextureViewDimension::D2Array),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("curve creation compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("curve creation pipeline layout".into()),
                        layouts: [bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve creation compute shader".into()),
                        code: include_str!("./shaders/create_curves.comp.wgsl").into(),
                    }),
                },
            })
            .await;

        (bind_layout, pipeline)
    }

    async fn init_curve_segment_creation_pipeline(
        device: &Device,
    ) -> (BindGroupLayout, ComputePipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve segment creation bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Storage),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("curve segment creation compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("curve creation pipeline layout".into()),
                        layouts: [bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve segment creation compute shader".into()),
                        code: include_str!("./shaders/create_segment_curve.comp.wgsl").into(),
                    }),
                },
            })
            .await;

        (bind_layout, pipeline)
    }

    async fn init_probability_computation_pipeline(
        device: &Device,
    ) -> ProbabilityComputationPipeline {
        let application_bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve application bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Storage),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Texture(TextureBindingLayout {
                        multisampled: None,
                        sample_type: Some(TextureSampleType::UnfilterableFloat),
                        view_dimension: Some(TextureViewDimension::D2Array),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Uniform),
                    }),
                },
            ],
        });

        let application_pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("curve application compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("curve application pipeline layout".into()),
                        layouts: [application_bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve application compute shader".into()),
                        code: include_str!("./shaders/apply_curves.comp.wgsl").into(),
                    }),
                },
            })
            .await;

        let reduction_bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve application reduction bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Storage),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Uniform),
                    }),
                },
            ],
        });

        let reduction_pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("curve application reduction compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("curve application reduction pipeline layout".into()),
                        layouts: [reduction_bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("curve application reduction compute shader".into()),
                        code: include_str!("./shaders/reduce_probability.comp.wgsl").into(),
                    }),
                },
            })
            .await;

        ProbabilityComputationPipeline {
            apply_curve_bind_layout: application_bind_layout,
            apply_curve_pipeline: application_pipeline,
            reduce_bind_layout: reduction_bind_layout,
            reduce_pipeline: reduction_pipeline,
        }
    }

    async fn init_color_scale_sampling_pipeline(
        device: &Device,
    ) -> (BindGroupLayout, ComputePipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("color scale sampling bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::StorageTexture(
                        StorageTextureBindingLayout {
                            access: Some(StorageTextureAccess::WriteOnly),
                            format: TextureFormat::Rgba32float,
                            view_dimension: Some(TextureViewDimension::D2),
                        },
                    ),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("color scale sampling compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("color scale sampling pipeline layout".into()),
                        layouts: [bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("color scale sampling compute shader".into()),
                        code: include_str!("./shaders/color_scale/sample_color_scale.comp.wgsl")
                            .into(),
                    }),
                },
            })
            .await;

        (bind_layout, pipeline)
    }

    async fn init_color_scale_transformation_pipeline(
        device: &Device,
    ) -> (BindGroupLayout, ComputePipeline) {
        let bind_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("color scale transformation bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Texture(TextureBindingLayout {
                        multisampled: None,
                        sample_type: Some(TextureSampleType::UnfilterableFloat),
                        view_dimension: Some(TextureViewDimension::D2),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::StorageTexture(
                        StorageTextureBindingLayout {
                            access: Some(StorageTextureAccess::WriteOnly),
                            format: TextureFormat::Rgba32float,
                            view_dimension: Some(TextureViewDimension::D2),
                        },
                    ),
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Uniform),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("color scale transformation compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("color scale transformation pipeline layout".into()),
                        layouts: [bind_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("color scale transformation compute shader".into()),
                        code: include_str!("./shaders/color_scale/transform_color_scale.comp.wgsl")
                            .into(),
                    }),
                },
            })
            .await;

        (bind_layout, pipeline)
    }
}
