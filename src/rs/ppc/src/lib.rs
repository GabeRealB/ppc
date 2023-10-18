use std::{borrow::Cow, cell::RefCell, collections::BTreeSet, mem::MaybeUninit, rc::Rc};

use async_channel::{Receiver, Sender};
use wasm_bindgen::prelude::*;

use web_sys::console;

use crate::coordinates::{Aabb, Length, Position};

mod webgpu;
mod wgsl;

mod axis;
mod buffers;
mod color_scale;
mod colors;
mod coordinates;
mod lerp;
mod pipelines;

const MSAA_SAMPLES: u32 = 4;

/// An event queue to interact with the renderer.
#[wasm_bindgen]
pub struct EventQueue {
    sender: Sender<Event>,
}

#[wasm_bindgen]
impl EventQueue {
    /// Spawns an event to shut down the renderer.
    pub fn exit(&self) {
        self.sender
            .send_blocking(Event::Exit)
            .expect("the channel should be open");
    }

    /// Updates the data of the renderer.
    pub fn update_data(&self, payload: UpdateDataPayload) {
        let axes = if payload.axes.is_empty() {
            None
        } else {
            Some(payload.axes.into())
        };

        let order = if payload.order.is_empty() {
            None
        } else {
            Some(payload.order.into())
        };

        self.sender
            .send_blocking(Event::UpdateData { axes, order })
            .expect("the channel should be open");
    }

    /// Spawns a `resize` event.
    pub fn resize(&self, width: u32, height: u32, device_pixel_ratio: f32) {
        self.sender
            .send_blocking(Event::Resize {
                width,
                height,
                device_pixel_ratio,
            })
            .expect("the channel should be open");
    }

    /// Spawns a `pointer_down` event.
    pub fn pointer_down(&self, event: web_sys::PointerEvent) {
        self.sender
            .send_blocking(Event::PointerDown { event })
            .expect("the channel should be open");
    }

    /// Spawns a `pointer_up` event.
    pub fn pointer_up(&self, event: web_sys::PointerEvent) {
        self.sender
            .send_blocking(Event::PointerUp { event })
            .expect("the channel should be open");
    }

    /// Spawns a `pointer_move` event.
    pub fn pointer_move(&self, event: web_sys::PointerEvent) {
        self.sender
            .send_blocking(Event::PointerMove { event })
            .expect("the channel should be open");
    }

    /// Spawns a `draw` event.
    pub async fn draw(&self) {
        let (sx, rx) = async_channel::bounded(1);

        // Spawn the event.
        self.sender
            .send(Event::Draw { completion: sx })
            .await
            .expect("the channel should be open when trying to send a message");

        // Wait for the event to complete.
        rx.recv().await.expect("the channel should be open");
    }
}

enum Event {
    Exit,
    UpdateData {
        axes: Option<Box<[AxisDef]>>,
        order: Option<Box<[String]>>,
    },
    Draw {
        completion: Sender<()>,
    },
    Resize {
        width: u32,
        height: u32,
        device_pixel_ratio: f32,
    },
    PointerDown {
        event: web_sys::PointerEvent,
    },
    PointerUp {
        event: web_sys::PointerEvent,
    },
    PointerMove {
        event: web_sys::PointerEvent,
    },
}

/// Definition of an axis.
#[wasm_bindgen]
#[derive(Default)]
pub struct UpdateDataPayload {
    axes: Vec<AxisDef>,
    order: Vec<String>,
}

struct AxisDef {
    key: Box<str>,
    label: Box<str>,
    datums: Box<[f32]>,
    range: Option<(f32, f32)>,
    visible_range: Option<(f32, f32)>,
    hidden: bool,
}

#[wasm_bindgen]
impl UpdateDataPayload {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            axes: Vec::new(),
            order: Vec::new(),
        }
    }

    pub fn new_axis(
        &mut self,
        key: &str,
        label: &str,
        datums: Box<[f32]>,
        range: Option<Box<[f32]>>,
        visible_range: Option<Box<[f32]>>,
        hidden: Option<bool>,
    ) {
        self.axes.push(AxisDef {
            key: key.into(),
            label: label.into(),
            datums,
            range: range.map(|v| (v[0], v[1])),
            visible_range: visible_range.map(|v| (v[0], v[1])),
            hidden: hidden.unwrap_or(false),
        });
    }

    pub fn add_order(&mut self, key: &str) {
        self.order.push(key.into())
    }
}

