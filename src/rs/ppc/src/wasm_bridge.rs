//! `Wasm` bridge types.
use std::rc::Rc;

use async_channel::Sender;
use wasm_bindgen::prelude::*;

use crate::{
    color_scale,
    colors::{self, Color},
    selection,
};

pub enum DataColorMode {
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

pub struct AxisDef {
    pub key: Box<str>,
    pub label: Box<str>,
    pub points: Box<[f32]>,
    pub range: Option<(f32, f32)>,
    pub visible_range: Option<(f32, f32)>,
    pub ticks: Option<Vec<(f32, Option<Rc<str>>)>>,
    pub hidden: bool,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new_axis(
        &mut self,
        key: &str,
        label: &str,
        points: Box<[f32]>,
        range: Option<Box<[f32]>>,
        visible_range: Option<Box<[f32]>>,
        ticks: Option<AxisTicksDef>,
        hidden: Option<bool>,
    ) {
        let ticks = if let Some(ticks) = ticks {
            assert!(
                ticks.tick_labels.is_empty()
                    || ticks.tick_positions.len() == ticks.tick_labels.len()
            );

            let positions = ticks.tick_positions.into_iter();
            let labels = ticks
                .tick_labels
                .into_iter()
                .map(Some)
                .chain(std::iter::repeat(None));

            Some(positions.zip(labels).collect::<Vec<_>>())
        } else {
            None
        };

        self.axes.push(AxisDef {
            key: key.into(),
            label: label.into(),
            points,
            range: range.map(|v| (v[0], v[1])),
            visible_range: visible_range.map(|v| (v[0], v[1])),
            ticks,
            hidden: hidden.unwrap_or(false),
        });
    }

    #[wasm_bindgen(js_name = addOrder)]
    pub fn add_order(&mut self, key: &str) {
        self.order.push(key.into())
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct AxisTicksDef {
    tick_positions: Vec<f32>,
    tick_labels: Vec<Rc<str>>,
}

#[wasm_bindgen]
impl AxisTicksDef {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tick_positions: Vec::new(),
            tick_labels: Vec::new(),
        }
    }

    #[wasm_bindgen(js_name = addTick)]
    pub fn add_tick(&mut self, value: f32) {
        assert!(!self.tick_positions.contains(&value));
        self.tick_positions.push(value);
    }

