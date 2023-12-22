use std::{borrow::Cow, cell::RefCell, collections::BTreeSet, mem::MaybeUninit, rc::Rc};

use async_channel::{Receiver, Sender};
use color_scale::ColorScaleDescriptor;
use colors::{Color, ColorOpaque, ColorQuery, ColorTransparent, SRgb, SRgbLinear, Xyz};
use coordinates::ScreenSpace;
use lerp::Lerp;
use wasm_bindgen::prelude::*;

use crate::coordinates::{Aabb, Length, Position};

mod webgpu;
mod wgsl;

mod action;
mod axis;
mod buffers;
mod color_bar;
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
    #[wasm_bindgen(js_name = updateData)]
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
    #[wasm_bindgen(js_name = pointerDown)]
    pub fn pointer_down(&self, event: web_sys::PointerEvent) {
        self.sender
            .send_blocking(Event::PointerDown { event })
            .expect("the channel should be open");
    }

    /// Spawns a `pointer_up` event.
    #[wasm_bindgen(js_name = pointerUp)]
    pub fn pointer_up(&self, event: web_sys::PointerEvent) {
        self.sender
            .send_blocking(Event::PointerUp { event })
            .expect("the channel should be open");
    }

    /// Spawns a `pointer_move` event.
    #[wasm_bindgen(js_name = pointerMove)]
    pub fn pointer_move(&self, event: web_sys::PointerEvent) {
        self.sender
            .send_blocking(Event::PointerMove { event })
            .expect("the channel should be open");
    }

    /// Sets an element to the default color.
    #[wasm_bindgen(js_name = setDefaultColor)]
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
    #[wasm_bindgen(js_name = setColorNamed)]
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
    #[wasm_bindgen(js_name = setColorValue)]
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

    #[wasm_bindgen(js_name = setDefaultColorScaleColor)]
    pub fn set_default_color_scale_color(&self) {
        let descriptor = DEFAULT_COLOR_SCALE();

        self.sender
            .send_blocking(Event::SetColorScale {
                color_space: ColorSpace::SRgb,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setColorScaleNamed)]
    pub fn set_color_scale_named(&self, name: &str) {
        let descriptor = ColorScaleDescriptor::Named(name.to_string().into());

        self.sender
            .send_blocking(Event::SetColorScale {
                color_space: ColorSpace::Xyz,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setColorScaleConstant)]
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

    #[wasm_bindgen(js_name = setColorScaleGradient)]
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

    #[wasm_bindgen(js_name = setDefaultSelectedDatumColoring)]
    pub fn set_default_selected_datum_coloring(&self) {
        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DEFAULT_DATUMS_COLORING(),
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setSelectedDatumColoringConstant)]
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

    #[wasm_bindgen(js_name = setSelectedDatumColoringAttribute)]
    pub fn set_selected_datum_coloring_attribute(&self, id: &str) {
        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DatumsColoring::Attribute(id.into()),
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setSelectedDatumColoringByProbability)]
    pub fn set_selected_datum_coloring_by_probability(&self) {
        self.sender
            .send_blocking(Event::SetDatumsColoring {
                coloring: DatumsColoring::Probability,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setColorBarVisibility)]
    pub fn set_color_bar_visibility(&self, visible: bool) {
        self.sender
            .send_blocking(Event::SetColorBarVisibility {
                visibility: visible,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = addLabel)]
    pub fn add_label(
        &self,
        id: String,
        color: Option<ColorDescription>,
        selection_threshold: Option<f32>,
    ) {
        let color = color.map(|color| {
            let ColorDescription {
                color_space,
                values,
                alpha,
            } = color;

            match color_space {
                ColorSpace::SRgb => ColorQuery::SRgb(values, alpha),
                ColorSpace::Xyz => ColorQuery::Xyz(values, alpha),
                ColorSpace::CieLab => ColorQuery::Lab(values, alpha),
                ColorSpace::CieLch => ColorQuery::Lch(values, alpha),
            }
        });

        self.sender
            .send_blocking(Event::AddLabel {
                id,
                color,
                selection_threshold,
                easing: selection::EasingType::Linear,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = removeLabel)]
    pub fn remove_label(&self, id: String) {
        self.sender
            .send_blocking(Event::RemoveLabel { id })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = switchActiveLabel)]
    pub fn switch_active_label(&self, id: String) {
        self.sender
            .send_blocking(Event::SwitchActiveLabel { id })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setLabelColor)]
    pub fn set_label_color(&self, id: String, color: Option<ColorDescription>) {
        let color = color.map(|color| {
            let ColorDescription {
                color_space,
                values,
                alpha,
            } = color;

            match color_space {
                ColorSpace::SRgb => ColorQuery::SRgb(values, alpha),
                ColorSpace::Xyz => ColorQuery::Xyz(values, alpha),
                ColorSpace::CieLab => ColorQuery::Lab(values, alpha),
                ColorSpace::CieLch => ColorQuery::Lch(values, alpha),
            }
        });

        self.sender
            .send_blocking(Event::SetLabelColor { id, color })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setLabelSelectionThreshold)]
    pub fn set_label_selection_threshold(&self, id: String, selection_threshold: Option<f32>) {
        self.sender
            .send_blocking(Event::SetLabelThreshold {
                id,
                selection_threshold,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setLabelEasing)]
    pub fn set_label_easing(&self, easing_type: Option<String>) {
        let easing = match easing_type.as_deref() {
            Some("linear") | None => selection::EasingType::Linear,
            Some("in") => selection::EasingType::EaseIn,
            Some("out") => selection::EasingType::EaseOut,
            Some("inout") => selection::EasingType::EaseInOut,
            _ => {
                web_sys::console::warn_1(&format!("unknown easing {easing_type:?}").into());
                selection::EasingType::Linear
            }
        };

        self.sender
            .send_blocking(Event::SetLabelEasing { easing })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setDebugOptions)]
    pub fn set_debug_options(&self, options: DebugOptions) {
        self.sender
            .send_blocking(Event::SetDebugOptions { options })
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
    gradient: Vec<(Option<f32>, ColorDescription)>,
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

    #[wasm_bindgen(js_name = withSample)]
    pub fn with_sample(&mut self, sample: Option<f32>, color: ColorDescription) {
        if let Some(sample) = sample {
            if self.gradient.is_empty() && sample != 0.0 {
                panic!("the first sample must be at position 0.0");
            }

            if !(0.0..=1.0).contains(&sample) {
                panic!("sample must lie in the [0, 1] range");
            }
        }

        self.gradient.push((sample, color));
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
    SetColorBarVisibility {
        visibility: bool,
    },
    AddLabel {
        id: String,
        color: Option<colors::ColorQuery<'static>>,
        selection_threshold: Option<f32>,
        easing: selection::EasingType,
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
        selection_threshold: Option<f32>,
    },
    SetLabelEasing {
        easing: selection::EasingType,
    },
    SetDebugOptions {
        options: DebugOptions,
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

    #[wasm_bindgen(js_name = newAxis)]
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

    #[wasm_bindgen(js_name = addOrder)]
    pub fn add_order(&mut self, key: &str) {
        self.order.push(key.into())
    }
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct DebugOptions {
    #[wasm_bindgen(js_name = showAxisBoundingBox)]
    pub show_axis_bounding_box: bool,
    #[wasm_bindgen(js_name = showLabelBoundingBox)]
    pub show_label_bounding_box: bool,
    #[wasm_bindgen(js_name = showCurvesBoundingBox)]
    pub show_curves_bounding_box: bool,
    #[wasm_bindgen(js_name = showAxisLineBoundingBox)]
    pub show_axis_line_bounding_box: bool,
    #[wasm_bindgen(js_name = showSelectionsBoundingBox)]
    pub show_selections_bounding_box: bool,
    #[wasm_bindgen(js_name = showColorBarBoundingBox)]
    pub show_color_bar_bounding_box: bool,
}

#[wasm_bindgen]
impl DebugOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        DebugOptions::default()
    }

    #[wasm_bindgen(js_name = anyIsActive)]
    pub fn any_is_active(&self) -> bool {
        self.show_axis_bounding_box
            || self.show_label_bounding_box
            || self.show_curves_bounding_box
            || self.show_axis_line_bounding_box
            || self.show_selections_bounding_box
            || self.show_color_bar_bounding_box
    }

    #[wasm_bindgen(js_name = noneIsActive)]
    pub fn none_is_active(&self) -> bool {
        !self.any_is_active()
    }
}

/// Implementation of the renderer for the parallel coordinates.
#[wasm_bindgen]
pub struct Renderer {
    callback: js_sys::Function,
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
    color_bar: color_bar::ColorBar,
    events: Vec<event::Event>,
    active_action: Option<action::Action>,
    active_label_idx: Option<usize>,
    labels: Vec<LabelInfo>,
    label_color_generator: LabelColorGenerator,
    datums_coloring: DatumsColoring,
    background_color: ColorTransparent<SRgb>,
    brush_color: ColorOpaque<Xyz>,
    unselected_color: ColorTransparent<Xyz>,
    debug: DebugOptions,
    pixel_ratio: f32,
    staging_data: StagingData,
}

struct LabelInfo {
    id: String,
    threshold_changed: bool,
    selection_threshold: f32,
    easing: selection::EasingType,
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
            0 => "rgb(228 26 28)",
            1 => "rgb(55 126 184)",
            2 => "rgb(77 175 74)",
            3 => "rgb(152 78 163)",
            4 => "rgb(255 127 0)",
            5 => "rgb(255 255 51)",
            6 => "rgb(166 86 40)",
            7 => "rgb(247 129 191)",
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
    color_bar_visibility: Vec<bool>,
    resize: Vec<(u32, u32, f32)>,
    label_additions: Vec<(
        String,
        Option<ColorQuery<'static>>,
        Option<f32>,
        selection::EasingType,
    )>,
    label_removals: Vec<String>,
    active_label: Vec<String>,
    label_color_changes: Vec<(String, Option<ColorQuery<'static>>)>,
    label_threshold_changes: Vec<(String, Option<f32>)>,
    label_easing_changes: Vec<selection::EasingType>,
    debug_options_changes: Vec<DebugOptions>,
}

#[wasm_bindgen]
impl Renderer {
    /// Constructs a new renderer.
    #[wasm_bindgen(constructor)]
    pub async fn new(
        callback: js_sys::Function,
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
            get_rem_length_screen.clone(),
            get_text_length_screen.clone(),
        );

        let color_bar = color_bar::ColorBar::new(
            client_width,
            client_height,
            get_rem_length_screen.clone(),
            get_text_length_screen.clone(),
        );

        let mut this = Self {
            callback,
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
            color_bar,
            events: Vec::default(),
            active_action: None,
            active_label_idx: None,
            labels: vec![],
            label_color_generator: LabelColorGenerator::default(),
            pixel_ratio: window.device_pixel_ratio() as f32,
            datums_coloring: DEFAULT_DATUMS_COLORING(),
            background_color: DEFAULT_BACKGROUND_COLOR(),
            brush_color: DEFAULT_BRUSH_COLOR(),
            unselected_color: DEFAULT_UNSELECTED_COLOR(),
            debug: DebugOptions::default(),
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
    #[wasm_bindgen(js_name = constructEventQueue)]
    pub fn construct_event_queue(&mut self) -> EventQueue {
        if self.event_queue.is_some() {
            panic!("EventQueue was already constructed.");
        }

        let (sx, rx) = async_channel::unbounded();
        self.event_queue = Some(rx);
        EventQueue { sender: sx }
    }

    /// Starts the event loop of the renderer.
    ///
    /// # Panics
    ///
    /// Panics if no [`EventQueue`] is associated with the renderer.
    #[wasm_bindgen(js_name = enterEventLoop)]
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
                Event::SetColorBarVisibility { visibility } => {
                    self.staging_data.color_bar_visibility.push(visibility);
                    self.events.push(event::Event::COLOR_BAR_VISIBILITY_CHANGE);
                }
                Event::AddLabel {
                    id,
                    color,
                    selection_threshold,
                    easing,
                } => {
                    self.staging_data.label_additions.push((
                        id,
                        color,
                        selection_threshold,
                        easing,
                    ));
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
                Event::SetLabelThreshold {
                    id,
                    selection_threshold,
                } => {
                    self.staging_data
                        .label_threshold_changes
                        .push((id, selection_threshold));
                    self.events.push(event::Event::LABEL_THRESHOLD_CHANGE);
                }
                Event::SetLabelEasing { easing } => {
                    self.staging_data.label_easing_changes.push(easing);
                    self.events.push(event::Event::LABEL_EASING_CHANGE);
                }
                Event::SetDebugOptions { options } => {
                    self.staging_data.debug_options_changes.push(options);
                    self.events.push(event::Event::DEBUG_OPTIONS_CHANGE);
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
        let axes = self.axes.borrow();
        let (viewport_start, viewport_size) = axes.viewport(self.pixel_ratio);
        let probabilities = if let Some(active_label_idx) = self.active_label_idx {
            self.buffers
                .datums()
                .probabilities(active_label_idx)
                .clone()
        } else {
            buffers::ProbabilitiesBuffer::empty(&self.device)
        };

        self.pipelines.render().datum_lines().render(
            self.background_color,
            self.buffers.shared().matrices(),
            self.buffers.datums().config(),
            self.buffers.shared().axes(),
            self.buffers.datums().lines(),
            self.buffers.datums().color_values(),
            &probabilities,
            self.buffers.shared().color_scale(),
            viewport_start,
            viewport_size,
            &self.device,
            encoder,
            msaa_texture,
            resolve_target,
        );
    }

    fn render_axes(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        let axes = self.axes.borrow();
        let (viewport_start, viewport_size) = axes.viewport(self.pixel_ratio);

        self.pipelines.render().axis_lines().render(
            self.buffers.shared().matrices(),
            self.buffers.axes().config(),
            self.buffers.shared().axes(),
            self.buffers.axes().lines(),
            viewport_start,
            viewport_size,
            &self.device,
            encoder,
            msaa_texture,
            resolve_target,
        );
    }

    fn render_selections(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        if self.active_label_idx.is_none() {
            return;
        }
        let active_label_idx = self.active_label_idx.unwrap();

        let axes = self.axes.borrow();
        let (viewport_start, viewport_size) = axes.viewport(self.pixel_ratio);

        self.pipelines.render().selections().render(
            self.buffers.shared().matrices(),
            self.buffers.selections().config(),
            self.buffers.shared().axes(),
            self.buffers.selections().lines(active_label_idx),
            self.buffers.shared().label_colors(),
            self.buffers.curves().sample_texture(active_label_idx),
            viewport_start,
            viewport_size,
            &self.device,
            encoder,
            msaa_texture,
            resolve_target,
        );
    }

    fn render_curve_segments(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        if self.active_label_idx.is_none() {
            return;
        }
        let active_label_idx = self.active_label_idx.unwrap();

        let axes = self.axes.borrow();
        let (viewport_start, viewport_size) = axes.viewport(self.pixel_ratio);
        let (min_curve_t, _) = axes.curve_t_range();

        let render = |label| {
            self.pipelines.render().curve_segments().render(
                label,
                active_label_idx,
                min_curve_t,
                self.buffers.shared().matrices(),
                self.buffers.shared().axes(),
                self.buffers.curves().lines(label),
                self.buffers.shared().label_colors(),
                viewport_start,
                viewport_size,
                &self.device,
                encoder,
                msaa_texture,
                resolve_target,
            );
        };

        for i in 0..self.labels.len() {
            if i == active_label_idx {
                continue;
            }
            render(i)
        }
        render(active_label_idx)
    }

    fn render_curves(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        if self.active_label_idx.is_none() {
            return;
        }
        let active_label_idx = self.active_label_idx.unwrap();

        let axes = self.axes.borrow();
        let (viewport_start, viewport_size) = axes.viewport(self.pixel_ratio);

        self.pipelines.render().curve_lines().render(
            self.buffers.shared().matrices(),
            self.buffers.curves().config(),
            self.buffers.shared().axes(),
            self.buffers.curves().lines(active_label_idx),
            viewport_start,
            viewport_size,
            &self.device,
            encoder,
            msaa_texture,
            resolve_target,
        );
    }

    fn render_color_bar(
        &self,
        encoder: &webgpu::CommandEncoder,
        msaa_texture: &webgpu::TextureView,
        resolve_target: &webgpu::TextureView,
    ) {
        if !self.color_bar.is_visible() {
            return;
        }

        let (viewport_start, viewport_size) = self.color_bar.bar_viewport(self.pixel_ratio);

        self.pipelines.render().color_bar().render(
            self.buffers.shared().color_scale(),
            self.background_color,
            viewport_start,
            viewport_size,
            &self.device,
            encoder,
            msaa_texture,
            resolve_target,
        );
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

    fn render_control_points(&self) {
        let active_label_idx = match self.active_label_idx {
            Some(x) => x,
            None => return,
        };

        self.context_2d.save();
        self.context_2d.set_fill_style(&"rgb(178 178 178)".into());

        let guard = self.axes.borrow();
        let radius = guard.control_points_radius().extract::<f32>() as f64;
        let screen_mapper = guard.space_transformer();

        for ax in guard.visible_axes() {
            if !ax.is_expanded() {
                continue;
            }

            let world_mapper = ax.space_transformer();
            let (sel_control_points, curve_control_points) = {
                let curve_builder = ax.borrow_selection_curve_builder(active_label_idx);
                (
                    Vec::from(curve_builder.get_selection_control_points()),
                    Vec::from(curve_builder.get_curve_control_points()),
                )
            };

            let (axis_start, axis_end) = ax.axis_line_range();
            for (rank, axis_value) in sel_control_points {
                if !(0.0..=1.0).contains(&axis_value) {
                    continue;
                }

                let rank_offset = ax.selection_offset_at_rank(rank);
                let position = axis_start.lerp(axis_end, axis_value) + rank_offset;
                let (x, y) = position
                    .transform(&world_mapper)
                    .transform(&screen_mapper)
                    .extract();

                self.context_2d.begin_path();
                self.context_2d
                    .arc(x as f64, y as f64, radius, 0.0, std::f64::consts::TAU)
                    .unwrap();
                self.context_2d.fill();
            }

            for [axis_value, curve_value] in curve_control_points {
                if !(0.0..=1.0).contains(&axis_value) {
                    continue;
                }

                let curve_offset = ax.curve_offset_at_curve_value(curve_value);
                let position = axis_start.lerp(axis_end, axis_value) + curve_offset;
                let (x, y) = position
                    .transform(&world_mapper)
                    .transform(&screen_mapper)
                    .extract();

                self.context_2d.begin_path();
                self.context_2d
                    .arc(x as f64, y as f64, radius, 0.0, std::f64::consts::TAU)
                    .unwrap();
                self.context_2d.fill();
            }
        }

        self.context_2d.restore();
    }

    fn render_color_bar_label(&self) {
        self.context_2d.save();
        self.context_2d.set_text_align("center");

        if !self.color_bar.is_visible() {
            return;
        }

        let label = self.color_bar.label();
        if label.is_empty() {
            return;
        }

        let position = self.color_bar.label_position();
        let (x, y) = position.extract();
        self.context_2d
            .fill_text(&label, x as f64, y as f64)
            .unwrap();
    }

    fn render_bounding_boxes(&self) {
        if self.debug.none_is_active() {
            return;
        }

        let axes = self.axes.borrow();
        let ((x, y), (w, h)) = axes.viewport(self.pixel_ratio);
        self.context_2d
            .stroke_rect(x as f64, y as f64, w as f64, h as f64);

        for axis in axes.visible_axes() {
            if self.debug.show_axis_bounding_box {
                let bounding_box = axis
                    .bounding_box(self.active_label_idx)
                    .transform(&axis.space_transformer())
                    .transform(&axes.space_transformer());
                let x = bounding_box.start().x;
                let y = bounding_box.end().y;
                let (w, h) = bounding_box.size().extract();
                self.context_2d
                    .stroke_rect(x as f64, y as f64, w as f64, h as f64);
            }

            if self.debug.show_label_bounding_box {
                let bounding_box = axis
                    .label_bounding_box()
                    .transform(&axis.space_transformer())
                    .transform(&axes.space_transformer());
                let x = bounding_box.start().x;
                let y = bounding_box.end().y;
                let (w, h) = bounding_box.size().extract();
                self.context_2d
                    .stroke_rect(x as f64, y as f64, w as f64, h as f64);
            }

            if self.debug.show_curves_bounding_box {
                let bounding_box = axis
                    .curves_bounding_box()
                    .transform(&axis.space_transformer())
                    .transform(&axes.space_transformer());
                let x = bounding_box.start().x;
                let y = bounding_box.end().y;
                let (w, h) = bounding_box.size().extract();
                self.context_2d
                    .stroke_rect(x as f64, y as f64, w as f64, h as f64);
            }

            if self.debug.show_axis_line_bounding_box {
                let bounding_box = axis
                    .axis_line_bounding_box()
                    .transform(&axis.space_transformer())
                    .transform(&axes.space_transformer());
                let x = bounding_box.start().x;
                let y = bounding_box.end().y;
                let (w, h) = bounding_box.size().extract();
                self.context_2d
                    .stroke_rect(x as f64, y as f64, w as f64, h as f64);
            }

            if self.debug.show_selections_bounding_box {
                if let Some(active_label_idx) = self.active_label_idx {
                    let bounding_box = axis
                        .selections_bounding_box(active_label_idx)
                        .transform(&axis.space_transformer())
                        .transform(&axes.space_transformer());
                    let x = bounding_box.start().x;
                    let y = bounding_box.end().y;
                    let (w, h) = bounding_box.size().extract();
                    self.context_2d
                        .stroke_rect(x as f64, y as f64, w as f64, h as f64);
                }
            }
        }

        if self.color_bar.is_visible() {
            let bounding_box = self.color_bar.bounding_box();
            let x = bounding_box.start().x;
            let y = bounding_box.end().y;
            let (w, h) = bounding_box.size().extract();
            self.context_2d
                .stroke_rect(x as f64, y as f64, w as f64, h as f64)
        }
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
        self.render_curve_segments(&command_encoder, &msaa_texture_view, &texture_view);
        self.render_curves(&command_encoder, &msaa_texture_view, &texture_view);
        self.render_color_bar(&command_encoder, &msaa_texture_view, &texture_view);

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
        self.render_control_points();
        self.render_color_bar_label();

        self.render_bounding_boxes();

        let mut probabilities_change = Vec::new();
        for label_idx in probabilities_changed.iter().copied() {
            let id = self.labels[label_idx].id.clone();
            let (probabilities, attributions) = self
                .extract_label_attribution_and_probability(label_idx)
                .await;
            probabilities_change.push((id, probabilities, attributions));
        }

        if !probabilities_change.is_empty() {
            // web_sys::console::log_1(&format!("{probabilities_change:?}").into());
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
            if events.is_empty() {
                continue;
            }

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

            if events.signaled(event::Event::COLOR_BAR_VISIBILITY_CHANGE) {
                let visible = self.staging_data.color_bar_visibility.pop().unwrap();
                self.set_color_bar_visibility(visible);
            }

            if events.signaled(event::Event::LABEL_ADDITION) {
                let (id, color, selection_threshold, easing_type) =
                    self.staging_data.label_additions.pop().unwrap();
                self.add_label(id, color, selection_threshold, easing_type);
            }

            if events.signaled(event::Event::LABEL_REMOVAL) {
                let id = self.staging_data.label_removals.pop().unwrap();
                self.remove_label(id);
            }

            if events.signaled(event::Event::ACTIVE_LABEL_CHANGE) {
                let id = self.staging_data.active_label.pop().unwrap();
                self.change_active_label(id);
            }

            if events.signaled(event::Event::LABEL_COLOR_CHANGE) {
                let (id, color) = self.staging_data.label_color_changes.pop().unwrap();
                self.change_label_color(id, color);
            }

            if events.signaled(event::Event::LABEL_THRESHOLD_CHANGE) {
                let (id, selection_threshold) =
                    self.staging_data.label_threshold_changes.pop().unwrap();
                self.change_label_threshold(id, selection_threshold);
            }

            if events.signaled(event::Event::LABEL_EASING_CHANGE) {
                let easing = self.staging_data.label_easing_changes.pop().unwrap();
                self.change_label_easing(easing);
            }

            if events.signaled(event::Event::DEBUG_OPTIONS_CHANGE) {
                let options = self.staging_data.debug_options_changes.pop().unwrap();
                self.change_debug_options(options);
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
                self.update_datum_lines_buffer();
            }
        }

        true
    }
}

// Callback events
impl Renderer {
    fn notify_easing_change(&self) {
        if let Some(active_label_idx) = self.active_label_idx {
            let easing = match self.labels[active_label_idx].easing {
                selection::EasingType::Linear => "linear",
                selection::EasingType::EaseIn => "in",
                selection::EasingType::EaseOut => "out",
                selection::EasingType::EaseInOut => "inout",
            };

            let this = JsValue::null();
            self.callback
                .call2(&this, &"easing".into(), &easing.into())
                .unwrap();
        }
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

        if let DatumsColoring::Attribute(id) = &self.datums_coloring {
            let axis = guard.axis(id).unwrap();
            self.color_bar.set_to_axis(&axis);
        }

        drop(guard);

        self.update_axes_config_buffer();
        self.update_datums_config_buffer();

        self.update_matrix_buffer();
        self.update_axes_buffer();
        self.update_axes_lines_buffer();
        self.update_datum_lines_buffer();
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
        self.update_datums_config_buffer();
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

        match &self.datums_coloring {
            DatumsColoring::Constant(_) => self.color_bar.set_to_empty(),
            DatumsColoring::Attribute(id) => {
                let axes = self.axes.borrow();
                let axis = axes.axis(id).unwrap();
                self.color_bar.set_to_axis(&axis);
            }
            DatumsColoring::Probability => {
                if let Some(active_label_idx) = self.active_label_idx {
                    let label = &self.labels[active_label_idx].id;
                    self.color_bar.set_to_label_probability(label);
                }
            }
        }

        let width = self.canvas_gpu.width() as f32 / self.pixel_ratio;
        let height = self.canvas_gpu.height() as f32 / self.pixel_ratio;
        if self.color_bar.is_visible() {
            let bounding_box = self.color_bar.bounding_box();
            let world_end_x = bounding_box.start().x;

            let guard = self.axes.borrow();
            guard.set_view_bounding_box(Aabb::new(
                Position::zero(),
                Position::new((world_end_x, height)),
            ));
            drop(guard);
        } else {
            let guard = self.axes.borrow();
            guard
                .set_view_bounding_box(Aabb::new(Position::zero(), Position::new((width, height))));
            drop(guard);
        }

        self.update_color_values_buffer();
        self.update_datums_config_buffer();
    }

    fn set_color_bar_visibility(&mut self, visible: bool) {
        let width = self.canvas_gpu.width() as f32 / self.pixel_ratio;
        let height = self.canvas_gpu.height() as f32 / self.pixel_ratio;

        self.color_bar.set_visible(visible);
        if self.color_bar.is_visible() {
            let bounding_box = self.color_bar.bounding_box();
            let world_end_x = bounding_box.start().x;

            let guard = self.axes.borrow();
            guard.set_view_bounding_box(Aabb::new(
                Position::zero(),
                Position::new((world_end_x, height)),
            ));
            drop(guard);
        } else {
            let guard = self.axes.borrow();
            guard
                .set_view_bounding_box(Aabb::new(Position::zero(), Position::new((width, height))));
            drop(guard);
        }
    }

    fn resize_drawing_area(&mut self, width: u32, height: u32, device_pixel_ratio: f32) {
        let scaled_width = (width as f32 * device_pixel_ratio) as u32;
        let scaled_height = (height as f32 * device_pixel_ratio) as u32;

        self.pixel_ratio = device_pixel_ratio;
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
                size: [scaled_width as usize, scaled_height as usize],
                usage: webgpu::TextureUsage::RENDER_ATTACHMENT,
                view_formats: None,
            });

        self.color_bar.set_screen_size(width as f32, height as f32);
        if self.color_bar.is_visible() {
            let bounding_box = self.color_bar.bounding_box();
            let world_end_x = bounding_box.start().x;

            let guard = self.axes.borrow();
            guard.set_view_bounding_box(Aabb::new(
                Position::zero(),
                Position::new((world_end_x, height as f32)),
            ));
            drop(guard);
        } else {
            let guard = self.axes.borrow();
            guard.set_view_bounding_box(Aabb::new(
                Position::zero(),
                Position::new((width as f32, height as f32)),
            ));
            drop(guard);
        }

        self.update_axes_config_buffer();
        self.update_datums_config_buffer();
        self.update_curves_config_buffer();
        self.update_selections_config_buffer();

        self.update_axes_buffer();
    }

    fn add_label(
        &mut self,
        id: String,
        color: Option<ColorQuery<'_>>,
        selection_threshold: Option<f32>,
        easing_type: selection::EasingType,
    ) {
        if self.labels.iter().any(|l| l.id == id) {
            panic!("id already exists");
        }

        let (color, color_dimmed) = if let Some(color) = color {
            let c = color.resolve();
            let c2 = LabelColorGenerator::dim(c);
            (c, c2)
        } else {
            self.label_color_generator.next()
        };

        let selection_threshold = selection_threshold.unwrap_or(std::f32::EPSILON);

        let label = LabelInfo {
            id,
            threshold_changed: true,
            selection_threshold,
            easing: easing_type,
            color,
            color_dimmed,
        };

        self.active_label_idx = Some(self.labels.len());
        self.labels.push(label);
        self.buffers.datums_mut().push_label(&self.device);
        self.buffers.curves_mut().push_label(&self.device);
        self.buffers.selections_mut().push_label(&self.device);

        let axes = self.axes.borrow();
        for axis in axes.visible_axes() {
            axis.push_label();
        }
        drop(axes);

        if let DatumsColoring::Probability = &self.datums_coloring {
            let label = &self.labels[self.active_label_idx.unwrap()].id;
            self.color_bar.set_to_label_probability(label);
        }

        self.update_selections_config_buffer();
        self.update_selection_lines_buffer();
        self.update_label_colors_buffer();

        self.notify_easing_change();
    }

    fn remove_label(&mut self, id: String) {
        let label_idx = self
            .labels
            .iter()
            .position(|l| l.id == id)
            .expect("no label with a matching id found");

        self.labels.remove(label_idx);
        self.buffers.datums_mut().remove_label(label_idx);
        self.buffers.curves_mut().remove_label(label_idx);
        self.buffers.selections_mut().remove_label(label_idx);

        if self.labels.is_empty() {
            self.active_label_idx = None;
        } else {
            self.active_label_idx = Some(self.labels.len() - 1);
        }

        let axes = self.axes.borrow();
        for axis in axes.visible_axes() {
            axis.remove_label(label_idx);
        }
        drop(axes);

        if let DatumsColoring::Probability = &self.datums_coloring {
            if let Some(active_label_idx) = self.active_label_idx {
                let label = &self.labels[active_label_idx].id;
                self.color_bar.set_to_label_probability(label);
            } else {
                self.color_bar.set_to_empty();
            }
        }

        self.update_selections_config_buffer();
        self.update_selection_lines_buffer();
        self.update_label_colors_buffer();

        self.notify_easing_change();
    }

    fn change_active_label(&mut self, id: String) {
        let label_idx = self
            .labels
            .iter()
            .position(|l| l.id == id)
            .expect("no label with a matching id found");
        self.active_label_idx = Some(label_idx);

        if let DatumsColoring::Probability = &self.datums_coloring {
            let label = &self.labels[self.active_label_idx.unwrap()].id;
            self.color_bar.set_to_label_probability(label);
        }

        self.update_selections_config_buffer();
        self.update_selection_lines_buffer();

        self.notify_easing_change();
    }

    fn change_label_color(&mut self, id: String, color: Option<ColorQuery<'_>>) {
        let label_idx = self
            .labels
            .iter()
            .position(|l| l.id == id)
            .expect("no label with a matching id found");

        let (color, color_dimmed) = if let Some(color) = color {
            let c = color.resolve();
            let c2 = LabelColorGenerator::dim(c);
            (c, c2)
        } else {
            self.label_color_generator.next()
        };

        self.labels[label_idx].color = color;
        self.labels[label_idx].color_dimmed = color_dimmed;

        self.update_selections_config_buffer();
        self.update_label_colors_buffer();
    }

    fn change_label_threshold(&mut self, id: String, selection_threshold: Option<f32>) {
        let label_idx = self
            .labels
            .iter()
            .position(|l| l.id == id)
            .expect("no label with a matching id found");

        let selection_threshold = selection_threshold.unwrap_or(std::f32::EPSILON);

        self.labels[label_idx].threshold_changed = true;
        self.labels[label_idx].selection_threshold = selection_threshold;

        if let Some(active_label_idx) = self.active_label_idx {
            if label_idx == active_label_idx {
                self.update_selections_config_buffer();
            }
        }
    }

    fn change_label_easing(&mut self, easing: selection::EasingType) {
        let label_idx = self.active_label_idx.expect("no label is present");
        self.labels[label_idx].easing = easing;

        let axes = self.axes.borrow();
        for axis in axes.visible_axes() {
            let curve_builder = axis.borrow_selection_curve_builder(label_idx);
            let curve = curve_builder.build(axis.visible_datums_range_normalized().into(), easing);
            axis.borrow_selection_curve_mut(label_idx).set_curve(curve);
        }
        drop(axes);

        self.update_selection_lines_buffer();
        self.notify_easing_change();
    }

    fn change_debug_options(&mut self, options: DebugOptions) {
        self.debug = options;
    }

    fn pointer_down(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() || event.button() != 0 {
            return;
        }

        self.create_action(event);
    }

    fn pointer_up(&mut self, event: web_sys::PointerEvent) {
        if !event.is_primary() || (event.button() != 0 && event.button() != -1) {
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
                    if let Some(active_label_idx) = self.active_label_idx {
                        self.active_action = Some(action::Action::new_select_selection_action(
                            axis,
                            selection_idx,
                            active_label_idx,
                            self.labels[active_label_idx].easing,
                        ))
                    }
                }
                axis::Element::SelectionControlPoint {
                    axis,
                    selection_idx,
                    segment_idx,
                } => {
                    if let Some(active_label_idx) = self.active_label_idx {
                        self.active_action =
                            Some(action::Action::new_select_selection_control_point_action(
                                axis,
                                selection_idx,
                                segment_idx,
                                active_label_idx,
                                self.labels[active_label_idx].easing,
                                event,
                            ))
                    }
                }
                axis::Element::CurveControlPoint {
                    axis,
                    selection_idx,
                    segment_idx,
                    is_upper,
                } => {
                    if let Some(active_label_idx) = self.active_label_idx {
                        self.active_action =
                            Some(action::Action::new_select_curve_control_point_action(
                                axis,
                                selection_idx,
                                segment_idx,
                                active_label_idx,
                                is_upper,
                                self.labels[active_label_idx].easing,
                            ))
                    }
                }
                axis::Element::AxisLine { axis } => {
                    if let Some(active_label_idx) = self.active_label_idx {
                        self.active_action = Some(action::Action::new_create_selection_action(
                            axis,
                            event,
                            active_label_idx,
                            self.labels[active_label_idx].easing,
                        ))
                    }
                }
            }
        }
    }

    fn update_action(&mut self, event: web_sys::PointerEvent) {
        if let Some(action) = &mut self.active_action {
            self.events.push(action.update(event));
        } else {
            let position =
                Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));

            let axes = self.axes.borrow();
            let element = axes.element_at_position(position, self.active_label_idx);
            match element {
                Some(axis::Element::Label { .. }) => self
                    .canvas_2d
                    .style()
                    .set_property("cursor", "ew-resize")
                    .unwrap(),
                Some(axis::Element::Selection { .. }) => self
                    .canvas_2d
                    .style()
                    .set_property("cursor", "ns-resize")
                    .unwrap(),
                Some(axis::Element::SelectionControlPoint { .. }) => self
                    .canvas_2d
                    .style()
                    .set_property("cursor", "row-resize")
                    .unwrap(),
                Some(axis::Element::CurveControlPoint { .. }) => self
                    .canvas_2d
                    .style()
                    .set_property("cursor", "move")
                    .unwrap(),
                Some(axis::Element::AxisLine { .. }) => self
                    .canvas_2d
                    .style()
                    .set_property("cursor", "crosshair")
                    .unwrap(),
                None => self
                    .canvas_2d
                    .style()
                    .set_property("cursor", "default")
                    .unwrap(),
            }
        }
    }

    fn finish_action(&mut self) {
        if let Some(action) = self.active_action.take() {
            self.events.push(action.finish());
        }
    }
}

