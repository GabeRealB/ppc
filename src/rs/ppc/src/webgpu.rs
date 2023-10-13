#![allow(dead_code)]

use std::{
    borrow::Cow,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::wgsl::HostSharable;

/// Wrapper of a [`web_sys::GpuDevice`].
#[derive(Debug, Clone)]
pub struct Device {
    device: web_sys::GpuDevice,
}

impl Device {
    pub fn new(raw: web_sys::GpuDevice) -> Self {
        if raw.is_falsy() {
            panic!("Invalid device provided");
        }

        Self { device: raw }
    }

    pub fn label(&self) -> String {
        self.device.label()
    }

    pub fn queue(&self) -> Queue {
        Queue {
            queue: self.device.queue(),
        }
    }

    pub fn create_bind_group_layout<const N: usize>(
        &self,
        descriptor: BindGroupLayoutDescriptor<'_, N>,
    ) -> BindGroupLayout {
        let layout = self.device.create_bind_group_layout(&descriptor.into());
        if layout.is_falsy() {
            panic!("could not create bind group layout");
        }

        BindGroupLayout { layout }
    }

    pub fn create_pipeline_layout<const N: usize>(
        &self,
        descriptor: PipelineLayoutDescriptor<'_, N>,
    ) -> PipelineLayout {
        let layout = self.device.create_pipeline_layout(&descriptor.into());
        if layout.is_falsy() {
            panic!("could not create pipeline layout");
        }

        PipelineLayout { layout }
    }

    pub fn create_compute_pipeline(
        &self,
        descriptor: ComputePipelineDescriptor<'_>,
    ) -> ComputePipeline {
        let pipeline = self.device.create_compute_pipeline(&descriptor.into());
        if pipeline.is_falsy() {
            panic!("could not create compute pipeline");
        }

        ComputePipeline { pipeline }
    }

    pub async fn create_compute_pipeline_async(
        &self,
        descriptor: ComputePipelineDescriptor<'_>,
    ) -> ComputePipeline {
        let promise = self
            .device
            .create_compute_pipeline_async(&descriptor.into());
        let pipeline = JsFuture::from(promise)
            .await
            .expect("could not create compute pipeline")
            .dyn_into::<web_sys::GpuComputePipeline>()
            .unwrap();

        ComputePipeline { pipeline }
    }

    pub fn create_render_pipeline<const N: usize>(
        &self,
        descriptor: RenderPipelineDescriptor<'_, N>,
    ) -> RenderPipeline {
        let pipeline = self.device.create_render_pipeline(&descriptor.into());
        if pipeline.is_falsy() {
            panic!("could not create render pipeline");
        }

        RenderPipeline { pipeline }
    }

    pub async fn create_render_pipeline_async<const N: usize>(
        &self,
        descriptor: RenderPipelineDescriptor<'_, N>,
    ) -> RenderPipeline {
        let promise = self.device.create_render_pipeline_async(&descriptor.into());
        let pipeline = JsFuture::from(promise)
            .await
            .expect("could not create render pipeline")
            .dyn_into::<web_sys::GpuRenderPipeline>()
            .unwrap();

        RenderPipeline { pipeline }
    }

    pub fn create_sampler(&self, descriptor: SamplerDescriptor<'_>) -> Sampler {
        let sampler = self
            .device
            .create_sampler_with_descriptor(&descriptor.into());
        if sampler.is_falsy() {
            panic!("could not create sampler");
        }

        Sampler { sampler }
    }

    pub fn create_shader_module(&self, descriptor: ShaderModuleDescriptor<'_>) -> ShaderModule {
        let shader_module = self.device.create_shader_module(&descriptor.into());
        if shader_module.is_falsy() {
            panic!("could not create shader_module");
        }

        ShaderModule {
            module: shader_module,
        }
    }
}

// Wrapper of a [`web_sys::GpuQueue`].
#[derive(Debug, Clone)]
pub struct Queue {
    queue: web_sys::GpuQueue,
}

impl Queue {
    pub fn label(&self) -> String {
        self.queue.label()
    }

    pub fn set_label(&self, value: &str) {
        self.queue.set_label(value);
    }

    pub fn write_buffer<T: HostSharable>(
        &self,
        buffer: &web_sys::GpuBuffer,
        buffer_offset: u32,
        data: &[T],
    ) {
        let data_offset = data as *const [T] as *const () as usize;
        let data_size = std::mem::size_of_val(data);
        assert!(data_offset <= u32::MAX as usize);
        assert!(data_size <= u32::MAX as usize);

        // let data_offset = data_offset as u32;
        // let data_size = data_size as u32;

        // Due to padding it is unsound to simply cast the slice to
        // a `[u8]`, as the padding bytes are uninitialized.
        // A workaround is to copy the data manually into a new buffer
        // and to initialize the padding bytes. While correct, it results
        // in doubling of the memory footprint and is slower than a simple
        // memory copy. Instead, we utilize the fact that we are in a runtime.
        // Inside the wasm runtime we have access to the memory object, which
        // represents the address space available to the program. Program
        // pointers map 1:1 to indices inside, i.e. an object at the pointer
        // `0xCAFE` with size `16` lies in `Memory[0xCAFE..0xCAFE + 16]`.
        // Knowing this, we can pass the buffer, and the respective ranges
        // to the queue, and avoid a slow copy operation.
        let memory = wasm_bindgen::memory()
            .dyn_into::<js_sys::WebAssembly::Memory>()
            .unwrap();
        let memory_data = memory.buffer().dyn_into::<js_sys::ArrayBuffer>().unwrap();
        let memory_data = js_sys::DataView::new(&memory_data, data_offset, data_size);

        self.queue.write_buffer_with_u32_and_buffer_source_and_u32(
            buffer,
            buffer_offset,
            &memory_data,
            0,
        )
    }

    pub fn write_buffer_single<T: HostSharable>(
        &self,
        buffer: &web_sys::GpuBuffer,
        buffer_offset: u32,
        data: &T,
    ) {
        let data = std::slice::from_ref(data);
        self.write_buffer(buffer, buffer_offset, data)
    }

    pub fn write_buffer_raw(&self, buffer: &web_sys::GpuBuffer, buffer_offset: u32, data: &[u8]) {
        self.queue
            .write_buffer_with_u32_and_u8_array(buffer, buffer_offset, data)
    }
}

/// Wrapper of a [`web_sys::GpuBindGroupLayout`].
#[derive(Debug, Clone)]
pub struct BindGroupLayout {
    layout: web_sys::GpuBindGroupLayout,
}

impl BindGroupLayout {
    pub fn label(&self) -> String {
        self.layout.label()
    }

    pub fn set_label(&self, value: &str) {
        self.layout.set_label(value);
    }
}

/// Wrapper of a [`web_sys::GpuPipelineLayout`].
#[derive(Debug, Clone)]
pub struct PipelineLayout {
    layout: web_sys::GpuPipelineLayout,
}

impl PipelineLayout {
    pub fn label(&self) -> String {
        self.layout.label()
    }

    pub fn set_label(&self, value: &str) {
        self.layout.set_label(value);
    }
}

/// Wrapper of a [`web_sys::GpuComputePipeline`].
#[derive(Debug, Clone)]
pub struct ComputePipeline {
    pipeline: web_sys::GpuComputePipeline,
}

impl ComputePipeline {
    pub fn label(&self) -> String {
        self.pipeline.label()
    }

    pub fn set_label(&self, value: &str) {
        self.pipeline.set_label(value);
    }

    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        let layout = self.pipeline.get_bind_group_layout(index);
        if layout.is_falsy() {
            panic!("invalid bind group layout index")
        }

        BindGroupLayout { layout }
    }
}