    #[wasm_bindgen(js_name = addTickLabel)]
    pub fn add_label(&mut self, label: &str) {
        self.tick_labels.push(label.into());
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
pub enum ColorSpace {
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

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InteractionMode {
    Disabled,
    RestrictedCompatibility,
    Compatibility,
    Restricted,
    Full,
}

pub enum Event {
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
    SetDataColorMode {
        color_mode: DataColorMode,
    },
    SetColorBarVisibility {
        visibility: bool,
    },
    AddLabel {
        id: String,
        color: Option<colors::ColorQuery<'static>>,
        selection_bounds: Option<(f32, f32)>,
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
    SetLabelSelectionBounds {
        id: String,
        selection_bounds: Option<(f32, f32)>,
    },
    SetLabelEasing {
        easing: selection::EasingType,
    },
    SetInteractionMode {
        mode: InteractionMode,
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

/// An event queue to interact with the renderer.
#[wasm_bindgen]
pub struct EventQueue {
    pub(crate) sender: Sender<Event>,
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
                let color = crate::DEFAULT_BACKGROUND_COLOR();
                colors::ColorQuery::SRgb(color.to_f32(), Some(color.alpha))
            }
            Element::Brush => {
                let color = crate::DEFAULT_BRUSH_COLOR();
                colors::ColorQuery::Xyz(color.to_f32(), None)
            }
            Element::Unselected => {
                let color = crate::DEFAULT_UNSELECTED_COLOR();
                colors::ColorQuery::Xyz(color.to_f32(), Some(color.alpha))
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
        let color = colors::ColorQuery::Named(color.to_string().into());
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
            ColorSpace::SRgb => colors::ColorQuery::SRgb(values, alpha),
            ColorSpace::Xyz => colors::ColorQuery::Xyz(values, alpha),
            ColorSpace::CieLab => colors::ColorQuery::Lab(values, alpha),
            ColorSpace::CieLch => colors::ColorQuery::Lch(values, alpha),
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
        let descriptor = crate::DEFAULT_COLOR_SCALE();

        self.sender
            .send_blocking(Event::SetColorScale {
                color_space: ColorSpace::SRgb,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setColorScaleNamed)]
    pub fn set_color_scale_named(&self, name: &str) {
        let descriptor = color_scale::ColorScaleDescriptor::Named(name.to_string().into());

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
            ColorSpace::SRgb => colors::ColorQuery::SRgb(values, alpha),
            ColorSpace::Xyz => colors::ColorQuery::Xyz(values, alpha),
            ColorSpace::CieLab => colors::ColorQuery::Lab(values, alpha),
            ColorSpace::CieLch => colors::ColorQuery::Lch(values, alpha),
        };
        let descriptor = color_scale::ColorScaleDescriptor::Constant(color);

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
                    ColorSpace::SRgb => colors::ColorQuery::SRgb(values, alpha),
                    ColorSpace::Xyz => colors::ColorQuery::Xyz(values, alpha),
                    ColorSpace::CieLab => colors::ColorQuery::Lab(values, alpha),
                    ColorSpace::CieLch => colors::ColorQuery::Lch(values, alpha),
                };

                (t, color)
            })
            .collect::<Vec<_>>();

        let descriptor = color_scale::ColorScaleDescriptor::Gradient(gradient);
        self.sender
            .send_blocking(Event::SetColorScale {
                color_space,
                scale: descriptor,
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setDefaultSelectedDataColorMode)]
    pub fn set_default_selected_data_color_mode(&self) {
        self.sender
            .send_blocking(Event::SetDataColorMode {
                color_mode: crate::DEFAULT_DATA_COLOR_MODE(),
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeConstant)]
    pub fn set_selected_data_color_mode_constant(&self, value: f32) {
        if !(0.0..=1.0).contains(&value) {
            panic!("constant must lie in the interval [0, 1], got '{value}'");
        }

        self.sender
            .send_blocking(Event::SetDataColorMode {
                color_mode: DataColorMode::Constant(value),
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeAttribute)]
    pub fn set_selected_data_color_mode_attribute(&self, id: &str) {
        self.sender
            .send_blocking(Event::SetDataColorMode {
                color_mode: DataColorMode::Attribute(id.into()),
            })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeProbability)]
    pub fn set_selected_data_color_mode_probability(&self) {
        self.sender
            .send_blocking(Event::SetDataColorMode {
                color_mode: DataColorMode::Probability,
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
        has_selection_bounds: bool,
        selection_bounds_start: f32,
        selection_bounds_end: f32,
    ) {
        let color = color.map(|color| {
            let ColorDescription {
                color_space,
                values,
                alpha,
            } = color;

            match color_space {
                ColorSpace::SRgb => colors::ColorQuery::SRgb(values, alpha),
                ColorSpace::Xyz => colors::ColorQuery::Xyz(values, alpha),
                ColorSpace::CieLab => colors::ColorQuery::Lab(values, alpha),
                ColorSpace::CieLch => colors::ColorQuery::Lch(values, alpha),
            }
        });
        let selection_bounds = if has_selection_bounds {
            Some((
                selection_bounds_start.clamp(f32::EPSILON, 1.0),
                selection_bounds_end.clamp(f32::EPSILON, 1.0),
            ))
        } else {
            None
        };

        self.sender
            .send_blocking(Event::AddLabel {
                id,
                color,
                selection_bounds,
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
                ColorSpace::SRgb => colors::ColorQuery::SRgb(values, alpha),
                ColorSpace::Xyz => colors::ColorQuery::Xyz(values, alpha),
                ColorSpace::CieLab => colors::ColorQuery::Lab(values, alpha),
                ColorSpace::CieLch => colors::ColorQuery::Lch(values, alpha),
            }
        });

        self.sender
            .send_blocking(Event::SetLabelColor { id, color })
            .expect("the channel should be open");
    }

    #[wasm_bindgen(js_name = setLabelSelectionBounds)]
    pub fn set_label_selection_bounds(
        &self,
        id: String,
        has_selection_bounds: bool,
        selection_bounds_start: f32,
        selection_bounds_end: f32,
    ) {
        let selection_bounds = if has_selection_bounds {
            Some((
                selection_bounds_start.clamp(f32::EPSILON, 1.0),
                selection_bounds_end.clamp(f32::EPSILON, 1.0),
            ))
        } else {
            None
        };
        self.sender
            .send_blocking(Event::SetLabelSelectionBounds {
                id,
                selection_bounds,
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

    #[wasm_bindgen(js_name = setInteractionMode)]
    pub fn set_interaction_mode(&self, mode: InteractionMode) {
        self.sender
            .send_blocking(Event::SetInteractionMode { mode })
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
