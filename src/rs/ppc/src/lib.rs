use std::{borrow::Cow, cell::RefCell, collections::BTreeSet, mem::MaybeUninit, rc::Rc};

use async_channel::{Receiver, Sender};
use color_scale::ColorScaleDescriptor;
use colors::{Color, ColorOpaque, ColorQuery, ColorTransparent, SRgb, SRgbLinear, Xyz};
use coordinates::ScreenSpace;
use wasm_bindgen::prelude::*;

use crate::coordinates::{Aabb, Length, Position};

mod webgpu;
mod wgsl;

mod action;
mod axis;
mod buffers;
mod color_scale;
mod colors;
mod coordinates;
mod event;
mod lerp;
mod pipelines;
mod selection;
mod spline;

const DEFAULT_BACKGROUND_COLOR: fn() -> ColorTransparent<SRgb> =
    || ColorTransparent::<SRgb>::from_f32_with_alpha([1.0, 1.0, 1.0, 1.0]);

const DEFAULT_BRUSH_COLOR: fn() -> ColorOpaque<Xyz> = || {
    let query = ColorQuery::Named("fuchsia".into());
    query.resolve()
};

const DEFAULT_UNSELECTED_COLOR: fn() -> ColorTransparent<Xyz> = || {
    let query = ColorQuery::Css("rgb(211 211 211 0.2)".into());
    query.resolve_with_alpha()
};

const DEFAULT_DATUMS_COLORING: fn() -> DatumsColoring = || DatumsColoring::Constant(0.5);

