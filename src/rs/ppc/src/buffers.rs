use std::{borrow::Cow, mem::MaybeUninit};

use crate::{
    webgpu::{
        Buffer, BufferDescriptor, BufferUsage, Device, Texture, TextureDescriptor,
        TextureDimension, TextureFormat, TextureUsage, TextureView, TextureViewDescriptor,
        TextureViewDimension,
    },
    wgsl::{HostSharable, Matrix4x4, Vec2, Vec3, Vec4},
};

/// Buffer containing the MVP matrices.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Matrices {
    pub mv_matrix: Matrix4x4<f32>,
    pub p_matrix: Matrix4x4<f32>,
}

impl Matrices {
    pub fn new(num_visible_axes: usize) -> Self {
        let mv_matrix = Matrix4x4::from_columns_array([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.5, 0.0, 0.0, 1.0],
        ]);
        let p_matrix = Matrix4x4::from_columns_array([
            [2.0 / num_visible_axes as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
        ]);

        Self {
            mv_matrix,
            p_matrix,
        }
    }
}

unsafe impl HostSharable for Matrices {}

/// Buffer layout of the axes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Axis {
    pub expanded_val: f32,
    pub center_x: f32,
    pub position_x: Vec2<f32>,
    pub range_y: Vec2<f32>,
}

unsafe impl HostSharable for Axis {}

/// Buffer layout of a label color pair.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LabelColor {
    pub color_high: Vec4<f32>,
    pub color_low: Vec4<f32>,
}

unsafe impl HostSharable for LabelColor {}

/// Config for rendering the axes lines.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AxesConfig {
    pub line_width: Vec2<f32>,
    pub color: Vec3<f32>,
}

unsafe impl HostSharable for AxesConfig {}

/// Representation of an axis line.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AxisLineInfo {
    pub axis: u32,
    pub axis_position: f32,
    pub min_expanded_val: f32,
}

impl AxisLineInfo {
    pub const LEFT: f32 = -1.0;
    pub const CENTER: f32 = 0.0;
    pub const RIGHT: f32 = 1.0;
}

unsafe impl HostSharable for AxisLineInfo {}

/// Value line rendering config buffer layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ValueLineConfig {
    pub line_width: Vec2<f32>,
    pub selection_bounds: Vec2<f32>,
    pub color_probabilities: u32,
    pub unselected_color: Vec4<f32>,
}

unsafe impl HostSharable for ValueLineConfig {}

/// Representation of an entry for the value lines buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ValueLine {
    pub curve_idx: u32,
    pub start_axis: u32,
    pub start_value: f32,
    pub end_axis: u32,
    pub end_value: f32,
}

unsafe impl HostSharable for ValueLine {}

/// Config for rendering probability curves.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CurvesConfig {
    pub line_width: Vec2<f32>,
    pub color: Vec3<f32>,
}

unsafe impl HostSharable for CurvesConfig {}

/// Representation of a probability curve line segment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CurveLineInfo {
    pub x_t_values: Vec2<f32>,
    pub y_t_values: Vec2<f32>,
    pub axis: u32,
}

unsafe impl HostSharable for CurveLineInfo {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CurveSegmentConfig {
    pub label: u32,
    pub active_label: u32,
    pub min_curve_t: f32,
}

unsafe impl HostSharable for CurveSegmentConfig {}

#[derive(Debug, Clone)]
pub struct CurveSegmentConfigBuffer {
    buffer: Buffer,
}

impl CurveSegmentConfigBuffer {
    pub fn new(device: &Device, config: CurveSegmentConfig) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("curve segment config buffer")),
            size: std::mem::size_of_val(&config),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        device.queue().write_buffer_single(&buffer, 0, &config);

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

/// Selection line rendering config buffer layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SelectionConfig {
    pub line_width: Vec2<f32>,
    pub high_color: Vec3<f32>,
    pub low_color: Vec3<f32>,
}

