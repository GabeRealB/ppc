use std::{borrow::Cow, mem::MaybeUninit};

use crate::{
    webgpu::{Buffer, BufferDescriptor, BufferUsage, Device, Sampler, Texture},
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
    pub fn new(world_width: f32) -> Self {
        let mv_matrix = Matrix4x4::from_columns_array([
            [1.0 / world_width, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let p_matrix = Matrix4x4::from_columns_array([
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, -2.0, 0.0],
            [-1.0, -1.0, -1.0, 1.0],
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
pub struct SelectionInfo {
    pub axis: u32,
    pub use_color: u32,
    pub use_left: u32,
    pub offset_x: u32,
    pub range: Vec2<f32>,
    pub color: Vec3<f32>,
}

unsafe impl HostSharable for SelectionInfo {}

/// Collection of buffers.
#[derive(Debug, Clone)]
pub struct Buffers {
    pub general: GeneralBuffers,
    pub axes: AxesDrawingBuffers,
}

impl Buffers {
    pub fn new(device: &Device) -> Self {
        Self {
            general: GeneralBuffers::new(device),
            axes: AxesDrawingBuffers::new(device),
        }
    }
}

/// Collection of shared buffers.
#[derive(Debug, Clone)]
pub struct GeneralBuffers {
    pub matrix: MatrixBuffer,
    pub axes: AxesBuffer,
}

impl GeneralBuffers {
    fn new(device: &Device) -> Self {
        Self {
            matrix: MatrixBuffer::new(device),
            axes: AxesBuffer::new(device),
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

/// Collection of buffers for drawing axes lines.
#[derive(Debug, Clone)]
pub struct AxesDrawingBuffers {
    pub config: AxesConfigBuffer,
    pub lines: AxesLineBuffer,
}

impl AxesDrawingBuffers {
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
    pub color_scale: Texture,
    pub sampler: Sampler,
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

    pub fn update(&mut self, device: &Device, lines: &[MaybeUninit<ValueLine>]) {
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