const DEFAULT_COLOR_SCALE: fn() -> ColorScaleDescriptor<'static> =
    || ColorScaleDescriptor::Constant(ColorQuery::Named("blue".into()));

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

    /// Sets an element to the default color.
    pub fn set_default_color(&self, element: Element) {
        let color = match element {
            Element::Background => {
                let color = DEFAULT_BACKGROUND_COLOR();
                ColorQuery::SRgb(color.to_f32(), Some(color.alpha))
            }
            Element::Brush => {
                let color = DEFAULT_BRUSH_COLOR();
                ColorQuery::Xyz(color.to_f32(), None)
            }
            Element::Unselected => {
                let color = DEFAULT_UNSELECTED_COLOR();
                ColorQuery::Xyz(color.to_f32(), Some(color.alpha))
            }
        };
        let event = match element {
            Element::Background => Event::SetBackgroundColor { color },
            Element::Brush => Event::SetBrushColor { color },
            Element::Unselected => Event::SetUnselectedColor { color },
        };

        self.sender
            .send_blocking(event)
            .expect("the channel should be open");
    }

    /// Sets the color of an element from a color name string.
    pub fn set_color_named(&self, element: Element, color: &str) {
        let color = ColorQuery::Named(color.to_string().into());
        let event = match element {
            Element::Background => Event::SetBackgroundColor { color },
            Element::Brush => Event::SetBrushColor { color },
            Element::Unselected => Event::SetUnselectedColor { color },
        };

        self.sender
            .send_blocking(event)
            .expect("the channel should be open");
    }

    /// Sets the color of an element from a color value.
    pub fn set_color_value(&self, element: Element, color: ColorDescription) {
        let ColorDescription {
            color_space,
            values,
            alpha,
        } = color;

        let color = match color_space {
            ColorSpace::SRgb => ColorQuery::SRgb(values, alpha),
            ColorSpace::Xyz => ColorQuery::Xyz(values, alpha),
            ColorSpace::CieLab => ColorQuery::Lab(values, alpha),
            ColorSpace::CieLch => ColorQuery::Lch(values, alpha),
        };
        let event = match element {
            Element::Background => Event::SetBackgroundColor { color },
            Element::Brush => Event::SetBrushColor { color },
            Element::Unselected => Event::SetUnselectedColor { color },
        };

        self.sender
            .send_blocking(event)
            .expect("the channel should be open");
    }

    pub fn set_default_color_scale_color(&self) {
        let descriptor = DEFAULT_COLOR_SCALE();

        self.sender
            .send_blocking(Event::SetColorScale {
                color_space: ColorSpace::SRgb,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    pub fn set_color_scale_named(&self, name: &str) {
        let descriptor = ColorScaleDescriptor::Named(name.to_string().into());

        self.sender
            .send_blocking(Event::SetColorScale {
                color_space: ColorSpace::SRgb,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    pub fn set_color_scale_constant(&self, color: ColorDescription) {
        let ColorDescription {
            color_space,
            values,
            alpha,
        } = color;

        let color = match color_space {
            ColorSpace::SRgb => ColorQuery::SRgb(values, alpha),
            ColorSpace::Xyz => ColorQuery::Xyz(values, alpha),
            ColorSpace::CieLab => ColorQuery::Lab(values, alpha),
            ColorSpace::CieLch => ColorQuery::Lch(values, alpha),
        };
        let descriptor = ColorScaleDescriptor::Constant(color);

        self.sender
            .send_blocking(Event::SetColorScale {
                color_space: ColorSpace::SRgb,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    pub fn set_color_scale_gradient(&self, scale: ColorScaleDescription) {
        let ColorScaleDescription {
            color_space,
            gradient,
        } = scale;
        let gradient = gradient
            .into_iter()
            .map(|(t, color)| {
                let ColorDescription {
                    color_space,
                    values,
                    alpha,
                } = color;

                let color = match color_space {
                    ColorSpace::SRgb => ColorQuery::SRgb(values, alpha),
                    ColorSpace::Xyz => ColorQuery::Xyz(values, alpha),
                    ColorSpace::CieLab => ColorQuery::Lab(values, alpha),
                    ColorSpace::CieLch => ColorQuery::Lch(values, alpha),
                };

                (t, color)
            })
            .collect::<Vec<_>>();

        let descriptor = ColorScaleDescriptor::Gradient(gradient);
        self.sender
            .send_blocking(Event::SetColorScale {
                color_space,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    pub fn set_default_selected_datum_coloring(&self) {
        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DEFAULT_DATUMS_COLORING(),
            })
            .expect("the channel should be open");
    }

    pub fn set_selected_datum_coloring_constant(&self, value: f32) {
        if !(0.0..=1.0).contains(&value) {
            panic!("constant must lie in the interval [0, 1], got '{value}'");
        }

        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DatumsColoring::Constant(value),
            })
            .expect("the channel should be open");
    }

    pub fn set_selected_datum_coloring_attribute(&self, id: &str) {
        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DatumsColoring::Attribute(id.into()),
            })
            .expect("the channel should be open");
    }

    pub fn set_selected_datum_coloring_by_probability(&self) {
        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DatumsColoring::Probability,
            })
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

#[wasm_bindgen]

pub struct ColorScaleDescription {
    color_space: ColorSpace,
    gradient: Vec<(f32, ColorDescription)>,
}

#[wasm_bindgen]
pub struct ColorDescription {
    color_space: ColorSpace,
    values: [f32; 3],
    alpha: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ColorSpace {
    SRgb,
    Xyz,
    CieLab,
    CieLch,
}

#[wasm_bindgen]
impl ColorScaleDescription {
    #[wasm_bindgen(constructor)]
    pub fn new(color_space: &str) -> Self {
        let color_space = match color_space {
            "srgb" => ColorSpace::SRgb,
            "xyz" => ColorSpace::Xyz,
            "cie_lab" => ColorSpace::CieLab,
            "cie_lch" => ColorSpace::CieLch,
            _ => panic!("unknown color space {color_space:?}"),
        };

        Self {
            color_space,
            gradient: Vec::new(),
        }
    }

    pub fn with_sample(&mut self, sample: f32, color: ColorDescription) {
        if !(0.0..=1.0).contains(&sample) {
            panic!("sample must lie in the [0, 1] range");
        }

        if self.gradient.is_empty() {
            self.gradient.push((sample, color));
        } else {
            match self
                .gradient
                .binary_search_by(|&(a, _)| a.partial_cmp(&sample).unwrap())
            {
                Ok(_) => panic!("sample is already contained in the color scale gradient"),
                Err(p) => self.gradient.insert(p, (sample, color)),
            }
        }
    }
}

#[wasm_bindgen]
impl ColorDescription {
    #[wasm_bindgen(constructor)]
    pub fn new(color_space: &str, values: &[f32]) -> Self {
        assert!(values.len() == 3 || values.len() == 4);

        let color_space = match color_space {
            "srgb" => ColorSpace::SRgb,
            "xyz" => ColorSpace::Xyz,
            "cie_lab" => ColorSpace::CieLab,
            "cie_lch" => ColorSpace::CieLch,
            _ => panic!("unknown color space {color_space:?}"),
        };

        let opaque = [values[0], values[1], values[2]];
        let alpha = if values.len() == 4 {
            Some(values[3])
        } else {
            None
        };

        Self {
            color_space,
            values: opaque,
            alpha,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Element {
    Background,
    Brush,
    Unselected,
}

enum Event {
    Exit,
    Resize {
        width: u32,
        height: u32,
        device_pixel_ratio: f32,
    },
    UpdateData {
        axes: Option<Box<[AxisDef]>>,
        order: Option<Box<[String]>>,
    },
    SetBackgroundColor {
        color: colors::ColorQuery<'static>,
    },
    SetBrushColor {
        color: colors::ColorQuery<'static>,
    },
    SetUnselectedColor {
        color: colors::ColorQuery<'static>,
    },
    SetColorScale {
        color_space: ColorSpace,
        scale: color_scale::ColorScaleDescriptor<'static>,
    },
    SetDatumsColoring {
        coloring: DatumsColoring,
    },
    AddLabel {
        id: String,
        color: Option<colors::ColorQuery<'static>>,
    },
    RemoveLabel {
        id: String,
    },
    SwitchActiveLabel {
        id: String,
    },
    SetLabelColor {
        id: String,
        color: Option<colors::ColorQuery<'static>>,
    },
    SetLabelThreshold {
        id: String,
        threshold: f32,
    },
    Draw {
        completion: Sender<()>,
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

enum DatumsColoring {
    Constant(f32),
    Attribute(String),
    Probability,
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
    event_queue: Option<Receiver<Event>>,
    axes: Rc<RefCell<axis::Axes>>,
    events: Vec<event::Event>,
    active_action: Option<action::Action>,
    labeling_threshold: f32,
    active_label_idx: usize,
    labels: Vec<LabelInfo>,
    label_color_generator: LabelColorGenerator,
    datums_coloring: DatumsColoring,
    background_color: ColorTransparent<SRgb>,
    brush_color: ColorOpaque<Xyz>,
    unselected_color: ColorTransparent<Xyz>,
    staging_data: StagingData,
}

struct LabelInfo {
    id: String,
    threshold_changed: bool,
    selection_threshold: f32,
    color: ColorOpaque<Xyz>,
    color_dimmed: ColorOpaque<Xyz>,
}

#[derive(Debug, Default)]
struct LabelColorGenerator {
    idx: usize,
}

impl LabelColorGenerator {
    fn next(&mut self) -> (ColorOpaque<Xyz>, ColorOpaque<Xyz>) {
        let css_string = match self.idx {
            0 => "rgb(166 206 227)",
            1 => "rgb(31 120 180)",
            2 => "rgb(178 223 138)",
            3 => "rgb(51 160 44)",
            4 => "rgb(251 154 153)",
            5 => "rgb(227 26 28)",
            6 => "rgb(253 191 111)",
            7 => "rgb(255 127 0)",
            8 => "rgb(202 178 214)",
            9 => "rgb(106 61 154)",
            10 => "rgb(255 255 153)",
            11 => "rgb(177 89 40)",
            _ => unreachable!(),
        };

        self.idx = (self.idx + 1) % 12;
        let color = ColorQuery::Css(css_string.into()).resolve();
        (color, Self::dim(color))
    }

    fn dim(color: ColorOpaque<Xyz>) -> ColorOpaque<Xyz> {
        let mut lab = color.transform::<colors::CieLab>();
        lab.values.l *= 0.7;
        lab.transform()
    }
}

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct StagingData {
    update_data: Vec<(Option<Box<[AxisDef]>>, Option<Box<[String]>>)>,
    background_color: Vec<ColorQuery<'static>>,
    brush_color: Vec<ColorQuery<'static>>,
    unselected_color: Vec<ColorQuery<'static>>,
    color_scale: Vec<(ColorSpace, color_scale::ColorScaleDescriptor<'static>)>,
    datums_coloring: Vec<DatumsColoring>,
    resize: Vec<(u32, u32, f32)>,
    label_additions: Vec<(String, Option<ColorQuery<'static>>)>,
    label_removals: Vec<String>,
    active_label: Vec<String>,
    label_color_changes: Vec<(String, Option<ColorQuery<'static>>)>,
    label_threshold_changes: Vec<(String, f32)>,
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

        let (color, color_dimmed) = LabelColorGenerator::default().next();
        let labels = vec![LabelInfo {
            id: "default".into(),
            threshold_changed: true,
            selection_threshold: f32::EPSILON,
            color,
            color_dimmed,
        }];

        let mut this = Self {
            canvas_gpu,
            canvas_2d,
            context_gpu,
            context_2d,
            device,
            pipelines,
            render_texture,
            buffers,
            event_queue: None,
            axes,
            events: Vec::default(),
            active_action: None,
            labeling_threshold: 0.0,
            active_label_idx: 0,
            labels,
            label_color_generator: LabelColorGenerator::default(),
            datums_coloring: DEFAULT_DATUMS_COLORING(),
            background_color: DEFAULT_BACKGROUND_COLOR(),
            brush_color: DEFAULT_BRUSH_COLOR(),
            unselected_color: DEFAULT_UNSELECTED_COLOR(),
            staging_data: StagingData::default(),
        };

        this.update_matrix_buffer();
        this.update_axes_buffer();
        this.update_label_colors_buffer();

        this.update_axes_config_buffer();
        this.update_axes_lines_buffer();
        this.update_curves_config_buffer();
        this.update_selections_config_buffer();

        this
    }

    /// Constructs a new event queue for this renderer.
    ///
    /// # Panics
    ///
    /// Panics if called multiple times.
    pub fn construct_event_queue(&mut self) -> EventQueue {
        if self.event_queue.is_some() {
            panic!("EventQueue was already constructed.");
        }

        let (sx, rx) = async_channel::unbounded();
        self.event_queue = Some(rx);

        let queue = EventQueue { sender: sx };
        queue.set_default_color(Element::Background);
        queue.set_default_color(Element::Brush);
        queue.set_default_color(Element::Unselected);
        queue.set_default_color_scale_color();
        queue.set_default_selected_datum_coloring();

        queue
    }

    /// Starts the event loop of the renderer.
    ///
    /// # Panics
    ///
    /// Panics if no [`EventQueue`] is associated with the renderer.
    pub async fn enter_event_loop(&mut self) {
        if self.event_queue.is_none() {
            panic!("EventQueue was not initialized.");
        }

        let events = self.event_queue.take().unwrap();
        loop {
            match events.recv().await.expect("the channel should be open") {
                Event::Exit => break,
                Event::Resize {
                    width,
                    height,
                    device_pixel_ratio,
                } => {
                    self.staging_data
                        .resize
                        .push((width, height, device_pixel_ratio));
                    self.events.push(event::Event::RESIZE);
                }
                Event::UpdateData { axes, order } => {
                    self.staging_data.update_data.push((axes, order));
                    self.events.push(event::Event::DATA_UPDATE);
                }
                Event::SetBackgroundColor { color } => {
                    self.staging_data.background_color.push(color);
                    self.events.push(event::Event::BACKGROUND_COLOR_CHANGE);
                }
                Event::SetBrushColor { color } => {
                    self.staging_data.brush_color.push(color);
                    self.events.push(event::Event::BRUSH_COLOR_CHANGE);
                }
                Event::SetUnselectedColor { color } => {
                    self.staging_data.unselected_color.push(color);
                    self.events.push(event::Event::UNSELECTED_COLOR_CHANGE);
                }
                Event::SetColorScale { color_space, scale } => {
                    self.staging_data.color_scale.push((color_space, scale));
                    self.events.push(event::Event::COLOR_SCALE_CHANGE);
                }
                Event::SetDatumsColoring { coloring } => {
                    self.staging_data.datums_coloring.push(coloring);
                    self.events.push(event::Event::DATUMS_COLORING_CHANGE);
                }
                Event::AddLabel { id, color } => {
                    self.staging_data.label_additions.push((id, color));
                    self.events.push(event::Event::LABEL_ADDITION);
                }
                Event::RemoveLabel { id } => {
                    self.staging_data.label_removals.push(id);
                    self.events.push(event::Event::LABEL_REMOVAL);
                }
                Event::SwitchActiveLabel { id } => {
                    self.staging_data.active_label.push(id);
                    self.events.push(event::Event::ACTIVE_LABEL_CHANGE);
                }
                Event::SetLabelColor { id, color } => {
                    self.staging_data.label_color_changes.push((id, color));
                    self.events.push(event::Event::LABEL_COLOR_CHANGE);
                }
                Event::SetLabelThreshold { id, threshold } => {
                    self.staging_data
                        .label_threshold_changes
                        .push((id, threshold));
                    self.events.push(event::Event::LABEL_THRESHOLD_CHANGE);
                }
                Event::Draw { completion } => self.render(completion).await,
                Event::PointerDown { event } => self.pointer_down(event),
                Event::PointerUp { event } => self.pointer_up(event),
                Event::PointerMove { event } => self.pointer_move(event),
            }
        }

        self.event_queue = Some(events);
    }
}

// Rendering
impl Renderer {
    fn render_datums(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        let num_lines = self.buffers.values.lines.len();
        if num_lines == 0 {
            return;
        }

        let descriptor = webgpu::RenderPassDescriptor {
            label: Some(Cow::Borrowed("datums render pass descriptor")),
            color_attachments: [webgpu::RenderPassColorAttachments {
                clear_value: Some(self.background_color.to_f32_with_alpha()),
                load_op: webgpu::RenderPassLoadOp::Clear,
                store_op: webgpu::RenderPassStoreOp::Store,
                resolve_target: Some(resolve_target.clone()),
                view: msaa_texture.clone(),
            }],
            max_draw_count: None,
        };

        let color_scale = self.buffers.values.color_scale.view();
        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("datum lines bind group")),
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
                        buffer: self.buffers.values.config.buffer().clone(),
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
                        buffer: self.buffers.values.lines.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 4,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.values.color_values.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 5,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.values.probabilities.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 6,
                    resource: webgpu::BindGroupEntryResource::TextureView(color_scale),
                },
                webgpu::BindGroupEntry {
                    binding: 7,
                    resource: webgpu::BindGroupEntryResource::Sampler(
                        self.pipelines.render_pipelines.draw_value_lines.2.clone(),
                    ),
                },
            ],
            layout: self.pipelines.render_pipelines.draw_value_lines.0.clone(),
        });

        let pass = encoder.begin_render_pass(descriptor);
        pass.set_pipeline(&self.pipelines.render_pipelines.draw_value_lines.1);
        pass.set_bind_group(0, &bind_group);
        pass.draw_with_instance_count(6, num_lines);
        pass.end();
    }

    fn render_axes(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
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
                resolve_target: Some(resolve_target.clone()),
                view: msaa_texture.clone(),
            }],
            max_draw_count: None,
        });
        pass.set_pipeline(&self.pipelines.render_pipelines.draw_lines.1);
        pass.set_bind_group(0, &bind_group);
        pass.draw_with_instance_count(6, num_lines);
        pass.end();
    }

    fn render_selections(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        let lines_buffer = self.buffers.selections.lines(self.active_label_idx);
        let num_lines = lines_buffer.len();
        if num_lines == 0 {
            return;
        }

        let descriptor = webgpu::RenderPassDescriptor {
            label: Some(Cow::Borrowed("selections render pass descriptor")),
            color_attachments: [webgpu::RenderPassColorAttachments {
                clear_value: None,
                load_op: webgpu::RenderPassLoadOp::Load,
                store_op: webgpu::RenderPassStoreOp::Store,
                resolve_target: Some(resolve_target.clone()),
                view: msaa_texture.clone(),
            }],
            max_draw_count: None,
        };

        let active_curve_idx = self.active_label_idx;
        let probability_curve_samples =
            self.buffers.curves.sample_textures[active_curve_idx].array_view();
        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("datum lines bind group")),
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
                        buffer: self.buffers.selections.config.buffer().clone(),
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
                        buffer: lines_buffer.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 4,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.general.colors.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 5,
                    resource: webgpu::BindGroupEntryResource::TextureView(
                        probability_curve_samples,
                    ),
                },
                webgpu::BindGroupEntry {
                    binding: 6,
                    resource: webgpu::BindGroupEntryResource::Sampler(
                        self.pipelines.render_pipelines.draw_selections.2.clone(),
                    ),
                },
            ],
            layout: self.pipelines.render_pipelines.draw_selections.0.clone(),
        });

        let pass = encoder.begin_render_pass(descriptor);
        pass.set_pipeline(&self.pipelines.render_pipelines.draw_selections.1);
        pass.set_bind_group(0, &bind_group);
        pass.draw_with_instance_count(6, num_lines);
        pass.end();
    }

    fn render_curves(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        let active_curve_idx = self.active_label_idx;
        let num_lines = self.buffers.curves.lines[active_curve_idx].len();
        if num_lines == 0 {
            return;
        }

        let descriptor = webgpu::RenderPassDescriptor {
            label: Some(Cow::Borrowed("curves render pass descriptor")),
            color_attachments: [webgpu::RenderPassColorAttachments {
                clear_value: None,
                load_op: webgpu::RenderPassLoadOp::Load,
                store_op: webgpu::RenderPassStoreOp::Store,
                resolve_target: Some(resolve_target.clone()),
                view: msaa_texture.clone(),
            }],
            max_draw_count: None,
        };

        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("datum lines bind group")),
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
                        buffer: self.buffers.curves.config.buffer().clone(),
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
                        buffer: self.buffers.curves.lines[active_curve_idx].buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self.pipelines.render_pipelines.draw_lines.0.clone(),
        });

        let pass = encoder.begin_render_pass(descriptor);
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

    async fn render(&mut self, completion: Sender<()>) {
        let redraw = self.handle_events();
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

        // Update the probability curves and probabilities.
        let probabilities_changed = self.update_probabilities(&command_encoder);

        // Draw the main view into the framebuffer.
        self.render_datums(&command_encoder, &msaa_texture_view, &texture_view);
        self.render_axes(&command_encoder, &msaa_texture_view, &texture_view);
        self.render_selections(&command_encoder, &msaa_texture_view, &texture_view);
        self.render_curves(&command_encoder, &msaa_texture_view, &texture_view);

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

        if probabilities_changed {
            let (probabilities, attributions) =
                self.extract_label_attribution_and_probability().await;
            web_sys::console::log_1(&format!("{probabilities:?}").into());
            web_sys::console::log_1(&format!("{attributions:?}").into());
        }

        completion
            .send(())
            .await
            .expect("the channel should be open");
    }
}

