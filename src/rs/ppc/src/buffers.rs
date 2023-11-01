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

/// Line rendering config buffer layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LineConfig {
    pub line_width: Vec2<f32>,
    pub line_type: u32,
    pub color_mode: u32,
    pub color: Vec3<f32>,
}

unsafe impl HostSharable for LineConfig {}

/// Representation of an entry for the line info buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LineInfo {
    pub min_expanded_val: f32,
    pub start_args: Vec2<f32>,
    pub end_args: Vec2<f32>,
    pub offset_start: Vec2<f32>,
    pub offset_end: Vec2<f32>,
}

unsafe impl HostSharable for LineInfo {}

/// Value line rendering config buffer layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ValueLineConfig {
    pub line_width: Vec2<f32>,
    pub selection_threshold: f32,
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
    pub use_low_color: u32,
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
    pub general: GeneralBuffers,
    pub axes: AxesBuffers,
    pub values: ValuesDrawingBuffers,
    pub curves: CurvesBuffers,
    pub selections: SelectionsBuffers,
}

impl Buffers {
    pub fn new(device: &Device) -> Self {
        Self {
            general: GeneralBuffers::new(device),
            axes: AxesBuffers::new(device),
            values: ValuesDrawingBuffers::new(device),
            curves: CurvesBuffers::new(device),
            selections: SelectionsBuffers::new(device),
        }
    }
}

/// Collection of shared buffers.
#[derive(Debug, Clone)]
pub struct GeneralBuffers {
    pub matrix: MatrixBuffer,
    pub axes: AxesBuffer,
    pub colors: LabelColorBuffer,
}

impl GeneralBuffers {
    fn new(device: &Device) -> Self {
        Self {
            matrix: MatrixBuffer::new(device),
            axes: AxesBuffer::new(device),
            colors: LabelColorBuffer::new(device),
        }
    }
}

/// A uniform buffer containing a [`Matrices`] instance.
#[derive(Debug, Clone)]
pub struct MatrixBuffer {
    buffer: Buffer,
}

impl MatrixBuffer {
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

/// Collection of buffers for drawing axes lines.
#[derive(Debug, Clone)]
pub struct AxesBuffers {
    pub config: AxesConfigBuffer,
    pub lines: AxesLineBuffer,
}

impl AxesBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: AxesConfigBuffer::new(device),
            lines: AxesLineBuffer::new(device),
        }
    }
}

/// A uniform buffer containing a [`LineConfig`] instance.
#[derive(Debug, Clone)]
pub struct AxesConfigBuffer {
    buffer: Buffer,
}

impl AxesConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("axes config buffer")),
            size: std::mem::size_of::<LineConfig>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, config: &LineConfig) {
        device.queue().write_buffer_single(&self.buffer, 0, config);
    }
}

/// A storage buffer containing the information required to draw the axis lines.
#[derive(Debug, Clone)]
pub struct AxesLineBuffer {
    buffer: Buffer,
}

impl AxesLineBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("axes line buffer")),
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
        self.buffer.size() / std::mem::size_of::<LineInfo>()
    }

    pub fn update(&mut self, device: &Device, lines: &[MaybeUninit<LineInfo>]) {
        if self.len() != lines.len() {
            self.buffer.destroy();
            self.buffer = device.create_buffer(BufferDescriptor {
                label: Some(Cow::Borrowed("axes line buffer")),
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
pub struct ValuesDrawingBuffers {
    pub config: ValueLineConfigBuffer,
    pub lines: ValueLineBuffer,
    pub datums: ValueDatumBuffer,
    pub color_values: ColorValuesBuffer,
    pub probabilities: ProbabilitiesBuffer,
    pub color_scale: ColorScaleTexture,
}

impl ValuesDrawingBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: ValueLineConfigBuffer::new(device),
            lines: ValueLineBuffer::new(device),
            datums: ValueDatumBuffer::new(device),
            color_values: ColorValuesBuffer::new(device),
            probabilities: ProbabilitiesBuffer::new(device),
            color_scale: ColorScaleTexture::new(device),
        }
    }
}

/// A uniform buffer storing an instance of an [`ValueLineConfig`].
#[derive(Debug, Clone)]
pub struct ValueLineConfigBuffer {
    buffer: Buffer,
}

impl ValueLineConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("value lines config buffer")),
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
pub struct ValueLineBuffer {
    buffer: Buffer,
}

impl ValueLineBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("value lines buffer")),
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
                label: Some(Cow::Borrowed("value lines buffer")),
                size: std::mem::size_of_val(lines),
                usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
        }

        device.queue().write_buffer(&self.buffer, 0, lines)
    }
}

#[derive(Debug, Clone)]
pub struct ValueDatumBuffer {
    buffer: Buffer,
}

impl ValueDatumBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("value datums buffer")),
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
                label: Some(Cow::Borrowed("value datums buffer")),
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

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn size(&self) -> usize {
        self.buffer.size()
    }

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

/// Collection of buffers for drawing the probability curves.
#[derive(Debug, Clone)]
pub struct CurvesBuffers {
    pub config: CurvesConfigBuffer,
    pub sample_textures: Vec<ProbabilitySampleTexture>,
    pub lines: Vec<CurveLinesInfoBuffer>,
}

impl CurvesBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: CurvesConfigBuffer::new(device),
            sample_textures: vec![ProbabilitySampleTexture::new(device)],
            lines: vec![CurveLinesInfoBuffer::new(device)],
        }
    }

    pub fn remove_idx(&mut self, index: usize) {
        self.sample_textures.remove(index);
        self.lines.remove(index);
    }

    pub fn push(&mut self, device: &Device) {
        self.sample_textures
            .push(ProbabilitySampleTexture::new(device));
        self.lines.push(CurveLinesInfoBuffer::new(device));
    }
}

/// A uniform buffer containing a [`LineConfig`] instance.
#[derive(Debug, Clone)]
pub struct CurvesConfigBuffer {
    buffer: Buffer,
}

impl CurvesConfigBuffer {
    fn new(device: &Device) -> Self {
        let buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("curves config buffer")),
            size: std::mem::size_of::<LineConfig>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn update(&mut self, device: &Device, config: &LineConfig) {
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
        self.buffer.size() / std::mem::size_of::<LineInfo>()
    }

    pub fn set_len(&mut self, device: &Device, len: usize) {
        self.buffer = device.create_buffer(BufferDescriptor {
            label: Some(Cow::Borrowed("curve lines info buffer")),
            size: len * std::mem::size_of::<LineInfo>(),
            usage: BufferUsage::STORAGE,
            mapped_at_creation: None,
        });
    }
}

/// Collection of buffers for drawing the selections.
#[derive(Debug, Clone)]
pub struct SelectionsBuffers {
    pub config: SelectionsConfigBuffer,
    lines: Vec<SelectionLinesBuffer>,
}

impl SelectionsBuffers {
    fn new(device: &Device) -> Self {
        Self {
            config: SelectionsConfigBuffer::new(device),
            lines: vec![SelectionLinesBuffer::new(device)],
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
