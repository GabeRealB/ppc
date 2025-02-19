use crate::buffers;
use crate::webgpu::*;

const NUM_SAMPLES: u32 = 4;

pub struct Pipelines {
    render_pipelines: RenderPipelines,
    compute_pipelines: ComputePipelines,
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

    pub fn render(&self) -> &RenderPipelines {
        &self.render_pipelines
    }

    pub fn compute(&self) -> &ComputePipelines {
        &self.compute_pipelines
    }
}

pub struct RenderPipelines {
    axis_lines: AxisLinesRenderPipeline,
    data_lines: DataLinesRenderPipeline,
    curve_lines: CurveLinesRenderPipeline,
    selections: SelectionsRenderPipeline,
    curve_segments: CurveSegmentsRenderPipeline,
    color_bar: ColorBarRenderPipeline,
}

impl RenderPipelines {
    pub async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        Self {
            axis_lines: AxisLinesRenderPipeline::new(device, presentation_format).await,
            data_lines: DataLinesRenderPipeline::new(device, presentation_format).await,
            curve_lines: CurveLinesRenderPipeline::new(device, presentation_format).await,
            selections: SelectionsRenderPipeline::new(device, presentation_format).await,
            curve_segments: CurveSegmentsRenderPipeline::new(device, presentation_format).await,
            color_bar: ColorBarRenderPipeline::new(device, presentation_format).await,
        }
    }

    pub fn axis_lines(&self) -> &AxisLinesRenderPipeline {
        &self.axis_lines
    }

    pub fn data_lines(&self) -> &DataLinesRenderPipeline {
        &self.data_lines
    }

    pub fn curve_lines(&self) -> &CurveLinesRenderPipeline {
        &self.curve_lines
    }

    pub fn selections(&self) -> &SelectionsRenderPipeline {
        &self.selections
    }

    pub fn curve_segments(&self) -> &CurveSegmentsRenderPipeline {
        &self.curve_segments
    }

    pub fn color_bar(&self) -> &ColorBarRenderPipeline {
        &self.color_bar
    }
}

pub struct AxisLinesRenderPipeline {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl AxisLinesRenderPipeline {
    async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("axis lines shader".into()),
            code: include_str!("./shaders/axis_lines.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("axis lines render pipeline bind group layout".into()),
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
                label: Some("axis lines render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [layout.clone()],
                    },
                )),
                depth_stencil: Some(DepthStencilState {
                    depth_bias: None,
                    depth_bias_clamp: None,
                    depth_bias_slope_scale: None,
                    depth_compare: CompareFunction::Always,
                    depth_write_enabled: false,
                    format: buffers::DepthTexture::DEPTH_FORMAT,
                }),
                vertex: VertexState {
                    entry_point: "vertex_main",
                    module: shader_module.clone(),
                },
                fragment: Some(FragmentState {
                    entry_point: "fragment_main",
                    module: shader_module,
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

        Self { layout, pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        matrices: &buffers::MatricesBuffer,
        config: &buffers::AxesConfigBuffer,
        axes: &buffers::AxesBuffer,
        axis_lines: &buffers::AxisLinesBuffer,
        viewport_start: (f32, f32),
        viewport_size: (f32, f32),
        device: &Device,
        render_pass: &RenderPassEncoder,
    ) {
        let num_lines = axis_lines.len();
        if num_lines == 0 {
            return;
        }

        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("axis lines bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: matrices.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: config.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: axes.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: axis_lines.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.layout.clone(),
        });

        let (x, y) = viewport_start;
        let (width, height) = viewport_size;

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group);
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.draw_with_instance_count(6, num_lines);
    }
}