// Shared buffers
impl Renderer {
    fn update_matrix_buffer(&mut self) {
        let guard = self.axes.borrow();
        self.buffers.shared_mut().matrices_mut().update(
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

            axes[ax.axis_index().unwrap()].write(buffers::Axis {
                expanded_val: if ax.is_expanded() { 1.0 } else { 0.0 },
                center_x: ax.world_offset(),
                position_x: wgsl::Vec2(extends),
                range_y: wgsl::Vec2(range),
            });
        }
        self.buffers
            .shared_mut()
            .axes_mut()
            .update(&self.device, &axes);
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
        self.buffers
            .shared_mut()
            .label_colors_mut()
            .update(&self.device, &colors);
    }

    fn update_color_scale_texture(
        &mut self,
        color_space: ColorSpace,
        scale: color_scale::ColorScale<colors::UnknownColorSpace>,
    ) {
        let color_scale_elements = scale
            .get_scale()
            .iter()
            .copied()
            .map(|(t, c)| buffers::ColorScaleElement {
                t,
                color: wgsl::Vec4(c.to_f32_with_alpha()),
            })
            .collect::<Vec<_>>();
        let color_scale_elements =
            buffers::ColorScaleElementBuffer::new(&self.device, &color_scale_elements);

        let encoder = self
            .device
            .create_command_encoder(webgpu::CommandEncoderDescriptor {
                label: Some("color scale sampling command encoder".into()),
            });
        self.pipelines.compute().color_scale_sampling().dispatch(
            color_space,
            self.buffers.shared_mut().color_scale_mut(),
            &color_scale_elements,
            &self.device,
            &encoder,
        );
        self.device.queue().submit(&[encoder.finish(None)]);
    }
}