unsafe impl HostSharable for SelectionConfig {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SelectionLineInfo {
    pub axis: u32,
    pub use_color: u32,
    pub use_left: u32,
    pub offset_x: f32,
    pub color_idx: u32,
    pub range: Vec2<f32>,
}

unsafe impl HostSharable for SelectionLineInfo {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ColorScaleElement {
    pub t: f32,
    pub color: Vec4<f32>,
}

unsafe impl HostSharable for ColorScaleElement {}

#[derive(Debug, Clone)]
pub struct ColorScaleElementBuffer {
    buffer: Buffer,
}

impl ColorScaleElementBuffer {
    pub fn new(device: &Device, elements: &[ColorScaleElement]) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("color scale element buffer")),
            size: std::mem::size_of_val(elements),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        device.queue().write_buffer(&buffer, 0, elements);

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ColorScaleBounds {
    pub start: f32,
    pub end: f32,
}

unsafe impl HostSharable for ColorScaleBounds {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SplineSegment {
    pub coefficients: Vec4<f32>,
    pub bounds: Vec2<f32>,
    pub t_range: Vec2<f32>,
}

unsafe impl HostSharable for SplineSegment {}

#[derive(Debug, Clone)]
pub struct SplineSegmentsBuffer {
    buffer: Buffer,
}

impl SplineSegmentsBuffer {
    pub fn new(device: &Device, elements: &[SplineSegment]) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("spline segment buffer")),
            size: std::mem::size_of_val(elements),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        device.queue().write_buffer(&buffer, 0, elements);

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

/// Collection of buffers.
#[derive(Debug, Clone)]
pub struct Buffers {
    shared: SharedBuffers,
    axes: AxesBuffers,
    datums: DatumsBuffers,
    curves: CurvesBuffers,
    selections: SelectionsBuffers,
}

impl Buffers {
    pub fn new(device: &Device) -> Self {
        Self {
            shared: SharedBuffers::new(device),
            axes: AxesBuffers::new(device),
            datums: DatumsBuffers::new(device),
            curves: CurvesBuffers::new(device),
            selections: SelectionsBuffers::new(device),
        }
    }

    pub fn shared(&self) -> &SharedBuffers {
        &self.shared
    }

    pub fn shared_mut(&mut self) -> &mut SharedBuffers {
        &mut self.shared
    }

    pub fn axes(&self) -> &AxesBuffers {
        &self.axes
    }

    pub fn axes_mut(&mut self) -> &mut AxesBuffers {
        &mut self.axes
    }

    pub fn datums(&self) -> &DatumsBuffers {
        &self.datums
    }

    pub fn datums_mut(&mut self) -> &mut DatumsBuffers {
        &mut self.datums
    }

    pub fn curves(&self) -> &CurvesBuffers {
        &self.curves
    }

    pub fn curves_mut(&mut self) -> &mut CurvesBuffers {
        &mut self.curves
    }

    pub fn selections(&self) -> &SelectionsBuffers {
        &self.selections
    }

    pub fn selections_mut(&mut self) -> &mut SelectionsBuffers {
        &mut self.selections
    }
}

/// Collection of shared buffers.
#[derive(Debug, Clone)]
pub struct SharedBuffers {
    matrix: MatricesBuffer,
    axes: AxesBuffer,
    colors: LabelColorBuffer,
    color_scale: ColorScaleTexture,
    color_scale_bounds: ColorScaleBoundsBuffer,
}

impl SharedBuffers {
    fn new(device: &Device) -> Self {
        Self {
            matrix: MatricesBuffer::new(device),
            axes: AxesBuffer::new(device),
            colors: LabelColorBuffer::new(device),
            color_scale: ColorScaleTexture::new(device),
            color_scale_bounds: ColorScaleBoundsBuffer::new(device),
        }
    }

    pub fn matrices(&self) -> &MatricesBuffer {
        &self.matrix
    }

    pub fn matrices_mut(&mut self) -> &mut MatricesBuffer {
        &mut self.matrix
    }

    pub fn axes(&self) -> &AxesBuffer {
        &self.axes
    }

    pub fn axes_mut(&mut self) -> &mut AxesBuffer {
        &mut self.axes
    }

    pub fn label_colors(&self) -> &LabelColorBuffer {
        &self.colors
    }

    pub fn label_colors_mut(&mut self) -> &mut LabelColorBuffer {
        &mut self.colors
    }

