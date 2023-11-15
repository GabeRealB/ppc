#![allow(dead_code)]

use std::{
    borrow::Cow,
    mem::MaybeUninit,
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

    pub fn create_bind_group<const N: usize>(
        &self,
        descriptor: BindGroupDescriptor<'_, N>,
    ) -> BindGroup {
        let group = self.device.create_bind_group(&descriptor.into());
        if group.is_falsy() {
            panic!("could not create bind group");
        }

        BindGroup { group }
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

    pub fn create_buffer(&self, descriptor: BufferDescriptor<'_>) -> Buffer {
        let buffer = self.device.create_buffer(&descriptor.into());
        if buffer.is_falsy() {
            panic!("could not create buffer");
        }

        Buffer { buffer }
    }

    pub fn create_command_encoder(
        &self,
        descriptor: CommandEncoderDescriptor<'_>,
    ) -> CommandEncoder {
        let encoder = self
            .device
            .create_command_encoder_with_descriptor(&descriptor.into());
        if encoder.is_falsy() {
            panic!("could not create command encoder")
        }

        CommandEncoder { encoder }
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
        let descriptor: web_sys::GpuRenderPipelineDescriptor = descriptor.into();
        let promise = self.device.create_render_pipeline_async(&descriptor);
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

    pub fn create_texture<const N: usize, const M: usize>(
        &self,
        descriptor: TextureDescriptor<'_, N, M>,
    ) -> Texture {
        let texture = self.device.create_texture(&descriptor.into());
        if texture.is_falsy() {
            panic!("could not create texture");
        }

        Texture { texture }
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

    pub fn submit(&self, command_buffers: &[CommandBuffer]) {
        let command_buffers =
            js_sys::Array::from_iter(command_buffers.iter().map(|x| x.command_buffer.clone()));
        self.queue.submit(&command_buffers.into());
    }

    pub fn write_buffer<T: HostSharable>(&self, buffer: &Buffer, buffer_offset: u32, data: &[T]) {
        let data_offset = data as *const [T] as *const () as usize;
        let data_size = std::mem::size_of_val(data);
        assert!(data_offset <= u32::MAX as usize);
        assert!(data_size <= u32::MAX as usize);

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
            &buffer.buffer,
            buffer_offset,
            &memory_data,
            0,
        )
    }

    pub fn write_buffer_single<T: HostSharable>(
        &self,
        buffer: &Buffer,
        buffer_offset: u32,
        data: &T,
    ) {
        let data = std::slice::from_ref(data);
        self.write_buffer(buffer, buffer_offset, data)
    }

    pub fn write_buffer_raw(&self, buffer: &Buffer, buffer_offset: u32, data: &[u8]) {
        self.queue
            .write_buffer_with_u32_and_u8_array(&buffer.buffer, buffer_offset, data)
    }
}

/// Wrapper of a [`web_sys::GpuBindGroup`].
#[derive(Debug, Clone)]
pub struct BindGroup {
    group: web_sys::GpuBindGroup,
}

impl BindGroup {
    pub fn label(&self) -> String {
        self.group.label()
    }

    pub fn set_label(&self, value: &str) {
        self.group.set_label(value);
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

/// Wrapper of a [`web_sys::GpuBuffer`]
#[derive(Debug, Clone)]
pub struct Buffer {
    buffer: web_sys::GpuBuffer,
}

impl Buffer {
    pub fn label(&self) -> String {
        self.buffer.label()
    }

    pub fn set_label(&self, value: &str) {
        self.buffer.set_label(value);
    }

    pub fn size(&self) -> usize {
        self.buffer.size() as usize
    }

    pub fn usage(&self) -> BufferUsage {
        let usage = self.buffer.usage();
        BufferUsage(usage)
    }

    pub fn map_state(&self) -> BufferMapState {
        self.buffer.map_state().into()
    }

    pub fn mapped_range(&self) -> js_sys::ArrayBuffer {
        self.buffer.get_mapped_range()
    }

    pub unsafe fn get_mapped_range<T: HostSharable>(&self) -> Box<[T]> {
        if self.size() % std::mem::size_of::<T>() != 0 {
            panic!("invalid buffer size for the selected element type")
        }

        let num_elements = self.size() / std::mem::size_of::<T>();
        let mut elements = Vec::<MaybeUninit<T>>::with_capacity(num_elements);

        let mapped_range = self.buffer.get_mapped_range();
        let mapped_range_u8 = js_sys::Uint8Array::new(&mapped_range);

        let elements_ptr = elements.as_mut_ptr().cast::<u8>();
        mapped_range_u8.raw_copy_to_ptr(elements_ptr);
        elements.set_len(num_elements);

        let elements: Box<[MaybeUninit<T>]> = elements.into();
        let ptr = Box::into_raw(elements);
        Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            ptr.cast::<T>(),
            num_elements,
        ))
    }

    pub async fn map_async(&self, mode: MapMode) {
        let promise = self.buffer.map_async(mode.0);
        JsFuture::from(promise).await.expect("could not map buffer");
    }

    pub fn unmap(&self) {
        self.buffer.unmap()
    }

    pub fn destroy(&self) {
        self.buffer.destroy();
    }
}

/// Wrapper of a [`web_sys::GpuCommandEncoder`].
#[derive(Debug, Clone)]
pub struct CommandEncoder {
    encoder: web_sys::GpuCommandEncoder,
}

impl CommandEncoder {
    pub fn label(&self) -> String {
        self.encoder.label()
    }

    pub fn set_label(&self, value: &str) {
        self.encoder.set_label(value);
    }

    pub fn begin_compute_pass(
        &self,
        descriptor: Option<ComputePassDescriptor<'_>>,
    ) -> ComputePassEncoder {
        let encoder = if let Some(descriptor) = descriptor {
            self.encoder
                .begin_compute_pass_with_descriptor(&descriptor.into())
        } else {
            self.encoder.begin_compute_pass()
        };
        if encoder.is_falsy() {
            panic!("could not begin compute pass")
        }

        ComputePassEncoder { encoder }
    }

    pub fn begin_render_pass<const N: usize>(
        &self,
        descriptor: RenderPassDescriptor<'_, N>,
    ) -> RenderPassEncoder {
        let encoder = self.encoder.begin_render_pass(&descriptor.into());
        if encoder.is_falsy() {
            panic!("could not begin render pass")
        }

        RenderPassEncoder { encoder }
    }

    pub fn clear_buffer(&self, buffer: &Buffer) {
        self.encoder.clear_buffer(&buffer.buffer)
    }

    pub fn clear_buffer_with_offset(&self, buffer: &Buffer, offset: usize) {
        self.encoder
            .clear_buffer_with_u32(&buffer.buffer, offset as u32)
    }

    pub fn clear_buffer_with_offset_and_size(&self, buffer: &Buffer, offset: usize, size: usize) {
        self.encoder
            .clear_buffer_with_u32_and_u32(&buffer.buffer, offset as u32, size as u32)
    }

    pub fn copy_buffer_to_buffer(
        &self,
        source: &Buffer,
        source_offset: usize,
        destination: &Buffer,
        destination_offset: usize,
        size: usize,
    ) {
        self.encoder.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            &source.buffer,
            source_offset as u32,
            &destination.buffer,
            destination_offset as u32,
            size as u32,
        )
    }

    pub fn finish(&self, descriptor: Option<CommandBufferDescriptor<'_>>) -> CommandBuffer {
        let command_buffer = if let Some(descriptor) = descriptor {
            self.encoder.finish_with_descriptor(&descriptor.into())
        } else {
            self.encoder.finish()
        };

        CommandBuffer { command_buffer }
    }
}

/// Wrapper of a [`web_sys::GpuComputePassEncoder`].
#[derive(Debug, Clone)]
pub struct ComputePassEncoder {
    encoder: web_sys::GpuComputePassEncoder,
}

impl ComputePassEncoder {
    pub fn label(&self) -> String {
        self.encoder.label()
    }

    pub fn set_label(&self, value: &str) {
        self.encoder.set_label(value);
    }

    pub fn dispatch_workgroups(&self, workgroup_count: &[u32]) {
        match workgroup_count.len() {
            1 => self.encoder.dispatch_workgroups(workgroup_count[0]),
            2 => self
                .encoder
                .dispatch_workgroups_with_workgroup_count_y(workgroup_count[0], workgroup_count[1]),
            3 => self
                .encoder
                .dispatch_workgroups_with_workgroup_count_y_and_workgroup_count_z(
                    workgroup_count[0],
                    workgroup_count[1],
                    workgroup_count[2],
                ),
            _ => panic!("invalid workgroup count"),
        }
    }

    pub fn dispatch_workgroups_indirect(&self, indirect_buffer: &Buffer, indirect_offset: usize) {
        self.encoder
            .dispatch_workgroups_indirect_with_f64(&indirect_buffer.buffer, indirect_offset as f64)
    }

    pub fn end(&self) {
        self.encoder.end()
    }

    pub fn set_pipeline(&self, pipeline: &ComputePipeline) {
        self.encoder.set_pipeline(&pipeline.pipeline)
    }

    pub fn set_bind_group(&self, index: u32, bind_group: &BindGroup) {
        self.encoder.set_bind_group(index, &bind_group.group)
    }
}

/// Wrapper of a [`web_sys::GpuRenderPassEncoder`].
#[derive(Debug, Clone)]
pub struct RenderPassEncoder {
    encoder: web_sys::GpuRenderPassEncoder,
}

impl RenderPassEncoder {
    pub fn label(&self) -> String {
        self.encoder.label()
    }

    pub fn set_label(&self, value: &str) {
        self.encoder.set_label(value);
    }

    pub fn begin_occlusion_query(&self, query_index: u32) {
        self.encoder.begin_occlusion_query(query_index)
    }

    pub fn end(&self) {
        self.encoder.end()
    }

    pub fn end_occlusion_query(&self) {
        self.encoder.end_occlusion_query()
    }

    pub fn set_blend_constant(&self, color: [f64; 4]) {
        let [r, g, b, a] = color;
        self.encoder
            .set_blend_constant_with_gpu_color_dict(&web_sys::GpuColorDict::new(a, b, g, r))
    }

    pub fn set_scissor_rect(&self, x: u32, y: u32, width: u32, height: u32) {
        self.encoder.set_scissor_rect(x, y, width, height)
    }

    pub fn set_stencil_reference(&self, reference: u32) {
        self.encoder.set_stencil_reference(reference)
    }

    pub fn set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.encoder
            .set_viewport(x, y, width, height, min_depth, max_depth)
    }

    pub fn set_bind_group(&self, index: u32, bind_group: &BindGroup) {
        self.encoder.set_bind_group(index, &bind_group.group)
    }

    pub fn draw(&self, vertex_count: usize) {
        self.encoder.draw(vertex_count as u32)
    }

    pub fn draw_with_instance_count(&self, vertex_count: usize, instance_count: usize) {
        self.encoder
            .draw_with_instance_count(vertex_count as u32, instance_count as u32)
    }

    pub fn draw_with_instance_count_and_first_vertex(
        &self,
        vertex_count: usize,
        instance_count: usize,
        first_vertex: usize,
    ) {
        self.encoder.draw_with_instance_count_and_first_vertex(
            vertex_count as u32,
            instance_count as u32,
            first_vertex as u32,
        )
    }

    pub fn draw_with_instance_count_and_first_vertex_and_first_instance(
        &self,
        vertex_count: usize,
        instance_count: usize,
        first_vertex: usize,
        first_instance: usize,
    ) {
        self.encoder
            .draw_with_instance_count_and_first_vertex_and_first_instance(
                vertex_count as u32,
                instance_count as u32,
                first_vertex as u32,
                first_instance as u32,
            )
    }

    pub fn set_pipeline(&self, pipeline: &RenderPipeline) {
        self.encoder.set_pipeline(&pipeline.pipeline)
    }

    pub fn set_vertex_buffer(&self, slot: u32, buffer: &Buffer) {
        self.encoder.set_vertex_buffer(slot, &buffer.buffer)
    }

    pub fn set_vertex_buffer_with_offset(&self, slot: u32, buffer: &Buffer, offset: usize) {
        self.encoder
            .set_vertex_buffer_with_u32(slot, &buffer.buffer, offset as u32)
    }

    pub fn set_vertex_buffer_with_offset_and_size(
        &self,
        slot: u32,
        buffer: &Buffer,
        offset: usize,
        size: usize,
    ) {
        self.encoder.set_vertex_buffer_with_u32_and_u32(
            slot,
            &buffer.buffer,
            offset as u32,
            size as u32,
        )
    }
}