// Axes lines buffers
impl Renderer {
    fn update_axes_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let (width, height) = guard.axis_line_size();
        self.buffers.axes_mut().config_mut().update(
            &self.device,
            &buffers::AxesConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
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
            lines[index].write(buffers::AxisLineInfo {
                axis: index as u32,
                axis_position: buffers::AxisLineInfo::CENTER,
                min_expanded_val: 0.0,
            });
            lines.push(MaybeUninit::new(buffers::AxisLineInfo {
                axis: index as u32,
                axis_position: buffers::AxisLineInfo::LEFT,
                min_expanded_val: 1.0,
            }));
            lines.push(MaybeUninit::new(buffers::AxisLineInfo {
                axis: index as u32,
                axis_position: buffers::AxisLineInfo::RIGHT,
                min_expanded_val: 1.0,
            }));
        }

        self.buffers
            .axes_mut()
            .lines_mut()
            .update(&self.device, &lines);
    }
}

// Datums buffers
impl Renderer {
    fn update_datums_config_buffer(&mut self) {
        let selection_threshold = if let Some(active_label_idx) = self.active_label_idx {
            self.labels[active_label_idx].selection_threshold
        } else {
            1.0
        };

        let guard = self.axes.borrow();
        let color_probabilities =
            matches!(self.datums_coloring, DatumsColoring::Probability) as u32;
        let (width, height) = guard.datums_line_size();
        self.buffers.datums_mut().config_mut().update(
            &self.device,
            &buffers::ValueLineConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                selection_threshold,
                color_probabilities,
                unselected_color: wgsl::Vec4(self.unselected_color.to_f32_with_alpha()),
            },
        );
    }

    fn update_datum_lines_buffer(&mut self) {
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

        self.buffers
            .datums_mut()
            .lines_mut()
            .update(&self.device, &lines)
    }

    fn update_color_values_buffer(&mut self) {
        let axes = self.axes.borrow();
        let num_datums = axes.num_datums();

        self.buffers
            .datums_mut()
            .color_values_mut()
            .resize(&self.device, num_datums);

        match &self.datums_coloring {
            DatumsColoring::Constant(x) => {
                let values = vec![*x; num_datums];
                self.buffers
                    .datums()
                    .color_values()
                    .update(&self.device, &values);
            }
            DatumsColoring::Attribute(key) => {
                let axis = axes.axis(key).expect("unknown attribute");
                let values = axis.datums_normalized();
                self.buffers
                    .datums()
                    .color_values()
                    .update(&self.device, values);
            }
            DatumsColoring::Probability => {}
        }
    }

    fn update_datums_buffer(&mut self) {
        let axes = self.axes.borrow();
        let num_datums = axes.num_datums();
        let num_visible_axes = axes.num_visible_axes();

        self.buffers
            .datums_mut()
            .datums_mut()
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
                .datums()
                .datums()
                .update(&self.device, datums, axis_idx);
        }
    }
}