    pub fn color_scale(&self) -> &ColorScaleTexture {
        &self.color_scale
    }

    pub fn color_scale_mut(&mut self) -> &mut ColorScaleTexture {
        &mut self.color_scale
    }

    pub fn color_scale_bounds(&self) -> &ColorScaleBoundsBuffer {
        &self.color_scale_bounds
    }

    pub fn color_scale_bounds_mut(&mut self) -> &mut ColorScaleBoundsBuffer {
        &mut self.color_scale_bounds
    }
}

/// A uniform buffer containing a [`Matrices`] instance.
#[derive(Debug, Clone)]
pub struct MatricesBuffer {
    buffer: Buffer,
}

impl MatricesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("matrix buffer")),
            size: std::mem::size_of::<Matrices>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, matrices: &Matrices) {
        device
            .queue()
            .write_buffer_single(&self.buffer, 0, matrices);
    }
}

/// A storage buffer of [`Axis`].
#[derive(Debug, Clone)]
pub struct AxesBuffer {
    buffer: Buffer,
}

impl AxesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("axes buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<Axis>()
    }

    pub fn update(&mut self, device: &Device, axes: &[MaybeUninit<Axis>]) {
        if self.len() != axes.len() {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("axes buffer")),
                size: std::mem::size_of_val(axes),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }

        device.queue().write_buffer(&self.buffer, 0, axes);
    }
}

/// A storage buffer of [`LabelColor`].
#[derive(Debug, Clone)]
pub struct LabelColorBuffer {
    buffer: Buffer,
}

impl LabelColorBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("label color buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<LabelColor>()
    }

    pub fn update(&mut self, device: &Device, colors: &[LabelColor]) {
        if self.len() != colors.len() {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("label color buffer")),
                size: std::mem::size_of_val(colors),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }

        device.queue().write_buffer(&self.buffer, 0, colors);
    }
}

/// A texture for storing a sampled color scale.
#[derive(Debug, Clone)]
pub struct ColorScaleTexture {
    texture: Texture,
}

impl ColorScaleTexture {
    pub const COLOR_SCALE_RESOLUTION: usize = 2048;

    pub fn new(device: &Device) -> Self {
        let texture = device.create_texture(TextureDescriptor::<2, 0> {
            label: Some(Cow::Borrowed("color scale texture")),
            dimension: Some(TextureDimension::D2),
            format: TextureFormat::Rgba32float,
            mip_level_count: None,
            sample_count: None,
            size: [Self::COLOR_SCALE_RESOLUTION, 1],
            usage: TextureUsage::STORAGE_BINDING | TextureUsage::TEXTURE_BINDING,
            view_formats: None,
        });

        Self { texture }
    }

    pub fn view(&self) -> TextureView {
        self.texture.create_view(Some(TextureViewDescriptor {
            label: Some(Cow::Borrowed("color scale texture view")),
            array_layer_count: None,
            aspect: None,
            base_array_layer: None,
            base_mip_level: None,
            dimension: Some(TextureViewDimension::D2),
            format: None,
            mip_level_count: None,
        }))
    }
}

/// A buffer containing the bounds of the color scale.
#[derive(Debug, Clone)]
pub struct ColorScaleBoundsBuffer {
    buffer: Buffer,
}

impl ColorScaleBoundsBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("color scale bounds buffer")),
            size: std::mem::size_of::<ColorScaleBounds>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        device.queue().write_buffer_single(
            &buffer,
            0,
            &ColorScaleBounds {
                start: 0.0,
                end: 1.0,
            },
        );

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, bounds: &ColorScaleBounds) {
        device.queue().write_buffer_single(&self.buffer, 0, bounds);
    }
}

/// Collection of buffers for drawing axes lines.
#[derive(Debug, Clone)]
pub struct AxesBuffers {
    config: AxesConfigBuffer,
    lines: AxisLinesBuffer,
}