/// Implementation of the renderer for the parallel coordinates.
#[wasm_bindgen]
pub struct Renderer {
    canvas_gpu: web_sys::HtmlCanvasElement,
    canvas_2d: web_sys::HtmlCanvasElement,
    context_gpu: web_sys::GpuCanvasContext,
    context_2d: web_sys::CanvasRenderingContext2d,
    device: webgpu::Device,
    pipelines: pipelines::Pipelines,
    buffers: buffers::Buffers,
    render_texture: webgpu::Texture,
    events: Option<Receiver<Event>>,
    axes: Rc<RefCell<axis::Axes>>,
    redraw: bool,
}

#[wasm_bindgen]
impl Renderer {
    /// Constructs a new renderer.
    #[wasm_bindgen(constructor)]
    pub async fn new(
        canvas_gpu: web_sys::HtmlCanvasElement,
        canvas_2d: web_sys::HtmlCanvasElement,
    ) -> Self {
        console_error_panic_hook::set_once();

        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        if navigator.gpu().is_falsy() {
            panic!("WebGPU is not supported in the current browser.");
        }
        let gpu = navigator.gpu();

        let adapter = match wasm_bindgen_futures::JsFuture::from(gpu.request_adapter()).await {
            Ok(adapter) => {
                if adapter.is_falsy() {
                    panic!("Could not request gpu adapter.");
                }

                adapter.dyn_into::<web_sys::GpuAdapter>().unwrap()
            }
            Err(err) => panic!("Could not request gpu adapter. Error: '{err:?}'"),
        };

        let device = match wasm_bindgen_futures::JsFuture::from(adapter.request_device()).await {
            Ok(device) => {
                if device.is_falsy() {
                    panic!("Could not request gpu device.");
                }

                device.dyn_into::<web_sys::GpuDevice>().unwrap()
            }
            Err(err) => panic!("Could not request gpu device. Error: '{err:?}'"),
        };

        let context_gpu = canvas_gpu
            .get_context("webgpu")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::GpuCanvasContext>()
            .unwrap();

        let context_2d = canvas_2d
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context_gpu.configure(
            web_sys::GpuCanvasConfiguration::new(&device, gpu.get_preferred_canvas_format())
                .alpha_mode(web_sys::GpuCanvasAlphaMode::Premultiplied),
        );

        let device = webgpu::Device::new(device);
        let preferred_format = gpu.get_preferred_canvas_format().into();
        let pipelines = pipelines::Pipelines::new(&device, preferred_format).await;
        let buffers = buffers::Buffers::new(&device);
        let render_texture = device.create_texture(webgpu::TextureDescriptor::<'_, 2, 0> {
            label: Some(Cow::Borrowed("render texture")),
            dimension: None,
            format: preferred_format,
            mip_level_count: None,
            sample_count: Some(MSAA_SAMPLES),
            size: [canvas_gpu.width() as usize, canvas_gpu.height() as usize],
            usage: webgpu::TextureUsage::RENDER_ATTACHMENT,
            view_formats: None,
        });

        let client_width = canvas_gpu.client_width() as f32;
        let client_height = canvas_gpu.client_height() as f32;
        let view_bounding_box = Aabb::new(
            Position::zero(),
            Position::new((client_width, client_height)),
        );

        let document = window.document().unwrap();
        let root_element = document.document_element().unwrap();
        let root_element_style = window.get_computed_style(&root_element).unwrap().unwrap();
        let get_rem_length_screen = Rc::new(move |rem| {
            let font_size_str = root_element_style.get_property_value("font-size").unwrap();
            let font_size = js_sys::parse_float(&font_size_str) as f32;
            Length::new(font_size * rem)
        });

        let get_text_length_screen = {
            let context_2d = context_2d.clone();
            Rc::new(move |text: &str| {
                let metrics = context_2d.measure_text(text).unwrap();
                let width = metrics.width() as f32;
                let height = (metrics.actual_bounding_box_ascent()
                    + metrics.actual_bounding_box_descent()) as f32;
                (Length::new(width), Length::new(height))
            })
        };