// Event handling
impl Renderer {
    fn handle_events(&mut self) -> bool {
        if self.events.is_empty() {
            return false;
        }

        let events = std::mem::take(&mut self.events);
        for events in events {
            // External events.
            if events.signaled(event::Event::RESIZE) {
                let (width, height, device_pixel_ratio) = self.staging_data.resize.pop().unwrap();
                self.resize_drawing_area(width, height, device_pixel_ratio);
            }

            if events.signaled(event::Event::DATA_UPDATE) {
                let (axes, order) = self.staging_data.update_data.pop().unwrap();
                self.update_data(axes, order);
            }

            if events.signaled(event::Event::BACKGROUND_COLOR_CHANGE) {
                let color = self.staging_data.background_color.pop().unwrap();
                self.set_background_color(color);
            }

            if events.signaled(event::Event::BRUSH_COLOR_CHANGE) {
                let color = self.staging_data.brush_color.pop().unwrap();
                self.set_brush_color(color);
            }

            if events.signaled(event::Event::UNSELECTED_COLOR_CHANGE) {
                let color = self.staging_data.unselected_color.pop().unwrap();
                self.set_unselected_color(color);
            }

            if events.signaled(event::Event::COLOR_SCALE_CHANGE) {
                let (color_space, scale) = self.staging_data.color_scale.pop().unwrap();
                self.set_color_scale(color_space, scale);
            }

            if events.signaled(event::Event::DATUMS_COLORING_CHANGE) {
                let coloring = self.staging_data.datums_coloring.pop().unwrap();
                self.set_datums_coloring(coloring);
            }

            if events.signaled(event::Event::LABEL_ADDITION) {
                todo!()
            }

            if events.signaled(event::Event::LABEL_REMOVAL) {
                todo!()
            }

            if events.signaled(event::Event::ACTIVE_LABEL_CHANGE) {
                todo!()
            }

            if events.signaled(event::Event::LABEL_COLOR_CHANGE) {
                todo!()
            }

            if events.signaled(event::Event::LABEL_THRESHOLD_CHANGE) {
                todo!()
            }

            // Internal events.
            let update_axes_buffer = events.signaled_any(&[
                event::Event::AXIS_STATE_CHANGE,
                event::Event::AXIS_POSITION_CHANGE,
                event::Event::SELECTIONS_CHANGE,
            ]);
            if update_axes_buffer {
                self.update_axes_buffer();
            }

            let update_selection_lines_buffer = events.signaled_any(&[
                event::Event::AXIS_STATE_CHANGE,
                event::Event::SELECTIONS_CHANGE,
            ]);
            if update_selection_lines_buffer {
                self.update_selection_lines_buffer();
            }

            let update_datum_lines_buffer = events.signaled_any(&[
                event::Event::AXIS_STATE_CHANGE,
                event::Event::AXIS_ORDER_CHANGE,
            ]);
            if update_datum_lines_buffer {
                self.update_value_lines_buffer();
            }
        }

        true
    }
}

