use async_channel::{Receiver, Sender};
use wasm_bindgen::prelude::*;

use web_sys::console;

mod webgpu;
mod wgsl;

mod axis;
mod color_scale;
mod colors;
mod coordinates;
mod lerp;
mod pipelines;

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

/// Implementation of the renderer for the parallel coordinates.
#[wasm_bindgen]
pub struct Renderer {
    canvas_gpu: web_sys::HtmlCanvasElement,
    canvas_2d: web_sys::HtmlCanvasElement,
    context_gpu: web_sys::GpuCanvasContext,
    context_2d: web_sys::CanvasRenderingContext2d,
    device: webgpu::Device,
    events: Option<Receiver<Event>>,
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

        let navigator = web_sys::window().unwrap().navigator();
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

        Self {
            canvas_gpu,
            canvas_2d,
            context_gpu,
            context_2d,
            device,
            events: None,
        }
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
    async fn render(&mut self, completion: Sender<()>) {
        completion
            .send(())
            .await
            .expect("the channel should be open");
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