        let axes = axis::Axes::new_rc(
            view_bounding_box,
            get_rem_length_screen,
            get_text_length_screen,
        );

        let mut this = Self {
            canvas_gpu,
            canvas_2d,
            context_gpu,
            context_2d,
            device,
            pipelines,
            render_texture,
            buffers,
            events: None,
            axes,
            redraw: true,
        };

        this.update_matrix_buffer();
        this.update_axes_buffer();
        this.update_axes_config_buffer();
        this.update_axes_lines_buffer();

        this
    }

    /// Constructs a new event queue for this renderer.
    ///
    /// # Panics
    ///
    /// Panics if called multiple times.
    pub fn construct_event_queue(&mut self) -> EventQueue {
        if self.events.is_some() {
            panic!("EventQueue was already constructed.");
        }

        let (sx, rx) = async_channel::unbounded();
        self.events = Some(rx);

        EventQueue { sender: sx }
    }

    /// Starts the event loop of the renderer.
    ///
    /// # Panics
    ///
    /// Panics if no [`EventQueue`] is associated with the renderer.
    pub async fn enter_event_loop(&mut self) {
        if self.events.is_none() {
            panic!("EventQueue was not initialized.");
        }

        let events = self.events.take().unwrap();
        loop {
            match events.recv().await.expect("the channel should be open") {
                Event::Exit => break,
                Event::UpdateData { axes, order } => self.update_data(axes, order).await,
                Event::Draw { completion } => self.render(completion).await,
                Event::Resize {
                    width,
                    height,
                    device_pixel_ratio,
                } => {
                    self.resize_drawing_area(width, height, device_pixel_ratio)
                        .await
                }
                Event::PointerDown { event } => self.pointer_down(event).await,
                Event::PointerUp { event } => self.pointer_up(event).await,
                Event::PointerMove { event } => self.pointer_move(event).await,
            }
        }

        self.events = Some(events);
    }
}

impl Renderer {
    fn render_axes(&self, encoder: &webgpu::CommandEncoder, msaa_texture: &webgpu::TextureView) {
        let num_lines = self.buffers.axes.lines.len();
        if num_lines == 0 {
            return;
        }

        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("axes lines bind group")),
            entries: [
                webgpu::BindGroupEntry {
                    binding: 0,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.general.matrix.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 1,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.axes.config.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 2,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.general.axes.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 3,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.axes.lines.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.pipelines.render_pipelines.draw_lines.0.clone(),
        });

        let pass = encoder.begin_render_pass(webgpu::RenderPassDescriptor {
            label: Some(Cow::Borrowed("axes render pass")),
            color_attachments: [webgpu::RenderPassColorAttachments {
                clear_value: None,
                load_op: webgpu::RenderPassLoadOp::Load,
                store_op: webgpu::RenderPassStoreOp::Store,
                resolve_target: None,
                view: msaa_texture.clone(),
            }],
            max_draw_count: None,
        });
        pass.set_pipeline(&self.pipelines.render_pipelines.draw_lines.1);
        pass.set_bind_group(0, &bind_group);
        pass.draw_with_instance_count(6, num_lines);
        pass.end();
    }

    fn render_labels(&self) {
        self.context_2d.save();
        self.context_2d.set_text_align("center");

        let guard = self.axes.borrow();
        let screen_mapper = guard.space_transformer();

        for ax in guard.visible_axes() {
            let label = ax.label();

            if label.is_empty() {
                continue;
            }

            let world_mapper = ax.space_transformer();
            let label_position = ax.label_position();
            let label_position = label_position.transform(&world_mapper);
            let label_position = label_position.transform(&screen_mapper);
            let (x, y) = label_position.extract();

            self.context_2d
                .fill_text(&label, x as f64, y as f64)
                .unwrap();
        }

        self.context_2d.restore();
    }

    fn render_min_max_labels(&self) {
        self.context_2d.save();
        self.context_2d.set_text_align("center");

        let guard = self.axes.borrow();
        let screen_mapper = guard.space_transformer();

        for ax in guard.visible_axes() {
            let min_label = ax.min_label();
            let max_label = ax.max_label();

            let world_mapper = ax.space_transformer();
            if !min_label.is_empty() {
                let position = ax.min_label_position();
                let position = position.transform(&world_mapper);
                let position = position.transform(&screen_mapper);
                let (x, y) = position.extract();

                self.context_2d
                    .fill_text(&min_label, x as f64, y as f64)
                    .unwrap();
            }

            if !max_label.is_empty() {
                let position = ax.max_label_position();
                let position = position.transform(&world_mapper);
                let position = position.transform(&screen_mapper);
                let (x, y) = position.extract();

                self.context_2d
                    .fill_text(&max_label, x as f64, y as f64)
                    .unwrap();
            }
        }

        self.context_2d.restore();
    }
}