// External events
impl Renderer {
    fn update_data(&mut self, axes: Option<Box<[AxisDef]>>, order: Option<Box<[String]>>) {
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

        self.update_axes_config_buffer();
        self.update_values_config_buffer();

        self.update_matrix_buffer();
        self.update_axes_buffer();
        self.update_axes_lines_buffer();
        self.update_value_lines_buffer();
        self.update_datums_buffer();
        self.update_color_values_buffer();

        self.update_curves_config_buffer();

        self.update_selections_config_buffer();
        self.update_selection_lines_buffer();
    }

    fn set_background_color(&mut self, color: ColorQuery<'_>) {
        let color = color.resolve_with_alpha::<SRgb>();
        self.background_color = color;
    }

    fn set_brush_color(&mut self, color: ColorQuery<'_>) {
        let color = color.resolve::<Xyz>();
        self.brush_color = color;
        self.update_selections_config_buffer();
    }

    fn set_unselected_color(&mut self, color: ColorQuery<'_>) {
        let color = color.resolve_with_alpha::<Xyz>();
        self.unselected_color = color;
        self.update_values_config_buffer();
    }

    fn set_color_scale(&mut self, color_space: ColorSpace, scale: ColorScaleDescriptor<'_>) {
        let scale = match color_space {
            ColorSpace::SRgb => scale
                .to_color_scale::<SRgbLinear>()
                .transform::<colors::UnknownColorSpace>(),
            ColorSpace::Xyz => scale
                .to_color_scale::<Xyz>()
                .transform::<colors::UnknownColorSpace>(),
            ColorSpace::CieLab => scale
                .to_color_scale::<colors::CieLab>()
                .transform::<colors::UnknownColorSpace>(),
            ColorSpace::CieLch => scale
                .to_color_scale::<colors::CieLch>()
                .transform::<colors::UnknownColorSpace>(),
        };

        self.update_color_scale_texture(color_space, scale);
    }