/// Wrapper of a [`web_sys::GpuCommandBuffer`].
#[derive(Debug, Clone)]
pub struct CommandBuffer {
    command_buffer: web_sys::GpuCommandBuffer,
}

impl CommandBuffer {
    pub fn label(&self) -> String {
        self.command_buffer.label()
    }

    pub fn set_label(&self, value: &str) {
        self.command_buffer.set_label(value);
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

/// Wrapper of a [`web_sys::GpuTexture`].
#[derive(Debug, Clone)]
pub struct Texture {
    texture: web_sys::GpuTexture,
}

impl Texture {
    pub fn from_raw(texture: web_sys::GpuTexture) -> Self {
        Self { texture }
    }

    pub fn width(&self) -> u32 {
        self.texture.width()
    }

    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    pub fn depth_or_array_layers(&self) -> u32 {
        self.texture.depth_or_array_layers()
    }

    pub fn mip_level_count(&self) -> u32 {
        self.texture.mip_level_count()
    }

    pub fn sample_count(&self) -> u32 {
        self.texture.sample_count()
    }

    pub fn dimension(&self) -> TextureDimension {
        self.texture.dimension().into()
    }

    pub fn format(&self) -> TextureFormat {
        self.texture.format().into()
    }

    pub fn usage(&self) -> TextureUsage {
        TextureUsage(self.texture.usage())
    }

    pub fn label(&self) -> String {
        self.texture.label()
    }

    pub fn set_label(&self, value: &str) {
        self.texture.set_label(value);
    }

    pub fn create_view(&self, descriptor: Option<TextureViewDescriptor>) -> TextureView {
        let view = if let Some(descriptor) = descriptor {
            self.texture.create_view_with_descriptor(&descriptor.into())
        } else {
            self.texture.create_view()
        };

        TextureView { view }
    }
}

/// Wrapper of a [`web_sys::GpuTextureView`].
#[derive(Debug, Clone)]
pub struct TextureView {
    view: web_sys::GpuTextureView,
}

impl TextureView {
    pub fn label(&self) -> String {
        self.view.label()
    }

    pub fn set_label(&self, value: &str) {
        self.view.set_label(value);
    }
}

/// Representation of a [`web_sys::GpuBindGroupDescriptor`].
#[derive(Debug)]
pub struct BindGroupDescriptor<'a, const N: usize> {
    pub label: Option<Cow<'a, str>>,
    pub entries: [BindGroupEntry; N],
    pub layout: BindGroupLayout,
}

impl<'a, const N: usize> From<BindGroupDescriptor<'a, N>> for web_sys::GpuBindGroupDescriptor {
    fn from(value: BindGroupDescriptor<'a, N>) -> Self {
        let entries = value
            .entries
            .map::<_, web_sys::GpuBindGroupEntry>(|e| e.into());
        let entries = js_sys::Array::from_iter(entries);

        let mut descriptor = web_sys::GpuBindGroupDescriptor::new(&entries, &value.layout.layout);

        if let Some(label) = value.label {
            descriptor.label(&label);
        }

        descriptor
    }
}

/// Representation of a [`web_sys::GpuBindGroupEntry`].
#[derive(Debug)]
pub struct BindGroupEntry {
    pub binding: u32,
    pub resource: BindGroupEntryResource,
}

impl From<BindGroupEntry> for web_sys::GpuBindGroupEntry {
    fn from(value: BindGroupEntry) -> Self {
        web_sys::GpuBindGroupEntry::new(value.binding, &value.resource.into())
    }
}

/// A resource for a [`BindGroupEntry`].
#[derive(Debug)]
pub enum BindGroupEntryResource {
    Buffer(BufferBinding),
    Sampler(Sampler),
    TextureView(TextureView),
}

impl From<BindGroupEntryResource> for JsValue {
    fn from(value: BindGroupEntryResource) -> Self {
        match value {
            BindGroupEntryResource::Buffer(x) => web_sys::GpuBufferBinding::from(x).into(),
            BindGroupEntryResource::Sampler(x) => x.sampler.into(),
            BindGroupEntryResource::TextureView(x) => x.view.into(),
        }
    }
}

/// Representation of a [`web_sys::GpuBufferBinding`].
#[derive(Debug)]
pub struct BufferBinding {
    pub buffer: Buffer,
    pub offset: Option<usize>,
    pub size: Option<usize>,
}

impl From<BufferBinding> for web_sys::GpuBufferBinding {
    fn from(value: BufferBinding) -> Self {
        let mut binding = web_sys::GpuBufferBinding::new(&value.buffer.buffer);

        if let Some(offset) = value.offset {
            binding.offset(offset as f64);
        }
        if let Some(size) = value.size {
            binding.size(size as f64);
        }

        binding
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

/// Representation of a [`web_sys::GpuBufferDescriptor`].
#[derive(Debug)]
pub struct BufferDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
    pub size: usize,
    pub usage: BufferUsage,
    pub mapped_at_creation: Option<bool>,
}

impl From<BufferDescriptor<'_>> for web_sys::GpuBufferDescriptor {
    fn from(value: BufferDescriptor) -> Self {
        let size = value.size as f64;
        let usage = value.usage.0;

        let mut descriptor = web_sys::GpuBufferDescriptor::new(size, usage);
        value.label.map(|x| descriptor.label(&x));
        value
            .mapped_at_creation
            .map(|x| descriptor.mapped_at_creation(x));
        descriptor
    }
}

/// Representation of a buffer usage bitset.
#[derive(Debug)]
pub struct BufferUsage(u32);

impl BufferUsage {
    pub const COPY_SRC: Self = BufferUsage(web_sys::gpu_buffer_usage::COPY_SRC);
    pub const COPY_DST: Self = BufferUsage(web_sys::gpu_buffer_usage::COPY_DST);
    pub const INDEX: Self = BufferUsage(web_sys::gpu_buffer_usage::INDEX);
    pub const INDIRECT: Self = BufferUsage(web_sys::gpu_buffer_usage::INDIRECT);
    pub const MAP_READ: Self = BufferUsage(web_sys::gpu_buffer_usage::MAP_READ);
    pub const MAP_WRITE: Self = BufferUsage(web_sys::gpu_buffer_usage::MAP_WRITE);
    pub const QUERY_RESOLVE: Self = BufferUsage(web_sys::gpu_buffer_usage::QUERY_RESOLVE);
    pub const STORAGE: Self = BufferUsage(web_sys::gpu_buffer_usage::STORAGE);
    pub const UNIFORM: Self = BufferUsage(web_sys::gpu_buffer_usage::UNIFORM);
    pub const VERTEX: Self = BufferUsage(web_sys::gpu_buffer_usage::VERTEX);
}

impl BitAnd for BufferUsage {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for BufferUsage {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BufferUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for BufferUsage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for BufferUsage {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for BufferUsage {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

/// Representation of a [`web_sys::GpuBufferMapState`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BufferMapState {
    Unmapped,
    Pending,
    Mapped,
}

impl From<BufferMapState> for web_sys::GpuBufferMapState {
    fn from(value: BufferMapState) -> Self {
        match value {
            BufferMapState::Unmapped => web_sys::GpuBufferMapState::Unmapped,
            BufferMapState::Pending => web_sys::GpuBufferMapState::Pending,
            BufferMapState::Mapped => web_sys::GpuBufferMapState::Mapped,
        }
    }
}

impl From<web_sys::GpuBufferMapState> for BufferMapState {
    fn from(value: web_sys::GpuBufferMapState) -> Self {
        match value {
            web_sys::GpuBufferMapState::Unmapped => BufferMapState::Unmapped,
            web_sys::GpuBufferMapState::Pending => BufferMapState::Pending,
            web_sys::GpuBufferMapState::Mapped => BufferMapState::Mapped,
            _ => panic!("unsupported map state"),
        }
    }
}

/// Representation of a buffer map mode bitset.
#[derive(Debug)]
pub struct MapMode(u32);

impl MapMode {
    pub const READ: Self = Self(web_sys::gpu_map_mode::READ);
    pub const WRITE: Self = Self(web_sys::gpu_map_mode::WRITE);
}

impl BitAnd for MapMode {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for MapMode {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for MapMode {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for MapMode {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for MapMode {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for MapMode {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

/// Representation of a [`web_sys::GpuTextureDimension`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
}

impl From<web_sys::GpuTextureDimension> for TextureDimension {
    fn from(value: web_sys::GpuTextureDimension) -> Self {
        match value {
            web_sys::GpuTextureDimension::N1d => TextureDimension::D1,
            web_sys::GpuTextureDimension::N2d => TextureDimension::D2,
            web_sys::GpuTextureDimension::N3d => TextureDimension::D3,
            _ => panic!("unknown texture dimension"),
        }
    }
}

impl From<TextureDimension> for web_sys::GpuTextureDimension {
    fn from(value: TextureDimension) -> web_sys::GpuTextureDimension {
        match value {
            TextureDimension::D1 => web_sys::GpuTextureDimension::N1d,
            TextureDimension::D2 => web_sys::GpuTextureDimension::N2d,
            TextureDimension::D3 => web_sys::GpuTextureDimension::N3d,
        }
    }
}

/// Representation of a [`web_sys::GpuTextureViewDimension`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

// Available texture formats.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
            object.set("operation".into(), operation.into());
        }