// Curves buffers
impl Renderer {
    fn update_curves_config_buffer(&mut self) {
        let guard = self.axes.borrow();
        let (width, height) = guard.curve_line_size();
        self.buffers.curves_mut().config_mut().update(
            &self.device,
            &buffers::CurvesConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
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
        self.buffers.selections_mut().config_mut().update(
            &self.device,
            &buffers::SelectionConfig {
                line_width: wgsl::Vec2([width.0, height.0]),
                high_color: wgsl::Vec3(self.brush_color.to_f32()),
                low_color: wgsl::Vec3([0.0; 3]),
            },
        );
    }

    fn update_selection_lines_buffer(&mut self) {
        if self.active_label_idx.is_none() {
            return;
        }
        let active_label_idx = self.active_label_idx.unwrap();

        let guard = self.axes.borrow();

        let mut segments = Vec::new();
        for axis in guard.visible_axes() {
            let is_expanded = axis.is_expanded();
            let axis_index = axis
                .axis_index()
                .expect("all visible axes must have an index");
            let datums_range = axis.visible_datums_range_normalized().into();
            let curve_builder = axis.borrow_selection_curve_builder(active_label_idx);

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
                        color_idx: active_label_idx as u32,
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
            .selections_mut()
            .lines_mut(active_label_idx)
            .update(&self.device, &segments);
    }
}

// Probability
impl Renderer {
    fn sample_probability_curve(
        &mut self,
        encoder: &webgpu::CommandEncoder,
        label_idx: usize,
    ) -> bool {
        let axes = self.axes.borrow();
        self.buffers
            .curves_mut()
            .sample_texture_mut(label_idx)
            .set_num_curves(&self.device, axes.num_visible_axes());

        let mut changed = false;
        for axis in axes.visible_axes() {
            let mut selection_curve = axis.borrow_selection_curve_mut(label_idx);
            let spline = match selection_curve.get_changed_curve() {
                Some(s) => s,
                None => continue,
            };
            changed = true;

            let axis_idx = axis
                .axis_index()
                .expect("all visible axes must have an index");
            let probability_texture = self.buffers.curves().sample_texture(label_idx);

            let spline_segments = spline
                .segments()
                .iter()
                .map(|s| buffers::SplineSegment {
                    coefficients: wgsl::Vec4(s.coefficients),
                    bounds: wgsl::Vec2(s.bounds),
                    t_range: wgsl::Vec2(s.t_range),
                })
                .collect::<Vec<_>>();
            let spline_segments =
                buffers::SplineSegmentsBuffer::new(&self.device, &spline_segments);

            self.pipelines.compute().curve_spline_sampling().dispatch(
                axis_idx,
                probability_texture,
                &spline_segments,
                &self.device,
                encoder,
            );
        }

        changed
    }

    fn create_probability_curve_lines(
        &mut self,
        encoder: &webgpu::CommandEncoder,
        label_idx: usize,
    ) {
        let axes = self.axes.borrow();

        // Ensure that the buffer is large enough.
        let num_lines = axes.num_visible_axes()
            * buffers::ProbabilitySampleTexture::PROBABILITY_CURVE_RESOLUTION;
        self.buffers
            .curves_mut()
            .lines_mut(label_idx)
            .set_len(&self.device, num_lines);

        let lines_buffer = self.buffers.curves().lines(label_idx).buffer().clone();
        let samples = self.buffers.curves().sample_texture(label_idx).array_view();

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
            layout: self.pipelines.compute().create_curves.0.clone(),
        });

        let num_workgroups = ((num_lines + 63) / 64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(&self.pipelines.compute().create_curves.1);
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[num_workgroups]);
        pass.end();
    }