impl AxesBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: AxesConfigBuffer::new(device),
            lines: AxisLinesBuffer::new(device),
        }
    }

    pub fn config(&self) -> &AxesConfigBuffer {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AxesConfigBuffer {
        &mut self.config
    }

    pub fn lines(&self) -> &AxisLinesBuffer {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut AxisLinesBuffer {
        &mut self.lines
    }
}

/// A uniform buffer containing a [`AxesConfig`] instance.
#[derive(Debug, Clone)]
pub struct AxesConfigBuffer {
    buffer: Buffer,
}

impl AxesConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("axes config buffer")),
            size: std::mem::size_of::<AxesConfig>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, config: &AxesConfig) {
        device.queue().write_buffer_single(&self.buffer, 0, config);
    }
}

/// A storage buffer containing the information required to draw the axis lines.
#[derive(Debug, Clone)]
pub struct AxisLinesBuffer {
    buffer: Buffer,
}

impl AxisLinesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("axis lines buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<AxisLineInfo>()
    }

    pub fn update(&mut self, device: &Device, lines: &[MaybeUninit<AxisLineInfo>]) {
        if self.len() != lines.len() {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("axis lines buffer")),
                size: std::mem::size_of_val(lines),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }

        device.queue().write_buffer(&self.buffer, 0, lines)
    }
}

/// Collection of buffers for drawing values.
#[derive(Debug, Clone)]
pub struct DatumsBuffers {
    config: DatumsConfigBuffer,
    lines: DatumLinesBuffer,
    datums: DatumBuffer,
    color_values: ColorValuesBuffer,
    probabilities: Vec<ProbabilitiesBuffer>,
}

impl DatumsBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: DatumsConfigBuffer::new(device),
            lines: DatumLinesBuffer::new(device),
            datums: DatumBuffer::new(device),
            color_values: ColorValuesBuffer::new(device),
            probabilities: vec![],
        }
    }

    pub fn config(&self) -> &DatumsConfigBuffer {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut DatumsConfigBuffer {
        &mut self.config
    }

    pub fn lines(&self) -> &DatumLinesBuffer {
        &self.lines
    }

    pub fn lines_mut(&mut self) -> &mut DatumLinesBuffer {
        &mut self.lines
    }

    pub fn datums(&self) -> &DatumBuffer {
        &self.datums
    }

    pub fn datums_mut(&mut self) -> &mut DatumBuffer {
        &mut self.datums
    }

    pub fn color_values(&self) -> &ColorValuesBuffer {
        &self.color_values
    }

    pub fn color_values_mut(&mut self) -> &mut ColorValuesBuffer {
        &mut self.color_values
    }

    pub fn probabilities(&self, label_idx: usize) -> &ProbabilitiesBuffer {
        &self.probabilities[label_idx]
    }

    pub fn probabilities_mut(&mut self, label_idx: usize) -> &mut ProbabilitiesBuffer {
        &mut self.probabilities[label_idx]
    }

    pub fn push_label(&mut self, device: &Device) {
        self.probabilities.push(ProbabilitiesBuffer::new(device))
    }

    pub fn remove_label(&mut self, label_idx: usize) {
        self.probabilities.remove(label_idx);
    }
}

/// A uniform buffer storing an instance of an [`ValueLineConfig`].
#[derive(Debug, Clone)]
pub struct DatumsConfigBuffer {
    buffer: Buffer,
}

impl DatumsConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("datums config buffer")),
            size: std::mem::size_of::<ValueLineConfig>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, config: &ValueLineConfig) {
        device.queue().write_buffer_single(&self.buffer, 0, config);
    }
}

/// A storage buffer containing the information required to draw the value lines.
#[derive(Debug, Clone)]
pub struct DatumLinesBuffer {
    buffer: Buffer,
}

impl DatumLinesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("datum lines buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<ValueLine>()
    }

    pub fn update(&mut self, device: &Device, lines: &[ValueLine]) {
        if self.len() != lines.len() {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("datum lines buffer")),
                size: std::mem::size_of_val(lines),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }

        device.queue().write_buffer(&self.buffer, 0, lines)
    }
}

#[derive(Debug, Clone)]
pub struct DatumBuffer {
    buffer: Buffer,
}