/// Wrapper of a [`web_sys::GpuPipelineLayout`].
#[derive(Debug, Clone)]
pub struct RenderPipeline {
    pipeline: web_sys::GpuRenderPipeline,
}

impl RenderPipeline {
    pub fn label(&self) -> String {
        self.pipeline.label()
    }

    pub fn set_label(&self, value: &str) {
        self.pipeline.set_label(value);
    }

    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        let layout = self.pipeline.get_bind_group_layout(index);
        if layout.is_falsy() {
            panic!("invalid bind group layout index")
        }

        BindGroupLayout { layout }
    }
}

/// Wrapper of a [`web_sys::GpuSampler`].
#[derive(Debug, Clone)]
pub struct Sampler {
    sampler: web_sys::GpuSampler,
}

impl Sampler {
    pub fn label(&self) -> String {
        self.sampler.label()
    }

    pub fn set_label(&self, value: &str) {
        self.sampler.set_label(value);
    }
}

/// Wrapper of a [`web_sys::GpuShaderModule`].
#[derive(Debug, Clone)]
pub struct ShaderModule {
    module: web_sys::GpuShaderModule,
}

impl ShaderModule {
    pub fn label(&self) -> String {
        self.module.label()
    }

    pub fn set_label(&self, value: &str) {
        self.module.set_label(value);
    }

    pub async fn compilation_info(&self) -> Result<web_sys::GpuCompilationInfo, JsValue> {
        let promise = self.module.compilation_info();
        let compilation_info = JsFuture::from(promise).await?;
        compilation_info.dyn_into::<web_sys::GpuCompilationInfo>()
    }
}

/// Representation of a [`web_sys::GpuBindGroupLayoutDescriptor`].
#[derive(Debug)]
pub struct BindGroupLayoutDescriptor<'a, const N: usize> {
    pub label: Option<Cow<'a, str>>,
    pub entries: [BindGroupLayoutEntry; N],
}

impl<'a, const N: usize> From<BindGroupLayoutDescriptor<'a, N>>
    for web_sys::GpuBindGroupLayoutDescriptor
{
    fn from(value: BindGroupLayoutDescriptor<'a, N>) -> Self {
        let entries = value
            .entries
            .map::<_, web_sys::GpuBindGroupLayoutEntry>(|e| e.into());
        let entries = js_sys::Array::from_iter(entries);

        let mut descriptor = web_sys::GpuBindGroupLayoutDescriptor::new(&entries);

        if let Some(label) = value.label {
            descriptor.label(&label);
        }

        descriptor
    }
}

/// Representation of a [`web_sys::GpuBindGroupLayoutEntry`].
#[derive(Debug)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: ShaderStage,
    pub resource: BindGroupLayoutEntryResource,
}

impl From<BindGroupLayoutEntry> for web_sys::GpuBindGroupLayoutEntry {
    fn from(value: BindGroupLayoutEntry) -> Self {
        let mut entry = web_sys::GpuBindGroupLayoutEntry::new(value.binding, value.visibility.val);
        match value.resource {
            BindGroupLayoutEntryResource::Buffer(b) => entry.buffer(&b.into()),
            BindGroupLayoutEntryResource::Texture(t) => entry.texture(&t.into()),
            BindGroupLayoutEntryResource::StorageTexture(s) => entry.storage_texture(&s.into()),
            BindGroupLayoutEntryResource::Sampler(s) => entry.sampler(&s.into()),
        };

        entry
    }
}

/// Possible shader stages.
#[derive(Debug, Clone, Copy, Hash)]
pub struct ShaderStage {
    val: u32,
}

impl ShaderStage {
    pub const VERTEX: ShaderStage = ShaderStage {
        val: web_sys::gpu_shader_stage::VERTEX,
    };
    pub const FRAGMENT: ShaderStage = ShaderStage {
        val: web_sys::gpu_shader_stage::FRAGMENT,
    };
    pub const COMPUTE: ShaderStage = ShaderStage {
        val: web_sys::gpu_shader_stage::COMPUTE,
    };
}

impl BitAnd for ShaderStage {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val & rhs.val,
        }
    }
}

impl BitAndAssign for ShaderStage {
    fn bitand_assign(&mut self, rhs: Self) {
        self.val &= rhs.val;
    }
}

impl BitOr for ShaderStage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val | rhs.val,
        }
    }
}

impl BitOrAssign for ShaderStage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.val |= rhs.val;
    }
}

impl BitXor for ShaderStage {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val ^ rhs.val,
        }
    }
}

impl BitXorAssign for ShaderStage {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.val ^= rhs.val;
    }
}

/// Possible resources of a [`BindGroupLayoutEntry`].
#[derive(Debug)]
pub enum BindGroupLayoutEntryResource {
    Buffer(BufferBindingLayout),
    Texture(TextureBindingLayout),
    StorageTexture(StorageTextureBindingLayout),
    Sampler(SamplerBindingLayout),
}

/// Representation of a [`web_sys::GpuBufferBindingLayout`].
#[derive(Debug, Default)]
pub struct BufferBindingLayout {
    pub has_dynamic_offset: Option<bool>,
    pub min_binding_size: Option<f64>,
    pub r#type: Option<BufferBindingType>,
}