    fn apply_probability_curves(&mut self, encoder: &webgpu::CommandEncoder, label_idx: usize) {
        let axes = self.axes.borrow();
        let num_datums = axes.num_datums();

        // Ensure that the buffer is large enough.
        self.buffers
            .datums_mut()
            .probabilities_mut(label_idx)
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

        let curve_samples = self.buffers.curves().sample_texture(label_idx).array_view();
        let output_buffer = self.device.create_buffer(webgpu::BufferDescriptor {
            label: Some(Cow::Borrowed("curve application output")),
            size: std::mem::size_of::<u32>() * self.buffers.datums().datums().len(),
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
                        buffer: self.buffers.datums().datums().buffer().clone(),
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
                .compute()
                .compute_probability
                .apply_curve_bind_layout
                .clone(),
        });

        let num_workgroups = ((self.buffers.datums().datums().len() + 63) / 64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(
            &self
                .pipelines
                .compute()
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
                        buffer: self
                            .buffers
                            .datums()
                            .probabilities(label_idx)
                            .buffer()
                            .clone(),
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
                .compute()
                .compute_probability
                .reduce_bind_layout
                .clone(),
        });

        let num_workgroups = ((num_datums + 63) / 64) as u32;

        let pass = encoder.begin_compute_pass(None);
        pass.set_pipeline(&self.pipelines.compute().compute_probability.reduce_pipeline);
        pass.set_bind_group(0, &bind_group);
        pass.dispatch_workgroups(&[num_workgroups]);
        pass.end();
    }