pub struct DataLinesRenderPipeline {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl DataLinesRenderPipeline {
    async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("data lines shader".into()),
            code: include_str!("./shaders/data_lines.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("data lines render pipeline bind group layout".into()),
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
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        r#type: Some(BufferBindingType::ReadOnlyStorage),
                        ..Default::default()
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
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
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("data lines render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [layout.clone()],
                    },
                )),
                depth_stencil: Some(DepthStencilState {
                    depth_bias: None,
                    depth_bias_clamp: None,
                    depth_bias_slope_scale: None,
                    depth_compare: CompareFunction::LessEqual,
                    depth_write_enabled: true,
                    format: buffers::DepthTexture::DEPTH_FORMAT,
                }),
                vertex: VertexState {
                    entry_point: "vertex_main",
                    module: shader_module.clone(),
                },
                fragment: Some(FragmentState {
                    entry_point: "fragment_main",
                    module: shader_module,
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

        Self { layout, pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        matrices: &buffers::MatricesBuffer,
        config: &buffers::DataConfigBuffer,
        axes: &buffers::AxesBuffer,
        data_lines: &buffers::DataLinesBuffer,
        color_values: &buffers::ColorValuesBuffer,
        probabilities: &buffers::ProbabilitiesBuffer,
        color_scale: &buffers::ColorScaleTexture,
        viewport_start: (f32, f32),
        viewport_size: (f32, f32),
        device: &Device,
        render_pass: &RenderPassEncoder,
    ) {
        let num_lines = data_lines.len();
        if num_lines == 0 {
            return;
        }

        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("data lines bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: matrices.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: config.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: axes.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: data_lines.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: color_values.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: probabilities.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindGroupEntryResource::TextureView(color_scale.view()),
                },
            ],
            layout: self.layout.clone(),
        });

        let (x, y) = viewport_start;
        let (width, height) = viewport_size;

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group);
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.draw_with_instance_count(6, num_lines);
    }
}

pub struct CurveLinesRenderPipeline {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl CurveLinesRenderPipeline {
    async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("curve lines shader".into()),
            code: include_str!("./shaders/curve_lines.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve lines render pipeline bind group layout".into()),
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
                label: Some("curve lines render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [layout.clone()],
                    },
                )),
                depth_stencil: Some(DepthStencilState {
                    depth_bias: None,
                    depth_bias_clamp: None,
                    depth_bias_slope_scale: None,
                    depth_compare: CompareFunction::Always,
                    depth_write_enabled: false,
                    format: buffers::DepthTexture::DEPTH_FORMAT,
                }),
                vertex: VertexState {
                    entry_point: "vertex_main",
                    module: shader_module.clone(),
                },
                fragment: Some(FragmentState {
                    entry_point: "fragment_main",
                    module: shader_module,
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

        Self { layout, pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        matrices: &buffers::MatricesBuffer,
        config: &buffers::CurvesConfigBuffer,
        axes: &buffers::AxesBuffer,
        curve_lines: &buffers::CurveLinesInfoBuffer,
        viewport_start: (f32, f32),
        viewport_size: (f32, f32),
        device: &Device,
        render_pass: &RenderPassEncoder,
    ) {
        let num_lines = curve_lines.len();
        if num_lines == 0 {
            return;
        }

        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("curve lines bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: matrices.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: config.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: axes.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: curve_lines.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.layout.clone(),
        });

        let (x, y) = viewport_start;
        let (width, height) = viewport_size;

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group);
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.draw_with_instance_count(6, num_lines);
    }
}