impl From<BufferBindingLayout> for web_sys::GpuBufferBindingLayout {
    fn from(value: BufferBindingLayout) -> Self {
        let has_dynamic_offset = value.has_dynamic_offset.unwrap_or(false);
        let min_binding_size = value.min_binding_size.unwrap_or(0.0);
        let r#type = value.r#type.unwrap_or(BufferBindingType::Uniform).into();

        let mut layout = web_sys::GpuBufferBindingLayout::new();
        layout.has_dynamic_offset(has_dynamic_offset);
        layout.min_binding_size(min_binding_size);
        layout.type_(r#type);

        layout
    }
}

/// Representation of a [`web_sys::GpuBufferBindingType`].
#[derive(Debug)]
pub enum BufferBindingType {
    Uniform,
    Storage,
    ReadOnlyStorage,
}

impl From<BufferBindingType> for web_sys::GpuBufferBindingType {
    fn from(value: BufferBindingType) -> Self {
        match value {
            BufferBindingType::Uniform => web_sys::GpuBufferBindingType::Uniform,
            BufferBindingType::Storage => web_sys::GpuBufferBindingType::Storage,
            BufferBindingType::ReadOnlyStorage => web_sys::GpuBufferBindingType::ReadOnlyStorage,
        }
    }
}

/// Representation of a [`web_sys::GpuTextureBindingLayout`].
#[derive(Debug)]
pub struct TextureBindingLayout {
    pub multisampled: Option<bool>,
    pub sample_type: Option<TextureSampleType>,
    pub view_dimension: Option<TextureViewDimension>,
}

impl From<TextureBindingLayout> for web_sys::GpuTextureBindingLayout {
    fn from(value: TextureBindingLayout) -> Self {
        let mut layout = web_sys::GpuTextureBindingLayout::new();
        value.multisampled.map(|x| layout.multisampled(x));
        value.sample_type.map(|x| layout.sample_type(x.into()));
        value
            .view_dimension
            .map(|x| layout.view_dimension(x.into()));
        layout
    }
}

/// Representation of a [`web_sys::GpuTextureSampleType`].
#[derive(Debug)]
pub enum TextureSampleType {
    Float,
    UnfilterableFloat,
    Depth,
    SInt,
    UInt,
}

impl From<TextureSampleType> for web_sys::GpuTextureSampleType {
    fn from(value: TextureSampleType) -> Self {
        match value {
            TextureSampleType::Float => web_sys::GpuTextureSampleType::Float,
            TextureSampleType::UnfilterableFloat => {
                web_sys::GpuTextureSampleType::UnfilterableFloat
            }
            TextureSampleType::Depth => web_sys::GpuTextureSampleType::Depth,
            TextureSampleType::SInt => web_sys::GpuTextureSampleType::Sint,
            TextureSampleType::UInt => web_sys::GpuTextureSampleType::Uint,
        }
    }
}

/// Representation of a [`web_sys::GpuTextureViewDimension`].
#[derive(Debug)]
pub enum TextureViewDimension {
    D1,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

impl From<TextureViewDimension> for web_sys::GpuTextureViewDimension {
    fn from(value: TextureViewDimension) -> Self {
        match value {
            TextureViewDimension::D1 => web_sys::GpuTextureViewDimension::N1d,
            TextureViewDimension::D2 => web_sys::GpuTextureViewDimension::N2d,
            TextureViewDimension::D2Array => web_sys::GpuTextureViewDimension::N2dArray,
            TextureViewDimension::Cube => web_sys::GpuTextureViewDimension::Cube,
            TextureViewDimension::CubeArray => web_sys::GpuTextureViewDimension::CubeArray,
            TextureViewDimension::D3 => web_sys::GpuTextureViewDimension::N3d,
        }
    }
}

// Available texture formats
#[derive(Debug, Clone, Copy, Hash)]
pub enum TextureFormat {
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R16Uint,
    R16sint,
    R16float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8uint,
    Rg8sint,
    R32uint,
    R32sint,
    R32float,
    Rg16uint,
    Rg16sint,
    Rg16float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8uint,
    Rgba8sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgb9e5ufloat,
    Rgb10a2Unorm,
    Rg11b10ufloat,
    Rg32uint,
    Rg32sint,
    Rg32float,
    Rgba16uint,
    Rgba16sint,
    Rgba16float,
    Rgba32uint,
    Rgba32sint,
    Rgba32float,
    Stencil8,
    Depth16Unorm,
    Depth24plus,
    Depth24plusStencil8,
    Depth32float,
    Depth32floatStencil8,
    Bc1RgbaUnorm,
    Bc1RgbaUnormSrgb,
    Bc2RgbaUnorm,
    Bc2RgbaUnormSrgb,
    Bc3RgbaUnorm,
    Bc3RgbaUnormSrgb,
    Bc4RUnorm,
    Bc4RSnorm,
    Bc5RgUnorm,
    Bc5RgSnorm,
    Bc6hRgbUfloat,
    Bc6hRgbFloat,
    Bc7RgbaUnorm,
    Bc7RgbaUnormSrgb,
    Etc2Rgb8Unorm,
    Etc2Rgb8UnormSrgb,
    Etc2Rgb8a1Unorm,
    Etc2Rgb8a1UnormSrgb,
    Etc2Rgba8Unorm,
    Etc2Rgba8UnormSrgb,
    EacR11Unorm,
    EacR11Snorm,
    EacRg11Unorm,
    EacRg11Snorm,
    Astc4x4Unorm,
    Astc4x4UnormSrgb,
    Astc5x4Unorm,
    Astc5x4UnormSrgb,
    Astc5x5Unorm,
    Astc5x5UnormSrgb,
    Astc6x5Unorm,
    Astc6x5UnormSrgb,
    Astc6x6Unorm,
    Astc6x6UnormSrgb,
    Astc8x5Unorm,
    Astc8x5UnormSrgb,
    Astc8x6Unorm,
    Astc8x6UnormSrgb,
    Astc8x8Unorm,
    Astc8x8UnormSrgb,
    Astc10x5Unorm,
    Astc10x5UnormSrgb,
    Astc10x6Unorm,
    Astc10x6UnormSrgb,
    Astc10x8Unorm,
    Astc10x8UnormSrgb,
    Astc10x10Unorm,
    Astc10x10UnormSrgb,
    Astc12x10Unorm,
    Astc12x10UnormSrgb,
    Astc12x12Unorm,
    Astc12x12UnormSrgb,
}

impl From<TextureFormat> for web_sys::GpuTextureFormat {
    fn from(value: TextureFormat) -> Self {
        match value {
            TextureFormat::R8Unorm => web_sys::GpuTextureFormat::R8unorm,
            TextureFormat::R8Snorm => web_sys::GpuTextureFormat::R8snorm,
            TextureFormat::R8Uint => web_sys::GpuTextureFormat::R8uint,
            TextureFormat::R8Sint => web_sys::GpuTextureFormat::R8sint,
            TextureFormat::R16Uint => web_sys::GpuTextureFormat::R16uint,
            TextureFormat::R16sint => web_sys::GpuTextureFormat::R16sint,
            TextureFormat::R16float => web_sys::GpuTextureFormat::R16float,
            TextureFormat::Rg8Unorm => web_sys::GpuTextureFormat::Rg8unorm,
            TextureFormat::Rg8Snorm => web_sys::GpuTextureFormat::Rg8snorm,
            TextureFormat::Rg8uint => web_sys::GpuTextureFormat::Rg8uint,
            TextureFormat::Rg8sint => web_sys::GpuTextureFormat::Rg8sint,
            TextureFormat::R32uint => web_sys::GpuTextureFormat::R32uint,
            TextureFormat::R32sint => web_sys::GpuTextureFormat::R32sint,
            TextureFormat::R32float => web_sys::GpuTextureFormat::R32float,
            TextureFormat::Rg16uint => web_sys::GpuTextureFormat::Rg16uint,
            TextureFormat::Rg16sint => web_sys::GpuTextureFormat::Rg16sint,
            TextureFormat::Rg16float => web_sys::GpuTextureFormat::Rg16float,
            TextureFormat::Rgba8Unorm => web_sys::GpuTextureFormat::Rgba8unorm,
            TextureFormat::Rgba8UnormSrgb => web_sys::GpuTextureFormat::Rgba8unormSrgb,
            TextureFormat::Rgba8Snorm => web_sys::GpuTextureFormat::Rgba8snorm,
            TextureFormat::Rgba8uint => web_sys::GpuTextureFormat::Rgba8uint,
            TextureFormat::Rgba8sint => web_sys::GpuTextureFormat::Rgba8sint,
            TextureFormat::Bgra8Unorm => web_sys::GpuTextureFormat::Bgra8unorm,
            TextureFormat::Bgra8UnormSrgb => web_sys::GpuTextureFormat::Bgra8unormSrgb,
            TextureFormat::Rgb9e5ufloat => web_sys::GpuTextureFormat::Rgb9e5ufloat,
            TextureFormat::Rgb10a2Unorm => web_sys::GpuTextureFormat::Rgb10a2unorm,
            TextureFormat::Rg11b10ufloat => web_sys::GpuTextureFormat::Rg11b10ufloat,
            TextureFormat::Rg32uint => web_sys::GpuTextureFormat::Rg32uint,
            TextureFormat::Rg32sint => web_sys::GpuTextureFormat::Rg32sint,
            TextureFormat::Rg32float => web_sys::GpuTextureFormat::Rg32float,
            TextureFormat::Rgba16uint => web_sys::GpuTextureFormat::Rgba16uint,
            TextureFormat::Rgba16sint => web_sys::GpuTextureFormat::Rgba16sint,
            TextureFormat::Rgba16float => web_sys::GpuTextureFormat::Rgba16float,
            TextureFormat::Rgba32uint => web_sys::GpuTextureFormat::Rgba32uint,
            TextureFormat::Rgba32sint => web_sys::GpuTextureFormat::Rgba32sint,
            TextureFormat::Rgba32float => web_sys::GpuTextureFormat::Rgba32float,
            TextureFormat::Stencil8 => web_sys::GpuTextureFormat::Stencil8,
            TextureFormat::Depth16Unorm => web_sys::GpuTextureFormat::Depth16unorm,
            TextureFormat::Depth24plus => web_sys::GpuTextureFormat::Depth24plus,
            TextureFormat::Depth24plusStencil8 => web_sys::GpuTextureFormat::Depth24plusStencil8,
            TextureFormat::Depth32float => web_sys::GpuTextureFormat::Depth32float,
            TextureFormat::Depth32floatStencil8 => web_sys::GpuTextureFormat::Depth32floatStencil8,
            TextureFormat::Bc1RgbaUnorm => web_sys::GpuTextureFormat::Bc1RgbaUnorm,
            TextureFormat::Bc1RgbaUnormSrgb => web_sys::GpuTextureFormat::Bc1RgbaUnormSrgb,
            TextureFormat::Bc2RgbaUnorm => web_sys::GpuTextureFormat::Bc2RgbaUnorm,
            TextureFormat::Bc2RgbaUnormSrgb => web_sys::GpuTextureFormat::Bc2RgbaUnormSrgb,
            TextureFormat::Bc3RgbaUnorm => web_sys::GpuTextureFormat::Bc3RgbaUnorm,
            TextureFormat::Bc3RgbaUnormSrgb => web_sys::GpuTextureFormat::Bc3RgbaUnormSrgb,
            TextureFormat::Bc4RUnorm => web_sys::GpuTextureFormat::Bc4RUnorm,
            TextureFormat::Bc4RSnorm => web_sys::GpuTextureFormat::Bc4RSnorm,
            TextureFormat::Bc5RgUnorm => web_sys::GpuTextureFormat::Bc5RgUnorm,
            TextureFormat::Bc5RgSnorm => web_sys::GpuTextureFormat::Bc5RgSnorm,
            TextureFormat::Bc6hRgbUfloat => web_sys::GpuTextureFormat::Bc6hRgbUfloat,
            TextureFormat::Bc6hRgbFloat => web_sys::GpuTextureFormat::Bc6hRgbFloat,
            TextureFormat::Bc7RgbaUnorm => web_sys::GpuTextureFormat::Bc7RgbaUnorm,
            TextureFormat::Bc7RgbaUnormSrgb => web_sys::GpuTextureFormat::Bc7RgbaUnormSrgb,
            TextureFormat::Etc2Rgb8Unorm => web_sys::GpuTextureFormat::Etc2Rgb8unorm,
            TextureFormat::Etc2Rgb8UnormSrgb => web_sys::GpuTextureFormat::Etc2Rgb8unormSrgb,
            TextureFormat::Etc2Rgb8a1Unorm => web_sys::GpuTextureFormat::Etc2Rgb8a1unorm,
            TextureFormat::Etc2Rgb8a1UnormSrgb => web_sys::GpuTextureFormat::Etc2Rgb8a1unormSrgb,
            TextureFormat::Etc2Rgba8Unorm => web_sys::GpuTextureFormat::Etc2Rgba8unorm,
            TextureFormat::Etc2Rgba8UnormSrgb => web_sys::GpuTextureFormat::Etc2Rgba8unormSrgb,
            TextureFormat::EacR11Unorm => web_sys::GpuTextureFormat::EacR11unorm,
            TextureFormat::EacR11Snorm => web_sys::GpuTextureFormat::EacR11snorm,
            TextureFormat::EacRg11Unorm => web_sys::GpuTextureFormat::EacRg11unorm,
            TextureFormat::EacRg11Snorm => web_sys::GpuTextureFormat::EacRg11snorm,
            TextureFormat::Astc4x4Unorm => web_sys::GpuTextureFormat::Astc4x4Unorm,
            TextureFormat::Astc4x4UnormSrgb => web_sys::GpuTextureFormat::Astc4x4UnormSrgb,
            TextureFormat::Astc5x4Unorm => web_sys::GpuTextureFormat::Astc5x4Unorm,
            TextureFormat::Astc5x4UnormSrgb => web_sys::GpuTextureFormat::Astc5x4UnormSrgb,
            TextureFormat::Astc5x5Unorm => web_sys::GpuTextureFormat::Astc5x5Unorm,
            TextureFormat::Astc5x5UnormSrgb => web_sys::GpuTextureFormat::Astc5x5UnormSrgb,
            TextureFormat::Astc6x5Unorm => web_sys::GpuTextureFormat::Astc6x5Unorm,
            TextureFormat::Astc6x5UnormSrgb => web_sys::GpuTextureFormat::Astc6x5UnormSrgb,
            TextureFormat::Astc6x6Unorm => web_sys::GpuTextureFormat::Astc6x6Unorm,
            TextureFormat::Astc6x6UnormSrgb => web_sys::GpuTextureFormat::Astc6x6UnormSrgb,
            TextureFormat::Astc8x5Unorm => web_sys::GpuTextureFormat::Astc8x5Unorm,
            TextureFormat::Astc8x5UnormSrgb => web_sys::GpuTextureFormat::Astc8x5UnormSrgb,
            TextureFormat::Astc8x6Unorm => web_sys::GpuTextureFormat::Astc8x6Unorm,
            TextureFormat::Astc8x6UnormSrgb => web_sys::GpuTextureFormat::Astc8x6UnormSrgb,
            TextureFormat::Astc8x8Unorm => web_sys::GpuTextureFormat::Astc8x8Unorm,
            TextureFormat::Astc8x8UnormSrgb => web_sys::GpuTextureFormat::Astc8x8UnormSrgb,
            TextureFormat::Astc10x5Unorm => web_sys::GpuTextureFormat::Astc10x5Unorm,
            TextureFormat::Astc10x5UnormSrgb => web_sys::GpuTextureFormat::Astc10x5UnormSrgb,
            TextureFormat::Astc10x6Unorm => web_sys::GpuTextureFormat::Astc10x6Unorm,
            TextureFormat::Astc10x6UnormSrgb => web_sys::GpuTextureFormat::Astc10x6UnormSrgb,
            TextureFormat::Astc10x8Unorm => web_sys::GpuTextureFormat::Astc10x8Unorm,
            TextureFormat::Astc10x8UnormSrgb => web_sys::GpuTextureFormat::Astc10x8UnormSrgb,
            TextureFormat::Astc10x10Unorm => web_sys::GpuTextureFormat::Astc10x10Unorm,
            TextureFormat::Astc10x10UnormSrgb => web_sys::GpuTextureFormat::Astc10x10UnormSrgb,
            TextureFormat::Astc12x10Unorm => web_sys::GpuTextureFormat::Astc12x10Unorm,
            TextureFormat::Astc12x10UnormSrgb => web_sys::GpuTextureFormat::Astc12x10UnormSrgb,
            TextureFormat::Astc12x12Unorm => web_sys::GpuTextureFormat::Astc12x12Unorm,
            TextureFormat::Astc12x12UnormSrgb => web_sys::GpuTextureFormat::Astc12x12UnormSrgb,
        }
    }
}

impl From<web_sys::GpuTextureFormat> for TextureFormat {
    fn from(value: web_sys::GpuTextureFormat) -> Self {
        match value {
            web_sys::GpuTextureFormat::R8unorm => TextureFormat::R8Unorm,
            web_sys::GpuTextureFormat::R8snorm => TextureFormat::R8Snorm,
            web_sys::GpuTextureFormat::R8uint => TextureFormat::R8Uint,
            web_sys::GpuTextureFormat::R8sint => TextureFormat::R8Sint,
            web_sys::GpuTextureFormat::R16uint => TextureFormat::R16Uint,
            web_sys::GpuTextureFormat::R16sint => TextureFormat::R16sint,
            web_sys::GpuTextureFormat::R16float => TextureFormat::R16float,
            web_sys::GpuTextureFormat::Rg8unorm => TextureFormat::Rg8Unorm,
            web_sys::GpuTextureFormat::Rg8snorm => TextureFormat::Rg8Snorm,
            web_sys::GpuTextureFormat::Rg8uint => TextureFormat::Rg8uint,
            web_sys::GpuTextureFormat::Rg8sint => TextureFormat::Rg8sint,
            web_sys::GpuTextureFormat::R32uint => TextureFormat::R32uint,
            web_sys::GpuTextureFormat::R32sint => TextureFormat::R32sint,
            web_sys::GpuTextureFormat::R32float => TextureFormat::R32float,
            web_sys::GpuTextureFormat::Rg16uint => TextureFormat::Rg16uint,
            web_sys::GpuTextureFormat::Rg16sint => TextureFormat::Rg16sint,
            web_sys::GpuTextureFormat::Rg16float => TextureFormat::Rg16float,
            web_sys::GpuTextureFormat::Rgba8unorm => TextureFormat::Rgba8Unorm,
            web_sys::GpuTextureFormat::Rgba8unormSrgb => TextureFormat::Rgba8UnormSrgb,
            web_sys::GpuTextureFormat::Rgba8snorm => TextureFormat::Rgba8Snorm,
            web_sys::GpuTextureFormat::Rgba8uint => TextureFormat::Rgba8uint,
            web_sys::GpuTextureFormat::Rgba8sint => TextureFormat::Rgba8sint,
            web_sys::GpuTextureFormat::Bgra8unorm => TextureFormat::Bgra8Unorm,
            web_sys::GpuTextureFormat::Bgra8unormSrgb => TextureFormat::Bgra8UnormSrgb,
            web_sys::GpuTextureFormat::Rgb9e5ufloat => TextureFormat::Rgb9e5ufloat,
            web_sys::GpuTextureFormat::Rgb10a2unorm => TextureFormat::Rgb10a2Unorm,
            web_sys::GpuTextureFormat::Rg11b10ufloat => TextureFormat::Rg11b10ufloat,
            web_sys::GpuTextureFormat::Rg32uint => TextureFormat::Rg32uint,
            web_sys::GpuTextureFormat::Rg32sint => TextureFormat::Rg32sint,
            web_sys::GpuTextureFormat::Rg32float => TextureFormat::Rg32float,
            web_sys::GpuTextureFormat::Rgba16uint => TextureFormat::Rgba16uint,
            web_sys::GpuTextureFormat::Rgba16sint => TextureFormat::Rgba16sint,
            web_sys::GpuTextureFormat::Rgba16float => TextureFormat::Rgba16float,
            web_sys::GpuTextureFormat::Rgba32uint => TextureFormat::Rgba32uint,
            web_sys::GpuTextureFormat::Rgba32sint => TextureFormat::Rgba32sint,
            web_sys::GpuTextureFormat::Rgba32float => TextureFormat::Rgba32float,
            web_sys::GpuTextureFormat::Stencil8 => TextureFormat::Stencil8,
            web_sys::GpuTextureFormat::Depth16unorm => TextureFormat::Depth16Unorm,
            web_sys::GpuTextureFormat::Depth24plus => TextureFormat::Depth24plus,
            web_sys::GpuTextureFormat::Depth24plusStencil8 => TextureFormat::Depth24plusStencil8,
            web_sys::GpuTextureFormat::Depth32float => TextureFormat::Depth32float,
            web_sys::GpuTextureFormat::Depth32floatStencil8 => TextureFormat::Depth32floatStencil8,
            web_sys::GpuTextureFormat::Bc1RgbaUnorm => TextureFormat::Bc1RgbaUnorm,
            web_sys::GpuTextureFormat::Bc1RgbaUnormSrgb => TextureFormat::Bc1RgbaUnormSrgb,
            web_sys::GpuTextureFormat::Bc2RgbaUnorm => TextureFormat::Bc2RgbaUnorm,
            web_sys::GpuTextureFormat::Bc2RgbaUnormSrgb => TextureFormat::Bc2RgbaUnormSrgb,
            web_sys::GpuTextureFormat::Bc3RgbaUnorm => TextureFormat::Bc3RgbaUnorm,
            web_sys::GpuTextureFormat::Bc3RgbaUnormSrgb => TextureFormat::Bc3RgbaUnormSrgb,
            web_sys::GpuTextureFormat::Bc4RUnorm => TextureFormat::Bc4RUnorm,
            web_sys::GpuTextureFormat::Bc4RSnorm => TextureFormat::Bc4RSnorm,
            web_sys::GpuTextureFormat::Bc5RgUnorm => TextureFormat::Bc5RgUnorm,
            web_sys::GpuTextureFormat::Bc5RgSnorm => TextureFormat::Bc5RgSnorm,
            web_sys::GpuTextureFormat::Bc6hRgbUfloat => TextureFormat::Bc6hRgbUfloat,
            web_sys::GpuTextureFormat::Bc6hRgbFloat => TextureFormat::Bc6hRgbFloat,
            web_sys::GpuTextureFormat::Bc7RgbaUnorm => TextureFormat::Bc7RgbaUnorm,
            web_sys::GpuTextureFormat::Bc7RgbaUnormSrgb => TextureFormat::Bc7RgbaUnormSrgb,
            web_sys::GpuTextureFormat::Etc2Rgb8unorm => TextureFormat::Etc2Rgb8Unorm,
            web_sys::GpuTextureFormat::Etc2Rgb8unormSrgb => TextureFormat::Etc2Rgb8UnormSrgb,
            web_sys::GpuTextureFormat::Etc2Rgb8a1unorm => TextureFormat::Etc2Rgb8a1Unorm,
            web_sys::GpuTextureFormat::Etc2Rgb8a1unormSrgb => TextureFormat::Etc2Rgb8a1UnormSrgb,
            web_sys::GpuTextureFormat::Etc2Rgba8unorm => TextureFormat::Etc2Rgba8Unorm,
            web_sys::GpuTextureFormat::Etc2Rgba8unormSrgb => TextureFormat::Etc2Rgba8UnormSrgb,
            web_sys::GpuTextureFormat::EacR11unorm => TextureFormat::EacR11Unorm,
            web_sys::GpuTextureFormat::EacR11snorm => TextureFormat::EacR11Snorm,
            web_sys::GpuTextureFormat::EacRg11unorm => TextureFormat::EacRg11Unorm,
            web_sys::GpuTextureFormat::EacRg11snorm => TextureFormat::EacRg11Snorm,
            web_sys::GpuTextureFormat::Astc4x4Unorm => TextureFormat::Astc4x4Unorm,
            web_sys::GpuTextureFormat::Astc4x4UnormSrgb => TextureFormat::Astc4x4UnormSrgb,
            web_sys::GpuTextureFormat::Astc5x4Unorm => TextureFormat::Astc5x4Unorm,
            web_sys::GpuTextureFormat::Astc5x4UnormSrgb => TextureFormat::Astc5x4UnormSrgb,
            web_sys::GpuTextureFormat::Astc5x5Unorm => TextureFormat::Astc5x5Unorm,
            web_sys::GpuTextureFormat::Astc5x5UnormSrgb => TextureFormat::Astc5x5UnormSrgb,
            web_sys::GpuTextureFormat::Astc6x5Unorm => TextureFormat::Astc6x5Unorm,
            web_sys::GpuTextureFormat::Astc6x5UnormSrgb => TextureFormat::Astc6x5UnormSrgb,
            web_sys::GpuTextureFormat::Astc6x6Unorm => TextureFormat::Astc6x6Unorm,
            web_sys::GpuTextureFormat::Astc6x6UnormSrgb => TextureFormat::Astc6x6UnormSrgb,
            web_sys::GpuTextureFormat::Astc8x5Unorm => TextureFormat::Astc8x5Unorm,
            web_sys::GpuTextureFormat::Astc8x5UnormSrgb => TextureFormat::Astc8x5UnormSrgb,
            web_sys::GpuTextureFormat::Astc8x6Unorm => TextureFormat::Astc8x6Unorm,
            web_sys::GpuTextureFormat::Astc8x6UnormSrgb => TextureFormat::Astc8x6UnormSrgb,
            web_sys::GpuTextureFormat::Astc8x8Unorm => TextureFormat::Astc8x8Unorm,
            web_sys::GpuTextureFormat::Astc8x8UnormSrgb => TextureFormat::Astc8x8UnormSrgb,
            web_sys::GpuTextureFormat::Astc10x5Unorm => TextureFormat::Astc10x5Unorm,
            web_sys::GpuTextureFormat::Astc10x5UnormSrgb => TextureFormat::Astc10x5UnormSrgb,
            web_sys::GpuTextureFormat::Astc10x6Unorm => TextureFormat::Astc10x6Unorm,
            web_sys::GpuTextureFormat::Astc10x6UnormSrgb => TextureFormat::Astc10x6UnormSrgb,
            web_sys::GpuTextureFormat::Astc10x8Unorm => TextureFormat::Astc10x8Unorm,
            web_sys::GpuTextureFormat::Astc10x8UnormSrgb => TextureFormat::Astc10x8UnormSrgb,
            web_sys::GpuTextureFormat::Astc10x10Unorm => TextureFormat::Astc10x10Unorm,
            web_sys::GpuTextureFormat::Astc10x10UnormSrgb => TextureFormat::Astc10x10UnormSrgb,
            web_sys::GpuTextureFormat::Astc12x10Unorm => TextureFormat::Astc12x10Unorm,
            web_sys::GpuTextureFormat::Astc12x10UnormSrgb => TextureFormat::Astc12x10UnormSrgb,
            web_sys::GpuTextureFormat::Astc12x12Unorm => TextureFormat::Astc12x12Unorm,
            web_sys::GpuTextureFormat::Astc12x12UnormSrgb => TextureFormat::Astc12x12UnormSrgb,
            _ => panic!("unrecognized texture format"),
        }
    }
}

/// Representation of a [`web_sys::GpuStorageTextureBindingLayout`].
#[derive(Debug)]
pub struct StorageTextureBindingLayout {
    pub access: Option<StorageTextureAccess>,
    pub format: TextureFormat,
    pub view_dimension: Option<TextureViewDimension>,
}

impl From<StorageTextureBindingLayout> for web_sys::GpuStorageTextureBindingLayout {
    fn from(value: StorageTextureBindingLayout) -> Self {
        let mut layout = web_sys::GpuStorageTextureBindingLayout::new(value.format.into());
        value.access.map(|x| layout.access(x.into()));
        value
            .view_dimension
            .map(|x| layout.view_dimension(x.into()));
        layout
    }
}

/// Representation of a [`web_sys::GpuStorageTextureAccess`].
#[derive(Debug, Clone, Copy, Hash)]
pub enum StorageTextureAccess {
    WriteOnly,
}

impl From<StorageTextureAccess> for web_sys::GpuStorageTextureAccess {
    fn from(value: StorageTextureAccess) -> Self {
        match value {
            StorageTextureAccess::WriteOnly => web_sys::GpuStorageTextureAccess::WriteOnly,
        }
    }
}

/// Representation of a [`web_sys::GpuSamplerBindingLayout`].
#[derive(Debug)]
pub struct SamplerBindingLayout {
    pub r#type: Option<SamplerBindingType>,
}

impl From<SamplerBindingLayout> for web_sys::GpuSamplerBindingLayout {
    fn from(value: SamplerBindingLayout) -> Self {
        let mut layout = web_sys::GpuSamplerBindingLayout::new();
        value.r#type.map(|x| layout.type_(x.into()));
        layout
    }
}

/// Representation of a [`web_sys::GpuSamplerBindingType`].
#[derive(Debug)]
pub enum SamplerBindingType {
    Filtering,
    NonFiltering,
    Comparison,
}

impl From<SamplerBindingType> for web_sys::GpuSamplerBindingType {
    fn from(value: SamplerBindingType) -> Self {
        match value {
            SamplerBindingType::Filtering => web_sys::GpuSamplerBindingType::Filtering,
            SamplerBindingType::NonFiltering => web_sys::GpuSamplerBindingType::NonFiltering,
            SamplerBindingType::Comparison => web_sys::GpuSamplerBindingType::Comparison,
        }
    }
}

/// Representation of a [`web_sys::GpuShaderModuleDescriptor`].
#[derive(Debug)]
pub struct ShaderModuleDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
    pub code: Cow<'a, str>,
}

impl<'a> From<ShaderModuleDescriptor<'a>> for web_sys::GpuShaderModuleDescriptor {
    fn from(value: ShaderModuleDescriptor<'a>) -> Self {
        let mut descriptor = web_sys::GpuShaderModuleDescriptor::new(&value.code);
        value.label.map(|l| descriptor.label(&l));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuComputePipelineDescriptor`].
#[derive(Debug)]
pub struct ComputePipelineDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
    pub layout: PipelineLayoutType,
    pub compute: ProgrammableStage<'a>,
}

impl<'a> From<ComputePipelineDescriptor<'a>> for web_sys::GpuComputePipelineDescriptor {
    fn from(value: ComputePipelineDescriptor<'a>) -> Self {
        let layout = value.layout.into();
        let compute = value.compute.into();
        let mut descriptor = web_sys::GpuComputePipelineDescriptor::new(&layout, &compute);
        value.label.map(|x| descriptor.label(&x));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuProgrammableStage`].
#[derive(Debug)]
pub struct ProgrammableStage<'a> {
    pub entry_point: &'a str,
    pub module: ShaderModule,
}

impl<'a> From<ProgrammableStage<'a>> for web_sys::GpuProgrammableStage {
    fn from(value: ProgrammableStage<'a>) -> Self {
        web_sys::GpuProgrammableStage::new(value.entry_point, &value.module.module)
    }
}

/// Representation of a [`web_sys::GpuRenderPipelineDescriptor`].
#[derive(Debug)]
pub struct RenderPipelineDescriptor<'a, const N: usize> {
    pub label: Option<Cow<'a, str>>,
    pub layout: PipelineLayoutType,
    pub vertex: VertexState<'a>,
    pub fragment: Option<FragmentState<'a, N>>,
    pub multisample: Option<MultisampleState>,
    pub primitive: Option<PrimitiveState>,
}