        if let Some(factor) = value.src_factor {
            object.set("srcFactor".into(), factor.into());
        }

        object.unchecked_into::<js_sys::Object>()
    }
}

/// Supported blend factors.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
        let mut state = web_sys::GpuMultisampleState::new();
        value
            .alpha_to_coverage_enabled
            .map(|x| state.alpha_to_coverage_enabled(x));
        value.count.map(|x| state.count(x));
        value.mask.map(|x| state.mask(x));
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

/// Possible front face modes.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

/// Representation of a [`web_sys::GpuTextureDescriptor`].
#[derive(Debug)]
pub struct TextureDescriptor<'a, const N: usize, const M: usize> {
    pub label: Option<Cow<'a, str>>,
    pub dimension: Option<TextureDimension>,
    pub format: TextureFormat,
    pub mip_level_count: Option<u32>,
    pub sample_count: Option<u32>,
    pub size: [usize; N],
    pub usage: TextureUsage,
    pub view_formats: Option<[TextureFormat; M]>,
}

impl<'a, const N: usize, const M: usize> From<TextureDescriptor<'a, N, M>>
    for web_sys::GpuTextureDescriptor
{
    fn from(value: TextureDescriptor<'a, N, M>) -> Self {
        let format = value.format.into();
        let size = js_sys::Array::from_iter(
            value
                .size
                .into_iter()
                .map(|x| js_sys::Number::from(x as u32)),
        );
        let usage = value.usage.0;

        let mut descriptor = web_sys::GpuTextureDescriptor::new(format, &size, usage);
        value.label.map(|x| descriptor.label(&x));
        value.dimension.map(|x| descriptor.dimension(x.into()));
        value.mip_level_count.map(|x| descriptor.mip_level_count(x));
        value.sample_count.map(|x| descriptor.sample_count(x));
        value.view_formats.map(|x| {
            let x = js_sys::Array::from_iter(
                x.map(|x| wasm_bindgen::JsValue::from(web_sys::GpuTextureFormat::from(x))),
            );
            descriptor.view_formats(&x)
        });
        descriptor
    }
}

/// Representation of a [`web_sys::GpuTextureViewDescriptor`].
#[derive(Debug)]
pub struct TextureViewDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
    pub array_layer_count: Option<u32>,
    pub aspect: Option<TextureAspect>,
    pub base_array_layer: Option<u32>,
    pub base_mip_level: Option<u32>,
    pub dimension: Option<TextureViewDimension>,
    pub format: Option<TextureFormat>,
    pub mip_level_count: Option<u32>,
}

impl<'a> From<TextureViewDescriptor<'a>> for web_sys::GpuTextureViewDescriptor {
    fn from(value: TextureViewDescriptor<'a>) -> Self {
        let mut descriptor = web_sys::GpuTextureViewDescriptor::new();
        value.label.map(|x| descriptor.label(&x));
        value
            .array_layer_count
            .map(|x| descriptor.array_layer_count(x));
        value.aspect.map(|x| descriptor.aspect(x.into()));
        value
            .base_array_layer
            .map(|x| descriptor.base_array_layer(x));
        value.base_mip_level.map(|x| descriptor.base_mip_level(x));
        value.dimension.map(|x| descriptor.dimension(x.into()));
        value.format.map(|x| descriptor.format(x.into()));
        value.mip_level_count.map(|x| descriptor.mip_level_count(x));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuTextureAspect`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TextureAspect {
    All,
    StencilOnly,
    DepthOnly,
}

impl From<TextureAspect> for web_sys::GpuTextureAspect {
    fn from(value: TextureAspect) -> Self {
        match value {
            TextureAspect::All => web_sys::GpuTextureAspect::All,
            TextureAspect::StencilOnly => web_sys::GpuTextureAspect::StencilOnly,
            TextureAspect::DepthOnly => web_sys::GpuTextureAspect::DepthOnly,
        }
    }
}

/// Representation of a texture usage bitset.
#[derive(Debug)]
pub struct TextureUsage(u32);

impl TextureUsage {
    pub const COPY_SRC: Self = Self(web_sys::gpu_texture_usage::COPY_SRC);
    pub const COPY_DST: Self = Self(web_sys::gpu_texture_usage::COPY_DST);
    pub const RENDER_ATTACHMENT: Self = Self(web_sys::gpu_texture_usage::RENDER_ATTACHMENT);
    pub const STORAGE_BINDING: Self = Self(web_sys::gpu_texture_usage::STORAGE_BINDING);
    pub const TEXTURE_BINDING: Self = Self(web_sys::gpu_texture_usage::TEXTURE_BINDING);
}

impl BitAnd for TextureUsage {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for TextureUsage {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for TextureUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for TextureUsage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for TextureUsage {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for TextureUsage {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

/// Representation of a [`web_sys::GpuCommandEncoderDescriptor`].
#[derive(Debug)]
pub struct CommandEncoderDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
}

impl From<CommandEncoderDescriptor<'_>> for web_sys::GpuCommandEncoderDescriptor {
    fn from(value: CommandEncoderDescriptor<'_>) -> Self {
        let mut descriptor = web_sys::GpuCommandEncoderDescriptor::new();
        value.label.map(|x| descriptor.label(&x));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuComputePassDescriptor`].
#[derive(Debug)]
pub struct ComputePassDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
}

impl From<ComputePassDescriptor<'_>> for web_sys::GpuComputePassDescriptor {
    fn from(value: ComputePassDescriptor<'_>) -> Self {
        let mut descriptor = web_sys::GpuComputePassDescriptor::new();
        value.label.map(|x| descriptor.label(&x));
        descriptor
    }
}

/// Representation of a [`web_sys::GpuRenderPassDescriptor`].
#[derive(Debug)]
pub struct RenderPassDescriptor<'a, const N: usize> {
    pub label: Option<Cow<'a, str>>,
    pub color_attachments: [RenderPassColorAttachments; N],
    pub max_draw_count: Option<usize>,
}

impl<const N: usize> From<RenderPassDescriptor<'_, N>> for web_sys::GpuRenderPassDescriptor {
    fn from(value: RenderPassDescriptor<'_, N>) -> Self {
        let color_attachments = value.color_attachments.map::<_, JsValue>(Into::into);
        let color_attachments = js_sys::Array::from_iter(color_attachments);

        let mut descriptor = web_sys::GpuRenderPassDescriptor::new(&color_attachments);
        value.label.map(|x| descriptor.label(&x));
        value
            .max_draw_count
            .map(|x| descriptor.max_draw_count(x as f64));
        descriptor
    }
}

/// Color attachments of a [`RenderPassDescriptor`].
#[derive(Debug)]
pub struct RenderPassColorAttachments {
    pub clear_value: Option<[f32; 4]>,
    pub load_op: RenderPassLoadOp,
    pub store_op: RenderPassStoreOp,
    pub resolve_target: Option<TextureView>,
    pub view: TextureView,
}

impl From<RenderPassColorAttachments> for JsValue {
    fn from(value: RenderPassColorAttachments) -> Self {
        let object: ObjectExt = js_sys::Object::new().unchecked_into::<ObjectExt>();

        if let Some(x) = value.clear_value {
            object.set(
                "clearValue".into(),
                js_sys::Array::from_iter(x.map(js_sys::Number::from)).into(),
            )
        }

        object.set("loadOp".into(), value.load_op.into());
        object.set("storeOp".into(), value.store_op.into());

        if let Some(resolve_target) = value.resolve_target {
            object.set("resolveTarget".into(), resolve_target.view.into())
        }

        object.set("view".into(), value.view.view.into());

        object.unchecked_into::<js_sys::Object>().into()
    }
}

/// Load operation of a [`RenderPassColorAttachments`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPassLoadOp {
    Clear,
    Load,
}