pub struct SelectionsRenderPipeline {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl SelectionsRenderPipeline {
    async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("selections shader".into()),
            code: include_str!("./shaders/selections.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("selections render pipeline bind group layout".into()),
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
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("selections render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [layout.clone()],
                    },
                )),
                depth_stencil: Some(DepthStencilState {
                    depth_bias: None,
                    depth_bias_clamp: None,
                    depth_bias_slope_scale: None,
                    depth_compare: CompareFunction::Always,
                    depth_write_enabled: false,
                    format: buffers::DepthTexture::DEPTH_FORMAT,
                }),
                vertex: VertexState {
                    entry_point: "vertex_main",
                    module: shader_module.clone(),
                },
                fragment: Some(FragmentState {
                    entry_point: "fragment_main",
                    module: shader_module,
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

        Self { layout, pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        matrices: &buffers::MatricesBuffer,
        config: &buffers::SelectionsConfigBuffer,
        axes: &buffers::AxesBuffer,
        selection_infos: &buffers::SelectionLinesBuffer,
        colors: &buffers::LabelColorBuffer,
        probability_samples: &buffers::ProbabilitySampleTexture,
        viewport_start: (f32, f32),
        viewport_size: (f32, f32),
        device: &Device,
        render_pass: &RenderPassEncoder,
    ) {
        let num_selections = selection_infos.len();
        if num_selections == 0 {
            return;
        }

        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("selections bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: matrices.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: config.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: axes.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: selection_infos.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: colors.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindGroupEntryResource::TextureView(probability_samples.array_view()),
                },
            ],
            layout: self.layout.clone(),
        });

        let (x, y) = viewport_start;
        let (width, height) = viewport_size;

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group);
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.draw_with_instance_count(6, num_selections);
    }
}

pub struct CurveSegmentsRenderPipeline {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl CurveSegmentsRenderPipeline {
    async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("curve segments shader".into()),
            code: include_str!("./shaders/curve_segments.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("curve segments render pipeline bind group layout".into()),
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
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("curve segments render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [layout.clone()],
                    },
                )),
                depth_stencil: Some(DepthStencilState {
                    depth_bias: None,
                    depth_bias_clamp: None,
                    depth_bias_slope_scale: None,
                    depth_compare: CompareFunction::Always,
                    depth_write_enabled: false,
                    format: buffers::DepthTexture::DEPTH_FORMAT,
                }),
                vertex: VertexState {
                    entry_point: "vertex_main",
                    module: shader_module.clone(),
                },
                fragment: Some(FragmentState {
                    entry_point: "fragment_main",
                    module: shader_module,
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

        Self { layout, pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        label_idx: usize,
        active_label_idx: usize,
        min_curve_t: f32,
        matrices: &buffers::MatricesBuffer,
        axes: &buffers::AxesBuffer,
        curve_lines: &buffers::CurveLinesInfoBuffer,
        label_colors: &buffers::LabelColorBuffer,
        viewport_start: (f32, f32),
        viewport_size: (f32, f32),
        device: &Device,
        render_pass: &RenderPassEncoder,
    ) {
        let num_lines = curve_lines.len();
        if num_lines == 0 {
            return;
        }

        let config = buffers::CurveSegmentConfigBuffer::new(
            device,
            buffers::CurveSegmentConfig {
                label: label_idx as u32,
                active_label: active_label_idx as u32,
                min_curve_t,
            },
        );

        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("curve segments bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: matrices.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: config.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: axes.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: curve_lines.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: label_colors.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.layout.clone(),
        });

        let (x, y) = viewport_start;
        let (width, height) = viewport_size;

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group);
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.draw_with_instance_count(6, num_lines);
    }
}

pub struct ColorBarRenderPipeline {
    layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl ColorBarRenderPipeline {
    async fn new(device: &Device, presentation_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("color bar shader".into()),
            code: include_str!("./shaders/color_bar.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("color bar rendering bind group layout".into()),
            entries: [
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Texture(TextureBindingLayout {
                        multisampled: None,
                        sample_type: Some(TextureSampleType::UnfilterableFloat),
                        view_dimension: Some(TextureViewDimension::D2),
                    }),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    resource: BindGroupLayoutEntryResource::Buffer(BufferBindingLayout {
                        has_dynamic_offset: None,
                        min_binding_size: None,
                        r#type: Some(BufferBindingType::Uniform),
                    }),
                },
            ],
        });