impl<'a, const N: usize> From<RenderPipelineDescriptor<'a, N>>
    for web_sys::GpuRenderPipelineDescriptor
{
    fn from(value: RenderPipelineDescriptor<'a, N>) -> Self {
        let layout = value.layout.into();
        let vertex = value.vertex.into();

        let mut descriptor = web_sys::GpuRenderPipelineDescriptor::new(&layout, &vertex);

        if let Some(label) = value.label {
            descriptor.label(&label);
        }

        if let Some(fragment) = value.fragment {
            descriptor.fragment(&fragment.into());
        }

        if let Some(multisample) = value.multisample {
            descriptor.multisample(&multisample.into());
        }

        if let Some(primitive) = value.primitive {
            descriptor.primitive(&primitive.into());
        }

        descriptor
    }
}

/// A pipeline layout.
#[derive(Debug)]
pub enum PipelineLayoutType {
    Auto,
    Layout(PipelineLayout),
}

impl From<PipelineLayoutType> for JsValue {
    fn from(value: PipelineLayoutType) -> Self {
        match value {
            PipelineLayoutType::Auto => JsValue::from_str("auto"),
            PipelineLayoutType::Layout(l) => l.layout.into(),
        }
    }
}

/// Representation of a [`web_sys::GpuPipelineLayoutDescriptor`].
#[derive(Debug)]
pub struct PipelineLayoutDescriptor<'a, const N: usize> {
    pub label: Option<Cow<'a, str>>,
    pub layouts: [BindGroupLayout; N],
}