    fn set_datums_coloring(&mut self, coloring: DatumsColoring) {
        self.datums_coloring = coloring;

        self.update_color_values_buffer();
        self.update_values_config_buffer();
    }

    fn resize_drawing_area(&mut self, width: u32, height: u32, device_pixel_ratio: f32) {
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
        self.update_values_config_buffer();
        self.update_curves_config_buffer();
        self.update_selections_config_buffer();

        self.update_axes_buffer();
    }

    fn pointer_down(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() || event.button() != 0 {
            return;
        }

        self.create_action(event);
    }

    fn pointer_up(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() || event.button() != 0 {
            return;
        }

        self.finish_action();
    }

    fn pointer_move(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() {
            return;
        }

        self.update_action(event);
    }
}

// Actions
impl Renderer {
    fn create_action(&mut self, event: web_sys::PointerEvent) {
        self.finish_action();

        let position =
            Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));

        let axes = self.axes.borrow();
        let element = axes.element_at_position(position, self.active_label_idx);
        if let Some(element) = element {
            match element {
                axis::Element::Label { axis } => {
                    self.active_action = Some(action::Action::new_move_axis_action(
                        axis,
                        event,
                        self.active_label_idx,
                    ))
                }
                axis::Element::Selection {
                    axis,
                    selection_idx,
                } => {
                    self.active_action = Some(action::Action::new_select_selection_action(
                        axis,
                        selection_idx,
                        self.active_label_idx,
                    ))
                }
                axis::Element::AxisLine { axis } => {
                    self.active_action = Some(action::Action::new_create_selection_action(
                        axis,
                        event,
                        self.active_label_idx,
                    ))
                }
            }
        }
    }

    fn update_action(&mut self, event: web_sys::PointerEvent) {
        if let Some(action) = &mut self.active_action {
            self.events.push(action.update(event));
        }
    }

    fn finish_action(&mut self) {
        if let Some(action) = self.active_action.take() {
            self.events.push(action.finish());
        }
    }
}

// General buffers
impl Renderer {
    fn update_matrix_buffer(&mut self) {
        let guard = self.axes.borrow();
        self.buffers.general.matrix.update(
            &self.device,
            &buffers::Matrices::new(guard.num_visible_axes()),
        );
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
            let extends = ax
                .expanded_extends(self.active_label_idx)
                .transform(&ax.space_transformer());
            let extends = [extends.start().x, extends.end().x];
            // web_sys::console::log_1(
            //     &format!(
            //         "Axis: expanded '{}', center '{}', position '{extends:?}', range '{range:?}', id {}",
            //         ax.is_expanded(),
            //         ax.world_offset(),
            //         ax.axis_index().unwrap()
            //     )
            //     .into(),
            // );

            axes[ax.axis_index().unwrap()].write(buffers::Axis {
                expanded_val: if ax.is_expanded() { 1.0 } else { 0.0 },
                center_x: ax.world_offset(),
                position_x: wgsl::Vec2(extends),
                range_y: wgsl::Vec2(range),
            });
        }
        self.buffers.general.axes.update(&self.device, &axes);
    }

    fn update_label_colors_buffer(&mut self) {
        let colors = self
            .labels
            .iter()
            .map(|l| buffers::LabelColor {
                color_high: wgsl::Vec4(l.color.with_alpha(0.5).to_f32_with_alpha()),
                color_low: wgsl::Vec4(l.color_dimmed.with_alpha(0.5).to_f32_with_alpha()),
            })
            .collect::<Vec<_>>();
        self.buffers.general.colors.update(&self.device, &colors);
    }
}