impl From<RenderPassLoadOp> for JsValue {
    fn from(value: RenderPassLoadOp) -> Self {
        match value {
            RenderPassLoadOp::Clear => JsValue::from_str("clear"),
            RenderPassLoadOp::Load => JsValue::from_str("load"),
        }
    }
}

/// Store operation of a [`RenderPassColorAttachments`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPassStoreOp {
    Discard,
    Store,
}

impl From<RenderPassStoreOp> for JsValue {
    fn from(value: RenderPassStoreOp) -> Self {
        match value {
            RenderPassStoreOp::Discard => JsValue::from_str("discard"),
            RenderPassStoreOp::Store => JsValue::from_str("store"),
        }
    }
}

/// Representation of a [`web_sys::GpuCommandBufferDescriptor`].
#[derive(Debug)]
pub struct CommandBufferDescriptor<'a> {
    pub label: Option<Cow<'a, str>>,
}

impl From<CommandBufferDescriptor<'_>> for web_sys::GpuCommandBufferDescriptor {
    fn from(value: CommandBufferDescriptor<'_>) -> Self {
        let mut descriptor = web_sys::GpuCommandBufferDescriptor::new();
        value.label.map(|x| descriptor.label(&x));
        descriptor
    }
}

/// Custom bindings to avoid using fallible `Reflect` for plain objects.
#[wasm_bindgen]
extern "C" {
    type ObjectExt;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &ObjectExt, key: js_sys::JsString, value: JsValue);
}