impl Renderer {
    async fn render(&mut self, completion: Sender<()>) {
        let redraw = self.redraw;
        self.redraw = false;

        if !redraw {
            completion
                .send(())
                .await
                .expect("the channel should be open");
            return;
        }

        let command_encoder = self
            .device
            .create_command_encoder(webgpu::CommandEncoderDescriptor { label: None });
        let texture_view =
            webgpu::Texture::from_raw(self.context_gpu.get_current_texture()).create_view(None);
        let msaa_texture_view = self.render_texture.create_view(None);

        // Draw the main view into the framebuffer.
        self.render_axes(&command_encoder, &msaa_texture_view);

        self.device.queue().submit(&[command_encoder.finish(None)]);

        // Draw the text and ui control elements.
        self.context_2d.clear_rect(
            0.0,
            0.0,
            self.canvas_2d.width() as f64,
            self.canvas_2d.height() as f64,
        );
        self.render_labels();
        self.render_min_max_labels();

        completion
            .send(())
            .await
            .expect("the channel should be open");
    }

    async fn update_data(&mut self, axes: Option<Box<[AxisDef]>>, order: Option<Box<[String]>>) {
        let axes_keys = axes
            .iter()
            .flat_map(|x| x.iter())
            .map(|a| &*a.key)
            .collect::<BTreeSet<_>>();

        let mut guard = self.axes.borrow_mut();
        guard.retain_axes(axes_keys);

        for axis in axes.into_iter().flat_map(Vec::from) {
            guard.construct_axis(
                &self.axes,
                &axis.key,
                &axis.label,
                axis.datums,
                axis.range,
                axis.visible_range,
                axis.hidden,
            );
        }

        if let Some(order) = order {
            guard.set_axes_order(&order);
        }
        drop(guard);

        self.update_matrix_buffer();
        self.update_axes_buffer();
        self.update_axes_lines_buffer();
    }

    async fn resize_drawing_area(&mut self, width: u32, height: u32, device_pixel_ratio: f32) {
        let scaled_width = (width as f32 * device_pixel_ratio) as u32;
        let scaled_height = (height as f32 * device_pixel_ratio) as u32;

        self.canvas_gpu.set_width(scaled_width);
        self.canvas_gpu.set_height(scaled_height);

        self.canvas_2d.set_width(scaled_width);
        self.canvas_2d.set_height(scaled_height);
        self.context_2d
            .scale(device_pixel_ratio as f64, device_pixel_ratio as f64)
            .unwrap();

        self.render_texture = self
            .device
            .create_texture(webgpu::TextureDescriptor::<'_, 2, 0> {
                label: Some(Cow::Borrowed("render texture")),
                dimension: None,
                format: self.render_texture.format(),
                mip_level_count: None,
                sample_count: Some(MSAA_SAMPLES),
                size: [
                    self.canvas_gpu.width() as usize,
                    self.canvas_gpu.height() as usize,
                ],
                usage: webgpu::TextureUsage::RENDER_ATTACHMENT,
                view_formats: None,
            });

        let guard = self.axes.borrow();
        guard.set_view_bounding_box(Aabb::new(
            Position::zero(),
            Position::new((width as f32, height as f32)),
        ));
        drop(guard);