// Axes lines buffers
impl Renderer {
    fn update_axes_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let (width, height) = guard.axis_line_size();
        self.buffers.axes.config.update(
            &self.device,
            &buffers::LineConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                line_type: 3,
                color_mode: 0,
                color: wgsl::Vec3([0.8, 0.8, 0.8]),
            },
        );
    }

    fn update_axes_lines_buffer(&mut self) {
        let guard = self.axes.borrow();

        let num_lines = guard.visible_axes().len();
        let mut lines = Vec::<MaybeUninit<_>>::with_capacity(num_lines * 3);
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

        self.buffers.axes.lines.update(&self.device, &lines);
    }
}

// Values buffers
impl Renderer {
    fn update_values_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let color_probabilities =
            matches!(self.datums_coloring, DatumsColoring::Probability) as u32;
        let (width, height) = guard.datums_line_size();
        self.buffers.values.config.update(
            &self.device,
            &buffers::ValueLineConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                selection_threshold: self.labeling_threshold,
                color_probabilities,
                unselected_color: wgsl::Vec4(self.unselected_color.to_f32_with_alpha()),
            },
        );
    }

    fn update_value_lines_buffer(&mut self) {
        let axes = self.axes.borrow();

        // Compute the curves.
        let mut curves = vec![Vec::new(); axes.num_datums()];
        let mut axis_indices = Vec::new();
        for axis in axes.visible_axes() {
            let axis_idx = axis
                .axis_index()
                .expect("all visible axes must have an axis index");
            axis_indices.push(axis_idx);

            let (start, end) = axis.visible_datums_range_normalized();
            let range = start..=end;

            for (i, datum) in axis.datums_normalized().iter().enumerate() {
                if range.contains(datum) {
                    curves[i].push(*datum);
                } else {
                    curves[i].push(f32::NAN);
                }
            }
        }

        // Filter curves with values outside of the requested range.
        let curves = curves
            .into_iter()
            .filter(|c| !c.iter().any(|d| d.is_nan()))
            .collect::<Vec<_>>();

        // Write the curves into a buffer.
        let num_curve_segments = axes.num_visible_axes().saturating_sub(1);
        let num_lines = num_curve_segments * curves.len();

        let mut lines = Vec::with_capacity(num_lines);
        for (i, curve) in curves.into_iter().enumerate() {
            for (values, indices) in curve.windows(2).zip(axis_indices.windows(2)) {
                let curve_idx = i as u32;
                let start_axis = indices[0] as u32;
                let end_axis = indices[1] as u32;
                let start_value = values[0];
                let end_value = values[1];

                lines.push(buffers::ValueLine {
                    curve_idx,
                    start_axis,
                    start_value,
                    end_axis,
                    end_value,
                });
            }
        }

        self.buffers.values.lines.update(&self.device, &lines)
    }

    fn update_color_values_buffer(&mut self) {
        let axes = self.axes.borrow();
        let num_datums = axes.num_datums();

        self.buffers
            .values
            .color_values
            .resize(&self.device, num_datums);

        match &self.datums_coloring {
            DatumsColoring::Constant(x) => {
                let values = vec![*x; num_datums];
                self.buffers
                    .values
                    .color_values
                    .update(&self.device, &values);
            }
            DatumsColoring::Attribute(key) => {
                let axis = axes.axis(key).expect("unknown attribute");
                let values = axis.datums_normalized();
                self.buffers
                    .values
                    .color_values
                    .update(&self.device, values);
            }
            DatumsColoring::Probability => {}
        }
    }

    fn update_color_scale_texture(
        &mut self,
        color_space: ColorSpace,
        scale: color_scale::ColorScale<colors::UnknownColorSpace>,
    ) {
        let scale = scale
            .get_scale()
            .iter()
            .copied()
            .map(|(t, c)| buffers::ColorScaleElement {
                t,
                color: wgsl::Vec4(c.to_f32_with_alpha()),
            })
            .collect::<Vec<_>>();
        let color_scale_buffer = buffers::ColorScaleElementBuffer::new(&self.device, &scale);
        let scale_view = self.buffers.values.color_scale.view();

        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("color scale sampling bind group")),
            entries: [
                webgpu::BindGroupEntry {
                    binding: 0,
                    resource: webgpu::BindGroupEntryResource::TextureView(scale_view.clone()),
                },
                webgpu::BindGroupEntry {
                    binding: 1,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: color_scale_buffer.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self
                .pipelines
                .compute_pipelines
                .sample_color_scale
                .0
                .clone(),
        });

        let encoder = self
            .device
            .create_command_encoder(webgpu::CommandEncoderDescriptor {
                label: Some(Cow::Borrowed("color scale creation command encoder")),
            });

        let num_workgroups =
            ((buffers::ColorScaleTexture::COLOR_SCALE_RESOLUTION + 63) / 64) as u32;

        let sampling_pass = encoder.begin_compute_pass(None);
        sampling_pass.set_pipeline(&self.pipelines.compute_pipelines.sample_color_scale.1);
        sampling_pass.set_bind_group(0, &bind_group);
        sampling_pass.dispatch_workgroups(&[num_workgroups]);
        sampling_pass.end();

        // Transform the color scale to XYZ.
        if color_space != ColorSpace::Xyz {
            let transformed_scale = buffers::ColorScaleTexture::new(&self.device);
            let transformed_scale_view = transformed_scale.view();

            let color_space: u32 = match color_space {
                ColorSpace::SRgb => 0,
                ColorSpace::Xyz => 1,
                ColorSpace::CieLab => 2,
                ColorSpace::CieLch => 3,
            };
            let color_space_buffer = self.device.create_buffer(webgpu::BufferDescriptor {
                label: Some(Cow::Borrowed("color space buffer")),
                size: std::mem::size_of::<u32>(),
                usage: webgpu::BufferUsage::UNIFORM | webgpu::BufferUsage::COPY_DST,
                mapped_at_creation: None,
            });
            self.device
                .queue()
                .write_buffer_single(&color_space_buffer, 0, &color_space);

            let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
                label: Some(Cow::Borrowed("color scale transformation bind group")),
                entries: [
                    webgpu::BindGroupEntry {
                        binding: 0,
                        resource: webgpu::BindGroupEntryResource::TextureView(scale_view),
                    },
                    webgpu::BindGroupEntry {
                        binding: 1,
                        resource: webgpu::BindGroupEntryResource::TextureView(
                            transformed_scale_view,
                        ),
                    },
                    webgpu::BindGroupEntry {
                        binding: 2,
                        resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                            buffer: color_space_buffer,
                            offset: None,
                            size: None,
                        }),
                    },
                ],
                layout: self
                    .pipelines
                    .compute_pipelines
                    .transform_color_scale
                    .0
                    .clone(),
            });

            let transformation_pass = encoder.begin_compute_pass(None);
            transformation_pass
                .set_pipeline(&self.pipelines.compute_pipelines.transform_color_scale.1);
            transformation_pass.set_bind_group(0, &bind_group);
            transformation_pass.dispatch_workgroups(&[num_workgroups]);
            transformation_pass.end();

            self.buffers.values.color_scale = transformed_scale;
        }

        self.device.queue().submit(&[encoder.finish(None)]);
    }

    fn update_datums_buffer(&mut self) {
        let axes = self.axes.borrow();
        let num_datums = axes.num_datums();
        let num_visible_axes = axes.num_visible_axes();

        self.buffers
            .values
            .datums
            .resize(&self.device, num_datums, num_visible_axes);

        if num_datums == 0 || num_visible_axes == 0 {
            return;
        }

        for axis in axes.visible_axes() {
            let datums = axis.datums_normalized();
            let axis_idx = axis
                .axis_index()
                .expect("all visible axes should have an index");
            self.buffers
                .values
                .datums
                .update(&self.device, datums, axis_idx);
        }
    }
}