impl<'a, const N: usize> From<PipelineLayoutDescriptor<'a, N>>
    for web_sys::GpuPipelineLayoutDescriptor
{
    fn from(value: PipelineLayoutDescriptor<'a, N>) -> Self {
        let layouts =
            js_sys::Array::from_iter(value.layouts.map::<_, JsValue>(|l| l.layout.into())).into();
        let mut descriptor = web_sys::GpuPipelineLayoutDescriptor::new(&layouts);
        value.label.map(|l| descriptor.label(&l));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuVertexState`].
#[derive(Debug)]
pub struct VertexState<'a> {
    pub entry_point: &'a str,
    pub module: ShaderModule,
}

impl<'a> From<VertexState<'a>> for web_sys::GpuVertexState {
    fn from(value: VertexState<'a>) -> Self {
        web_sys::GpuVertexState::new(value.entry_point, &value.module.module)
    }
}

/// Representation of a [`web_sys::GpuFragmentState`].
#[derive(Debug)]
pub struct FragmentState<'a, const N: usize> {
    pub entry_point: &'a str,
    pub module: ShaderModule,
    pub targets: [FragmentStateTarget; N],
}

impl<'a, const N: usize> From<FragmentState<'a, N>> for web_sys::GpuFragmentState {
    fn from(value: FragmentState<'a, N>) -> Self {
        let entry_point = value.entry_point;
        let module = value.module.module;
        let targets = value.targets.map::<_, js_sys::Object>(Into::into);
        let targets = js_sys::Array::from_iter(targets);

        web_sys::GpuFragmentState::new(entry_point, &module, &targets)
    }
}

/// Representation of a [`web_sys::GpuFragmentState`] target.
#[derive(Debug)]
pub struct FragmentStateTarget {
    pub format: TextureFormat,
    pub blend: Option<FragmentStateBlend>,
    pub write_mask: Option<u32>,
}

impl From<FragmentStateTarget> for js_sys::Object {
    fn from(value: FragmentStateTarget) -> Self {
        let format: web_sys::GpuTextureFormat = value.format.into();

        let object: ObjectExt = js_sys::Object::new().unchecked_into::<ObjectExt>();
        object.set("format".into(), format.into());

        if let Some(blend) = value.blend {
            object.set("blend".into(), js_sys::Object::from(blend).into());
        }

        if let Some(write_mask) = value.write_mask {
            object.set("writeMask".into(), write_mask.into());
        }

        object.unchecked_into::<js_sys::Object>()
    }
}

/// Representation of a [`web_sys::GpuFragmentState`] target blend configuration.
#[derive(Debug, Default)]
pub struct FragmentStateBlend {
    pub alpha: FragmentStateBlendEntry,
    pub color: FragmentStateBlendEntry,
}

impl From<FragmentStateBlend> for js_sys::Object {
    fn from(value: FragmentStateBlend) -> Self {
        let object: ObjectExt = js_sys::Object::new().unchecked_into::<ObjectExt>();
        object.set("alpha".into(), js_sys::Object::from(value.alpha).into());
        object.set("color".into(), js_sys::Object::from(value.color).into());
        object.unchecked_into::<js_sys::Object>()
    }
}

/// Representation of a [`web_sys::GpuFragmentState`] target blend configuration.
#[derive(Debug, Default)]
pub struct FragmentStateBlendEntry {
    pub dst_factor: Option<BlendFactor>,
    pub operation: Option<BlendOperation>,
    pub src_factor: Option<BlendFactor>,
}

impl From<FragmentStateBlendEntry> for js_sys::Object {
    fn from(value: FragmentStateBlendEntry) -> Self {
        let object: ObjectExt = js_sys::Object::new().unchecked_into::<ObjectExt>();

        if let Some(factor) = value.dst_factor {
            object.set("dstFactor".into(), factor.into());
        }

        if let Some(operation) = value.operation {
            object.set("srcFactor".into(), operation.into());
        }

        if let Some(factor) = value.src_factor {
            object.set("srcFactor".into(), factor.into());
        }

        object.unchecked_into::<js_sys::Object>()
    }
}

/// Supported blend factors.
#[derive(Debug)]
pub enum BlendFactor {
    Constant,
    Dst,
    DstAlpha,
    One,
    OneMinusDst,
    OneMinusSrc,
    OneMinusSrcAlpha,
    OneMinusDstAlpha,
    OneMinusConstant,
    Src,
    SrcAlpha,
    SrcAlphaSaturated,
    Zero,
}

impl From<BlendFactor> for JsValue {
    fn from(value: BlendFactor) -> Self {
        match value {
            BlendFactor::Constant => JsValue::from_str("constant"),
            BlendFactor::Dst => JsValue::from_str("dst"),
            BlendFactor::DstAlpha => JsValue::from_str("dst-alpha"),
            BlendFactor::One => JsValue::from_str("one"),
            BlendFactor::OneMinusDst => JsValue::from_str("one-minus-dst"),
            BlendFactor::OneMinusSrc => JsValue::from_str("one-minus-src"),
            BlendFactor::OneMinusSrcAlpha => JsValue::from_str("one-minus-src-alpha"),
            BlendFactor::OneMinusDstAlpha => JsValue::from_str("one-minus-dst-alpha"),
            BlendFactor::OneMinusConstant => JsValue::from_str("one-minus-constant"),
            BlendFactor::Src => JsValue::from_str("src"),
            BlendFactor::SrcAlpha => JsValue::from_str("src-alpha"),
            BlendFactor::SrcAlphaSaturated => JsValue::from_str("src-alpha-saturated"),
            BlendFactor::Zero => JsValue::from_str("zero"),
        }
    }
}

// Supported blend operations.
#[derive(Debug)]
pub enum BlendOperation {
    Add,
    Max,
    Min,
    ReverseSubtract,
    Subtract,
}

impl From<BlendOperation> for JsValue {
    fn from(value: BlendOperation) -> Self {
        match value {
            BlendOperation::Add => JsValue::from_str("add"),
            BlendOperation::Max => JsValue::from_str("max"),
            BlendOperation::Min => JsValue::from_str("min"),
            BlendOperation::ReverseSubtract => JsValue::from_str("reverse-subtract"),
            BlendOperation::Subtract => JsValue::from_str("subtract"),
        }
    }
}

/// Representation of a [`web_sys::GpuMultisampleState`].
#[derive(Debug, Default)]
pub struct MultisampleState {
    pub alpha_to_coverage_enabled: Option<bool>,
    pub count: Option<u32>,
    pub mask: Option<u32>,
}

impl From<MultisampleState> for web_sys::GpuMultisampleState {
    fn from(value: MultisampleState) -> Self {
        let alpha_to_coverage_enabled = value.alpha_to_coverage_enabled.unwrap_or(false);
        let count = value.count.unwrap_or(1);
        let mask = value.mask.unwrap_or(0xFFFFFFFF);

        let mut state = web_sys::GpuMultisampleState::new();
        state.alpha_to_coverage_enabled(alpha_to_coverage_enabled);
        state.count(count);
        state.mask(mask);
        state
    }
}

/// Representation of a [`web_sys::GpuPrimitiveState`].
#[derive(Debug, Default)]
pub struct PrimitiveState {
    pub cull_mode: Option<CullMode>,
    pub front_face: Option<FrontFace>,
    pub strip_index_format: Option<IndexFormat>,
    pub topology: Option<PrimitiveTopology>,
    pub unclipped_depth: Option<bool>,
}

impl From<PrimitiveState> for web_sys::GpuPrimitiveState {
    fn from(value: PrimitiveState) -> Self {
        let mut state = web_sys::GpuPrimitiveState::new();
        if let Some(v) = value.cull_mode {
            state.cull_mode(v.into());
        }
        if let Some(v) = value.front_face {
            state.front_face(v.into());
        }
        if let Some(v) = value.strip_index_format {
            state.strip_index_format(v.into());
        }
        if let Some(v) = value.topology {
            state.topology(v.into());
        }
        if let Some(v) = value.unclipped_depth {
            state.unclipped_depth(v);
        }
        state
    }
}

/// Possible cull modes.
#[derive(Debug)]
pub enum CullMode {
    Back,
    Front,
    None,
}

impl From<CullMode> for web_sys::GpuCullMode {
    fn from(value: CullMode) -> Self {
        match value {
            CullMode::Back => web_sys::GpuCullMode::Back,
            CullMode::Front => web_sys::GpuCullMode::Front,
            CullMode::None => web_sys::GpuCullMode::None,
        }
    }
}

/// Possible front face modes
#[derive(Debug)]
pub enum FrontFace {
    Ccw,
    CW,
}

impl From<FrontFace> for web_sys::GpuFrontFace {
    fn from(value: FrontFace) -> Self {
        match value {
            FrontFace::Ccw => web_sys::GpuFrontFace::Ccw,
            FrontFace::CW => web_sys::GpuFrontFace::Cw,
        }
    }
}

/// Possible index formats.
#[derive(Debug)]
pub enum IndexFormat {
    UInt16,
    UInt32,
}

impl From<IndexFormat> for web_sys::GpuIndexFormat {
    fn from(value: IndexFormat) -> Self {
        match value {
            IndexFormat::UInt16 => web_sys::GpuIndexFormat::Uint16,
            IndexFormat::UInt32 => web_sys::GpuIndexFormat::Uint32,
        }
    }
}

/// Possible primitive topologies.
#[derive(Debug)]
pub enum PrimitiveTopology {
    LineList,
    LineStrip,
    PointList,
    TriangleList,
    TriangleStrip,
}

impl From<PrimitiveTopology> for web_sys::GpuPrimitiveTopology {
    fn from(value: PrimitiveTopology) -> Self {
        match value {
            PrimitiveTopology::LineList => web_sys::GpuPrimitiveTopology::LineList,
            PrimitiveTopology::LineStrip => web_sys::GpuPrimitiveTopology::LineStrip,
            PrimitiveTopology::PointList => web_sys::GpuPrimitiveTopology::PointList,
            PrimitiveTopology::TriangleList => web_sys::GpuPrimitiveTopology::TriangleList,
            PrimitiveTopology::TriangleStrip => web_sys::GpuPrimitiveTopology::TriangleStrip,
        }
    }
}

/// Representation of a [`web_sys::GpuSamplerDescriptor`].
#[derive(Debug)]
pub struct SamplerDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
    pub address_mode_u: Option<AddressMode>,
    pub address_mode_v: Option<AddressMode>,
    pub address_mode_w: Option<AddressMode>,
    pub compare: Option<CompareFunction>,
    pub lod_max_clamp: Option<f32>,
    pub lod_min_clamp: Option<f32>,
    pub mag_filter: Option<FilterMode>,
    pub max_anisotropy: Option<u16>,
    pub min_filter: Option<FilterMode>,
    pub mipmap_filter: Option<MipMapFilterMode>,
}

impl<'a> From<SamplerDescriptor<'a>> for web_sys::GpuSamplerDescriptor {
    fn from(value: SamplerDescriptor<'a>) -> Self {
        let mut descriptor = web_sys::GpuSamplerDescriptor::new();
        value.label.map(|x| descriptor.label(&x));
        value
            .address_mode_u
            .map(|x| descriptor.address_mode_u(x.into()));
        value
            .address_mode_v
            .map(|x| descriptor.address_mode_v(x.into()));
        value
            .address_mode_w
            .map(|x| descriptor.address_mode_w(x.into()));
        value.compare.map(|x| descriptor.compare(x.into()));
        value.lod_max_clamp.map(|x| descriptor.lod_max_clamp(x));
        value.lod_min_clamp.map(|x| descriptor.lod_min_clamp(x));
        value.mag_filter.map(|x| descriptor.mag_filter(x.into()));
        value.max_anisotropy.map(|x| descriptor.max_anisotropy(x));
        value.min_filter.map(|x| descriptor.min_filter(x.into()));
        value
            .mipmap_filter
            .map(|x| descriptor.mipmap_filter(x.into()));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuAddressMode`].
#[derive(Debug)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

impl From<AddressMode> for web_sys::GpuAddressMode {
    fn from(value: AddressMode) -> Self {
        match value {
            AddressMode::ClampToEdge => web_sys::GpuAddressMode::ClampToEdge,
            AddressMode::Repeat => web_sys::GpuAddressMode::Repeat,
            AddressMode::MirrorRepeat => web_sys::GpuAddressMode::MirrorRepeat,
        }
    }
}

/// Representation of a [`web_sys::GpuCompareFunction`].
#[derive(Debug)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl From<CompareFunction> for web_sys::GpuCompareFunction {
    fn from(value: CompareFunction) -> Self {
        match value {
            CompareFunction::Never => web_sys::GpuCompareFunction::Never,
            CompareFunction::Less => web_sys::GpuCompareFunction::Less,
            CompareFunction::Equal => web_sys::GpuCompareFunction::Equal,
            CompareFunction::LessEqual => web_sys::GpuCompareFunction::LessEqual,
            CompareFunction::Greater => web_sys::GpuCompareFunction::Greater,
            CompareFunction::NotEqual => web_sys::GpuCompareFunction::NotEqual,
            CompareFunction::GreaterEqual => web_sys::GpuCompareFunction::GreaterEqual,
            CompareFunction::Always => web_sys::GpuCompareFunction::Always,
        }
    }
}

/// Representation of a [`web_sys::GpuFilterMode`].
#[derive(Debug)]
pub enum FilterMode {
    Nearest,
    Linear,
}

impl From<FilterMode> for web_sys::GpuFilterMode {
    fn from(value: FilterMode) -> Self {
        match value {
            FilterMode::Nearest => web_sys::GpuFilterMode::Nearest,
            FilterMode::Linear => web_sys::GpuFilterMode::Linear,
        }
    }
}

/// Representation of a [`web_sys::GpuMipmapFilterMode`].
#[derive(Debug)]
pub enum MipMapFilterMode {
    Nearest,
    Linear,
}

impl From<MipMapFilterMode> for web_sys::GpuMipmapFilterMode {
    fn from(value: MipMapFilterMode) -> Self {
        match value {
            MipMapFilterMode::Nearest => web_sys::GpuMipmapFilterMode::Nearest,
            MipMapFilterMode::Linear => web_sys::GpuMipmapFilterMode::Linear,
        }
    }
}

/// Custom bindings to avoid using fallible `Reflect` for plain objects.
#[wasm_bindgen]
extern "C" {
    type ObjectExt;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &ObjectExt, key: js_sys::JsString, value: JsValue);
}