    async fn extract_label_attribution_and_probability(
        &self,
        label_idx: usize,
    ) -> (Box<[f32]>, Box<[usize]>) {
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
            size: self.buffers.datums().probabilities(label_idx).size(),
            usage: webgpu::BufferUsage::MAP_READ | webgpu::BufferUsage::COPY_DST,
            mapped_at_creation: None,
        });
        encoder.copy_buffer_to_buffer(
            self.buffers.datums().probabilities(label_idx).buffer(),
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
            .filter(|(_, &p)| p >= self.labels[label_idx].selection_threshold)
            .map(|(i, _)| i)
            .collect::<Box<[_]>>();

        (probabilities, attribution)
    }

    fn update_probabilities(&mut self, encoder: &webgpu::CommandEncoder) -> Box<[usize]> {
        let mut changed = Vec::new();
        for i in 0..self.labels.len() {
            let curve_changed = self.sample_probability_curve(encoder, i);

            let threshold_changed = std::mem::replace(&mut self.labels[i].threshold_changed, false);
            if !curve_changed {
                if threshold_changed {
                    changed.push(i);
                }

                continue;
            }

            changed.push(i);
            self.create_probability_curve_lines(encoder, i);
            self.apply_probability_curves(encoder, i);
        }

        changed.into()
    }
}