// Curves buffers
impl Renderer {
    fn update_curves_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let (width, height) = guard.curve_line_size();
        self.buffers.curves.config.update(
            &self.device,
            &buffers::LineConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                line_type: 23,
                color_mode: 0,
                color: wgsl::Vec3([1.0, 0.8, 0.8]),
            },
        );
    }
}

// Selection buffers
impl Renderer {
    fn update_selections_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let (width, height) = guard.selections_line_size();
        self.buffers.selections.config.update(
            &self.device,
            &buffers::SelectionConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                high_color: wgsl::Vec3(self.brush_color.to_f32()),
                low_color: wgsl::Vec3([0.0; 3]),
            },
        );
    }

    fn update_selection_lines_buffer(&mut self) {
        let guard = self.axes.borrow();

        let mut segments = Vec::new();
        for axis in guard.visible_axes() {
            let is_expanded = axis.is_expanded();
            let axis_index = axis
                .axis_index()
                .expect("all visible axes must have an index");
            let datums_range = axis.visible_datums_range_normalized().into();
            let curve_builder = axis.borrow_selection_curve_builder(self.active_label_idx);

            if is_expanded {
                for segment in curve_builder
                    .get_selection_segment_info_in_range(datums_range)
                    .iter()
                {
                    let (offset_x, range, use_low_color) = match *segment {
                        selection::SelectionSegmentInfo::Visible { rank, range } => {
                            (axis.selection_offset_at_rank(rank).x, range, 0)
                        }
                        selection::SelectionSegmentInfo::Invisible { rank, range } => {
                            (axis.selection_offset_at_rank(rank).x, range, 1)
                        }
                    };

                    segments.push(buffers::SelectionLineInfo {
                        axis: axis_index as u32,
                        use_color: 1,
                        use_left: 0,
                        offset_x,
                        color_idx: self.active_label_idx as u32,
                        use_low_color,
                        range: wgsl::Vec2(range),
                    });
                }

                for range in curve_builder
                    .get_visible_selection_ranges_in_range(datums_range)
                    .iter()
                {
                    segments.push(buffers::SelectionLineInfo {
                        axis: axis_index as u32,
                        use_color: 0,
                        use_left: 1,
                        offset_x: 0.0,
                        color_idx: 0,
                        use_low_color: 0,
                        range: wgsl::Vec2(*range),
                    });
                }
            } else {
                for range in curve_builder
                    .get_visible_selection_ranges_in_range(datums_range)
                    .iter()
                {
                    segments.push(buffers::SelectionLineInfo {
                        axis: axis_index as u32,
                        use_color: 0,
                        use_left: 0,
                        offset_x: 0.0,
                        color_idx: 0,
                        use_low_color: 0,
                        range: wgsl::Vec2(*range),
                    });
                }
            }
        }
        self.buffers
            .selections
            .lines_mut(self.active_label_idx)
            .update(&self.device, &segments);
    }
}