        let pipeline = device
            .create_render_pipeline_async(RenderPipelineDescriptor {
                label: Some("color bar render pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: None,
                        layouts: [layout.clone()],
                    },
                )),
                depth_stencil: Some(DepthStencilState {
                    depth_bias: None,
                    depth_bias_clamp: None,
                    depth_bias_slope_scale: None,
                    depth_compare: CompareFunction::Always,
                    depth_write_enabled: false,
                    format: buffers::DepthTexture::DEPTH_FORMAT,
                }),
                vertex: VertexState {
                    entry_point: "vertex_main",
                    module: shader_module.clone(),
                },
                fragment: Some(FragmentState {
                    entry_point: "fragment_main",
                    module: shader_module,
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

        Self { layout, pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        color_scale: &buffers::ColorScaleTexture,
        color_scale_bounds: &buffers::ColorScaleBoundsBuffer,
        viewport_start: (f32, f32),
        viewport_size: (f32, f32),
        device: &Device,
        render_pass: &RenderPassEncoder,
    ) {
        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("color bar bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::TextureView(color_scale.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: color_scale_bounds.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.layout.clone(),
        });

        let (x, y) = viewport_start;
        let (width, height) = viewport_size;

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group);
        render_pass.set_viewport(x, y, width, height, 0.0, 1.0);
        render_pass.draw(6);
    }
}

pub struct ComputePipelines {
    pub create_curves: (BindGroupLayout, ComputePipeline),
    pub compute_probability: ProbabilityComputationPipeline,
    #[allow(unused)]
    pub transform_color_scale: (BindGroupLayout, ComputePipeline),
    curve_spline_sampling: ProbabilityCurveSplineSamplingComputePipeline,
    //
    //
    color_scale_sampling: ColorScaleSamplingComputePipeline,
}

pub struct ProbabilityComputationPipeline {
    pub apply_curve_bind_layout: BindGroupLayout,
    pub apply_curve_pipeline: ComputePipeline,
    pub reduce_bind_layout: BindGroupLayout,
    pub reduce_pipeline: ComputePipeline,
}

impl ComputePipelines {
    pub async fn new(device: &Device) -> Self {
        let create_curves = Self::init_curve_creation_pipeline(device).await;
        let compute_probability = Self::init_probability_computation_pipeline(device).await;
        let transform_color_scale = Self::init_color_scale_transformation_pipeline(device).await;

        Self {
            create_curves,
            compute_probability,
            transform_color_scale,
            curve_spline_sampling: ProbabilityCurveSplineSamplingComputePipeline::new(device).await,
            color_scale_sampling: ColorScaleSamplingComputePipeline::new(device).await,
        }
    }

    pub fn curve_spline_sampling(&self) -> &ProbabilityCurveSplineSamplingComputePipeline {
        &self.curve_spline_sampling
    }

    pub fn color_scale_sampling(&self) -> &ColorScaleSamplingComputePipeline {
        &self.color_scale_sampling
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

pub struct ProbabilityCurveSplineSamplingComputePipeline {
    layout: BindGroupLayout,
    pipeline: ComputePipeline,
}

impl ProbabilityCurveSplineSamplingComputePipeline {
    async fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("probability curve sampling compute shader".into()),
            code: include_str!("./shaders/probability_curve/sample_spline.comp.wgsl").into(),
        });

        let layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
            label: Some("probability curve spline sampling bind group layout".into()),
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
                label: Some("probability curve spline sampling compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("curve sampling pipeline layout".into()),
                        layouts: [layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: shader_module,
                },
            })
            .await;

        Self { layout, pipeline }
    }

    pub fn dispatch(
        &self,
        axis_idx: usize,
        probability_texture: &buffers::ProbabilitySampleTexture,
        spline_segments: &buffers::SplineSegmentsBuffer,
        device: &Device,
        encoder: &CommandEncoder,
    ) {
        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("probability curve spline sampling bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::TextureView(
                        probability_texture.axis_view(axis_idx),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: spline_segments.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.layout.clone(),
        });

        const NUM_WORKGROUPS: u32 =
            buffers::ProbabilitySampleTexture::PROBABILITY_CURVE_RESOLUTION.div_ceil(64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[NUM_WORKGROUPS]);
        pass.end();
    }
}