impl DatumBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("datums buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<f32>()
    }

    pub fn resize(&mut self, device: &Device, num_datums: usize, num_axes: usize) {
        if self.len() != num_datums * num_axes {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("datums buffer")),
                size: num_datums * num_axes * std::mem::size_of::<f32>(),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }
    }

    pub fn update(&self, device: &Device, datums: &[f32], index: usize) {
        let buffer_offset = (index * std::mem::size_of_val(datums)) as u32;
        device
            .queue()
            .write_buffer(&self.buffer, buffer_offset, datums)
    }
}

#[derive(Debug, Clone)]
pub struct ColorValuesBuffer {
    buffer: Buffer,
}

impl ColorValuesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("datum color values buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<f32>()
    }

    pub fn resize(&mut self, device: &Device, num_datums: usize) {
        if self.len() != num_datums {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("datum color values buffer")),
                size: num_datums * std::mem::size_of::<f32>(),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }
    }

    pub fn update(&self, device: &Device, values: &[f32]) {
        device.queue().write_buffer(&self.buffer, 0, values)
    }
}

#[derive(Debug, Clone)]
pub struct ProbabilitiesBuffer {
    buffer: Buffer,
}

impl ProbabilitiesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("probabilities buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn empty(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("probabilities buffer")),
            size: std::mem::size_of::<f32>(),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn size(&self) -> usize {
        self.buffer.size()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<f32>()
    }

    pub fn set_len(&mut self, device: &Device, len: usize) {
        self.buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("probabilities buffer")),
            size: len * std::mem::size_of::<f32>(),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            mapped_at_creation: None,
        });
    }
}

/// Collection of buffers for drawing the probability curves.
#[derive(Debug, Clone)]
pub struct CurvesBuffers {
    config: CurvesConfigBuffer,
    sample_textures: Vec<ProbabilitySampleTexture>,
    lines: Vec<CurveLinesInfoBuffer>,
}

impl CurvesBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: CurvesConfigBuffer::new(device),
            sample_textures: vec![],
            lines: vec![],
        }
    }

    pub fn config(&self) -> &CurvesConfigBuffer {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut CurvesConfigBuffer {
        &mut self.config
    }

    pub fn sample_texture(&self, label_idx: usize) -> &ProbabilitySampleTexture {
        &self.sample_textures[label_idx]
    }

    pub fn sample_texture_mut(&mut self, label_idx: usize) -> &mut ProbabilitySampleTexture {
        &mut self.sample_textures[label_idx]
    }

    pub fn lines(&self, label_idx: usize) -> &CurveLinesInfoBuffer {
        &self.lines[label_idx]
    }

    pub fn lines_mut(&mut self, label_idx: usize) -> &mut CurveLinesInfoBuffer {
        &mut self.lines[label_idx]
    }

    pub fn remove_label(&mut self, index: usize) {
        self.sample_textures.remove(index);
        self.lines.remove(index);
    }

    pub fn push_label(&mut self, device: &Device) {
        self.sample_textures
            .push(ProbabilitySampleTexture::new(device));
        self.lines.push(CurveLinesInfoBuffer::new(device));
    }
}

/// A uniform buffer containing a [`CurvesConfig`] instance.
#[derive(Debug, Clone)]
pub struct CurvesConfigBuffer {
    buffer: Buffer,
}

impl CurvesConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("curves config buffer")),
            size: std::mem::size_of::<CurvesConfig>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, config: &CurvesConfig) {
        device.queue().write_buffer_single(&self.buffer, 0, config);
    }
}

#[derive(Debug, Clone)]
pub struct ProbabilitySampleTexture {
    texture: Texture,
}

impl ProbabilitySampleTexture {
    pub const PROBABILITY_CURVE_RESOLUTION: usize = 1028;

    fn new(device: &Device) -> Self {
        let texture = device.create_texture(TextureDescriptor::<'_, 3, 2> {
            label: Some(Cow::Borrowed("probability curve sample texture")),
            dimension: Some(TextureDimension::D2),
            format: TextureFormat::R32float,
            mip_level_count: None,
            sample_count: None,
            size: [Self::PROBABILITY_CURVE_RESOLUTION, 1, 1],
            usage: TextureUsage::STORAGE_BINDING | TextureUsage::TEXTURE_BINDING,
            view_formats: None,
        });