        self.update_axes_config_buffer();
    }

    async fn pointer_down(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() || event.button() != 0 {
            return;
        }

        console::log_2(&"Pointer pressed".into(), &event);
    }

    async fn pointer_up(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() || event.button() != 0 {
            return;
        }

        console::log_2(&"Pointer released".into(), &event);
    }

    async fn pointer_move(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() {
            return;
        }

        console::log_2(&"Pointer moved".into(), &event);
    }
}

impl Renderer {
    fn update_matrix_buffer(&mut self) {
        let guard = self.axes.borrow();
        self.buffers
            .general
            .matrix
            .update(&self.device, &buffers::Matrices::new(guard.world_width()));
        self.redraw = true;
    }

    fn update_axes_buffer(&mut self) {
        let guard = self.axes.borrow();
        let mut axes = Vec::new();
        axes.resize_with(guard.visible_axes().len(), MaybeUninit::uninit);

        for ax in guard.visible_axes() {
            let range = ax.axis_line_range();
            let range = (
                range.0.transform(&ax.space_transformer()),
                range.1.transform(&ax.space_transformer()),
            );
            let range = [
                range.0.extract::<(f32, f32)>().1,
                range.1.extract::<(f32, f32)>().1,
            ];

            axes[ax.axis_index().unwrap()].write(buffers::Axis {
                expanded_val: if ax.is_expanded() { 1.0 } else { 0.0 },
                center_x: ax.get_world_offset(),
                position_x: wgsl::Vec2([ax.get_world_offset(), ax.get_world_offset()]), // TODO: Replace with extends of the axis.
                range_y: wgsl::Vec2(range),
            });
        }
        self.buffers.general.axes.update(&self.device, &axes);
        self.redraw = true;
    }

    fn update_axes_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let (width, height) = guard.axis_line_size();
        self.buffers.axes.config.update(
            &self.device,
            &buffers::LineConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                line_type: 1 + 2,
                color_mode: 0,
                color: wgsl::Vec3([0.8, 0.8, 0.8]),
            },
        );
        self.redraw = true;
    }

    fn update_axes_lines_buffer(&mut self) {
        let guard = self.axes.borrow();

        let num_lines = guard.visible_axes().len();
        let mut lines = Vec::<MaybeUninit<_>>::with_capacity(num_lines);
        unsafe { lines.set_len(num_lines) };

        for ax in guard.visible_axes() {
            let index = ax.axis_index().unwrap();
            let start_args_x = f32::from_ne_bytes((index as u32).to_ne_bytes());
            let end_args_x = f32::from_ne_bytes((index as u32).to_ne_bytes());

            lines[index].write(buffers::LineInfo {
                min_expanded_val: 0.0,
                start_args: wgsl::Vec2([start_args_x, 0.0]),
                end_args: wgsl::Vec2([end_args_x, 1.0]),
                offset_start: wgsl::Vec2([0.0, 0.0]),
                offset_end: wgsl::Vec2([0.0, 0.0]),
            });

            if ax.is_expanded() {
                let start_args_x = f32::from_ne_bytes((index as u32 + (1 << 31)).to_ne_bytes());
                let end_args_x = f32::from_ne_bytes((index as u32 + (1 << 31)).to_ne_bytes());
                lines.push(MaybeUninit::new(buffers::LineInfo {
                    min_expanded_val: 1.0,
                    start_args: wgsl::Vec2([start_args_x, 0.0]),
                    end_args: wgsl::Vec2([end_args_x, 1.0]),
                    offset_start: wgsl::Vec2([0.0, 0.0]),
                    offset_end: wgsl::Vec2([0.0, 0.0]),
                }));

                let start_args_x = f32::from_ne_bytes((index as u32 + (1 << 29)).to_ne_bytes());
                let end_args_x = f32::from_ne_bytes((index as u32 + (1 << 29)).to_ne_bytes());
                lines.push(MaybeUninit::new(buffers::LineInfo {
                    min_expanded_val: 1.0,
                    start_args: wgsl::Vec2([start_args_x, 0.0]),
                    end_args: wgsl::Vec2([end_args_x, 1.0]),
                    offset_start: wgsl::Vec2([0.0, 0.0]),
                    offset_end: wgsl::Vec2([0.0, 0.0]),
                }));
            }
        }

        self.buffers.axes.lines.update(&self.device, &lines);
        self.redraw = true;
    }
}