// Probability
impl Renderer {
    fn sample_probability_curve(&mut self, encoder: &webgpu::CommandEncoder) -> bool {
        let active_curve_idx = self.active_label_idx;
        let axes = self.axes.borrow();
        self.buffers.curves.sample_textures[active_curve_idx]
            .set_num_curves(&self.device, axes.num_visible_axes());

        let mut changed = false;
        for axis in axes.visible_axes() {
            let mut selection_curve = axis.borrow_selection_curve_mut(self.active_label_idx);
            let spline = match selection_curve.get_changed_curve() {
                Some(s) => s,
                None => continue,
            };
            changed = true;

            let spline_segments = spline
                .segments()
                .iter()
                .map(|s| buffers::SplineSegment {
                    coefficients: wgsl::Vec4(s.coefficients),
                    bounds: wgsl::Vec2(s.bounds),
                    t_range: wgsl::Vec2(s.t_range),
                })
                .collect::<Vec<_>>();
            let spline_segments_buffer =
                buffers::SplineSegmentsBuffer::new(&self.device, &spline_segments);

            let axis_idx = axis
                .axis_index()
                .expect("all visible axes must have an index");
            let probability_texture =
                self.buffers.curves.sample_textures[active_curve_idx].axis_view(axis_idx);

            // Sample the curve.
            let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
                label: Some(Cow::Borrowed(
                    "probability curve spline sampling bind group",
                )),
                entries: [
                    webgpu::BindGroupEntry {
                        binding: 0,
                        resource: webgpu::BindGroupEntryResource::TextureView(probability_texture),
                    },
                    webgpu::BindGroupEntry {
                        binding: 1,
                        resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                            buffer: spline_segments_buffer.buffer().clone(),
                            offset: None,
                            size: None,
                        }),
                    },
                ],
                layout: self.pipelines.compute_pipelines.sample_curves.0.clone(),
            });

            let num_workgroups = ((buffers::ProbabilitySampleTexture::PROBABILITY_CURVE_RESOLUTION
                + 63)
                / 64) as u32;

            let pass = encoder.begin_compute_pass(None);
            pass.set_pipeline(&self.pipelines.compute_pipelines.sample_curves.1);
            pass.set_bind_group(0, &bind_group);
            pass.dispatch_workgroups(&[num_workgroups]);
            pass.end();
        }

        changed
    }

    fn create_probability_curve_lines(&mut self, encoder: &webgpu::CommandEncoder) {
        let axes = self.axes.borrow();
        let active_curve_idx = self.active_label_idx;

        // Ensure that the buffer is large enough.
        let num_lines = axes.num_visible_axes()
            * buffers::ProbabilitySampleTexture::PROBABILITY_CURVE_RESOLUTION;
        self.buffers.curves.lines[active_curve_idx].set_len(&self.device, num_lines);

        let lines_buffer = self.buffers.curves.lines[active_curve_idx].buffer().clone();
        let samples = self.buffers.curves.sample_textures[active_curve_idx].array_view();

        // Fill the buffer using the compute pipeline.
        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("probability curve line sampling bind group")),
            entries: [
                webgpu::BindGroupEntry {
                    binding: 0,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: lines_buffer,
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 1,
                    resource: webgpu::BindGroupEntryResource::TextureView(samples),
                },
            ],
            layout: self.pipelines.compute_pipelines.create_curves.0.clone(),
        });

        let num_workgroups = ((num_lines + 63) / 64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(&self.pipelines.compute_pipelines.create_curves.1);
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[num_workgroups]);
        pass.end();
    }

    fn apply_probability_curves(&mut self, encoder: &webgpu::CommandEncoder) {
        let axes = self.axes.borrow();
        let num_datums = axes.num_datums();
        let active_curve_idx = self.active_label_idx;

        // Ensure that the buffer is large enough.
        self.buffers
            .values
            .probabilities
            .set_len(&self.device, num_datums);

        // If there are no datums we can skip the rest.
        if num_datums == 0 {
            return;
        }

        let num_datums_buffer = self.device.create_buffer(webgpu::BufferDescriptor {
            label: Some(Cow::Borrowed("num datums")),
            size: std::mem::size_of::<u32>(),
            usage: webgpu::BufferUsage::UNIFORM | webgpu::BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        self.device
            .queue()
            .write_buffer_single(&num_datums_buffer, 0, &(num_datums as u32));

        let curve_samples = self.buffers.curves.sample_textures[active_curve_idx].array_view();
        let output_buffer = self.device.create_buffer(webgpu::BufferDescriptor {
            label: Some(Cow::Borrowed("curve application output")),
            size: std::mem::size_of::<u32>() * self.buffers.values.datums.len(),
            usage: webgpu::BufferUsage::STORAGE,
            mapped_at_creation: None,
        });

        // First we apply the curves to each value.
        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("probability curve application bind group")),
            entries: [
                webgpu::BindGroupEntry {
                    binding: 0,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: output_buffer.clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 1,
                    resource: webgpu::BindGroupEntryResource::TextureView(curve_samples),
                },
                webgpu::BindGroupEntry {
                    binding: 2,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.values.datums.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 3,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: num_datums_buffer.clone(),
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self
                .pipelines
                .compute_pipelines
                .compute_probability
                .apply_curve_bind_layout
                .clone(),
        });

        let num_workgroups = ((self.buffers.values.datums.len() + 63) / 64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(
            &self
                .pipelines
                .compute_pipelines
                .compute_probability
                .apply_curve_pipeline,
        );
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[num_workgroups]);
        pass.end();

        // Then we reduce the value to a single one per curve.
        let bind_group = self.device.create_bind_group(webgpu::BindGroupDescriptor {
            label: Some(Cow::Borrowed("probability reduction bind group")),
            entries: [
                webgpu::BindGroupEntry {
                    binding: 0,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: self.buffers.values.probabilities.buffer().clone(),
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 1,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: output_buffer,
                        offset: None,
                        size: None,
                    }),
                },
                webgpu::BindGroupEntry {
                    binding: 2,
                    resource: webgpu::BindGroupEntryResource::Buffer(webgpu::BufferBinding {
                        buffer: num_datums_buffer,
                        offset: None,
                        size: None,
                    }),
                },
            ],
            layout: self
                .pipelines
                .compute_pipelines
                .compute_probability
                .reduce_bind_layout
                .clone(),
        });

        let num_workgroups = ((num_datums + 63) / 64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(
            &self
                .pipelines
                .compute_pipelines
                .compute_probability
                .reduce_pipeline,
        );
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[num_workgroups]);
        pass.end();
    }

    async fn extract_label_attribution_and_probability(&self) -> (Box<[f32]>, Box<[usize]>) {
        {
            let axes = self.axes.borrow();
            if axes.num_datums() == 0 {
                return (Box::new([]), Box::new([]));
            }
        }

        // Create a temporary staging buffer for mapping the computed probability.
        let encoder = self
            .device
            .create_command_encoder(webgpu::CommandEncoderDescriptor { label: None });
        let staging_buffer = self.device.create_buffer(webgpu::BufferDescriptor {
            label: Some(Cow::Borrowed("probability staging buffer")),
            size: self.buffers.values.probabilities.size(),
            usage: webgpu::BufferUsage::MAP_READ | webgpu::BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        encoder.copy_buffer_to_buffer(
            self.buffers.values.probabilities.buffer(),
            0,
            &staging_buffer,
            0,
            staging_buffer.size(),
        );
        self.device.queue().submit(&[encoder.finish(None)]);

        // Read the computed probabilities.
        staging_buffer.map_async(webgpu::MapMode::READ).await;
        let probabilities = unsafe { staging_buffer.get_mapped_range::<f32>() };
        let attribution = probabilities
            .iter()
            .enumerate()
            .filter(|(_, &p)| p >= self.labeling_threshold)
            .map(|(i, _)| i)
            .collect::<Box<[_]>>();

        (probabilities, attribution)
    }

    fn update_probabilities(&mut self, encoder: &webgpu::CommandEncoder) -> bool {
        let curve_changed = self.sample_probability_curve(encoder);

        if !curve_changed {
            return false;
        }

        self.create_probability_curve_lines(encoder);
        self.apply_probability_curves(encoder);
        true
    }
}