        Self { texture }
    }

    pub fn array_view(&self) -> TextureView {
        self.texture.create_view(Some(TextureViewDescriptor {
            label: Some(Cow::Borrowed("probability curve sample texture view")),
            array_layer_count: None,
            aspect: None,
            base_array_layer: None,
            base_mip_level: None,
            dimension: Some(TextureViewDimension::D2Array),
            format: None,
            mip_level_count: None,
        }))
    }

    pub fn axis_view(&self, axis: usize) -> TextureView {
        self.texture.create_view(Some(TextureViewDescriptor {
            label: Some(format!("axis {axis} probability curve sample texture view").into()),
            array_layer_count: Some(1),
            aspect: None,
            base_array_layer: Some(axis as u32),
            base_mip_level: None,
            dimension: Some(TextureViewDimension::D2),
            format: None,
            mip_level_count: None,
        }))
    }

    pub fn set_num_curves(&mut self, device: &Device, num_curves: usize) {
        let num_layers = num_curves.max(1);
        if self.texture.depth_or_array_layers() as usize == num_layers {
            return;
        }

        self.texture = device.create_texture(TextureDescriptor::<'_, 3, 2> {
            label: Some(Cow::Borrowed("probability curve sample texture")),
            dimension: Some(TextureDimension::D2),
            format: TextureFormat::R32float,
            mip_level_count: None,
            sample_count: None,
            size: [Self::PROBABILITY_CURVE_RESOLUTION, 1, num_layers],
            usage: TextureUsage::STORAGE_BINDING | TextureUsage::TEXTURE_BINDING,
            view_formats: None,
        });
    }
}

#[derive(Debug, Clone)]
pub struct CurveLinesInfoBuffer {
    buffer: Buffer,
}

impl CurveLinesInfoBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("curve lines info buffer")),
            size: 0,
            usage: BufferUsage::STORAGE,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<CurveLineInfo>()
    }

    pub fn set_len(&mut self, device: &Device, len: usize) {
        self.buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("curve lines info buffer")),
            size: len * std::mem::size_of::<CurveLineInfo>(),
            usage: BufferUsage::STORAGE,
            mapped_at_creation: None,
        });
    }
}

/// Collection of buffers for drawing the selections.
#[derive(Debug, Clone)]
pub struct SelectionsBuffers {
    config: SelectionsConfigBuffer,
    lines: Vec<SelectionLinesBuffer>,
}

impl SelectionsBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: SelectionsConfigBuffer::new(device),
            lines: vec![],
        }
    }

    pub fn config(&self) -> &SelectionsConfigBuffer {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SelectionsConfigBuffer {
        &mut self.config
    }

    pub fn lines(&self, active_label_idx: usize) -> &SelectionLinesBuffer {
        &self.lines[active_label_idx]
    }

    pub fn lines_mut(&mut self, active_label_idx: usize) -> &mut SelectionLinesBuffer {
        &mut self.lines[active_label_idx]
    }

    pub fn remove_label(&mut self, index: usize) {
        self.lines.remove(index);
    }

    pub fn push_label(&mut self, device: &Device) {
        self.lines.push(SelectionLinesBuffer::new(device));
    }
}

#[derive(Debug, Clone)]
pub struct SelectionsConfigBuffer {
    buffer: Buffer,
}

impl SelectionsConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("selection lines config buffer")),
            size: std::mem::size_of::<SelectionConfig>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, config: &SelectionConfig) {
        device.queue().write_buffer_single(&self.buffer, 0, config);
    }
}

#[derive(Debug, Clone)]
pub struct SelectionLinesBuffer {
    buffer: Buffer,
}

impl SelectionLinesBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("selection lines buffer")),
            size: 0,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.size() / std::mem::size_of::<SelectionLineInfo>()
    }

    pub fn update(&mut self, device: &Device, lines: &[SelectionLineInfo]) {
        if self.len() != lines.len() {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("selection lines buffer")),
                size: std::mem::size_of_val(lines),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }

        device.queue().write_buffer(&self.buffer, 0, lines)
    }
}