pub struct ColorScaleSamplingComputePipeline {
    sampling_layout: BindGroupLayout,
    sampling_pipeline: ComputePipeline,
    transformation_layout: BindGroupLayout,
    transformation_pipeline: ComputePipeline,
}

impl ColorScaleSamplingComputePipeline {
    async fn new(device: &Device) -> Self {
        let sampling_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("color scale sampling shader module".into()),
            code: include_str!("./shaders/color_scale/sample_color_scale.comp.wgsl").into(),
        });

        let transformation_shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("color scale transformation shader module".into()),
            code: include_str!("./shaders/color_scale/transform_color_scale.comp.wgsl").into(),
        });

        let sampling_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
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

        let transformation_layout = device.create_bind_group_layout(BindGroupLayoutDescriptor {
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

        let sampling_pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("color scale sampling compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("color scale sampling pipeline layout".into()),
                        layouts: [sampling_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: sampling_shader_module,
                },
            })
            .await;

        let transformation_pipeline = device
            .create_compute_pipeline_async(ComputePipelineDescriptor {
                label: Some("color scale transformation compute pipeline".into()),
                layout: PipelineLayoutType::Layout(device.create_pipeline_layout(
                    PipelineLayoutDescriptor {
                        label: Some("color scale transformation pipeline layout".into()),
                        layouts: [transformation_layout.clone()],
                    },
                )),
                compute: ProgrammableStage {
                    entry_point: "main",
                    module: transformation_shader_module,
                },
            })
            .await;

        Self {
            sampling_layout,
            sampling_pipeline,
            transformation_layout,
            transformation_pipeline,
        }
    }

    pub fn dispatch(
        &self,
        color_space: crate::wasm_bridge::ColorSpace,
        color_scale: &mut buffers::ColorScaleTexture,
        color_scale_elements: &buffers::ColorScaleElementBuffer,
        device: &Device,
        encoder: &CommandEncoder,
    ) {
        const NUM_WORKGROUPS: u32 =
            buffers::ColorScaleTexture::COLOR_SCALE_RESOLUTION.div_ceil(64) as u32;

        let color_scale_view = color_scale.view();
        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("color scale sampling bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::TextureView(color_scale_view.clone()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: color_scale_elements.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.sampling_layout.clone(),
        });

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(&self.sampling_pipeline);
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[NUM_WORKGROUPS]);
        pass.end();

        // We don't need to transform the color space, since it is already correct.
        if color_space == crate::wasm_bridge::ColorSpace::Xyz {
            return;
        }

        let tmp_color_scale = buffers::ColorScaleTexture::new(device);
        let color_space: u32 = match color_space {
            crate::wasm_bridge::ColorSpace::SRgb => 0,
            crate::wasm_bridge::ColorSpace::Xyz => 1,
            crate::wasm_bridge::ColorSpace::CieLab => 2,
            crate::wasm_bridge::ColorSpace::CieLch => 3,
        };
        let color_space_buffer = device.create_buffer(BufferDescriptor {
            label: Some("color space buffer".into()),
            size: std::mem::size_of::<u32>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        device
            .queue()
            .write_buffer_single(&color_space_buffer, 0, &color_space);

        let bind_group = device.create_bind_group(BindGroupDescriptor {
            label: Some("color scale transformation bind group".into()),
            entries: [
                BindGroupEntry {
                    binding: 0,
                    resource: BindGroupEntryResource::TextureView(color_scale_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindGroupEntryResource::TextureView(tmp_color_scale.view()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindGroupEntryResource::Buffer(BufferBinding {
                        buffer: color_space_buffer,
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.transformation_layout.clone(),
        });

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(&self.transformation_pipeline);
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[NUM_WORKGROUPS]);
        pass.end();

        *color_scale = tmp_color_scale;
    }
}
