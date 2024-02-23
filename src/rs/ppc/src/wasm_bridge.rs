//! `Wasm` bridge types.
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use async_channel::Sender;
use wasm_bindgen::prelude::*;

use crate::{
    color_scale,
    colors::{self, Color},
    selection,
};

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PowerProfile {
    Auto,
    Low,
    High,
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct AxisDef {
    pub(crate) key: Box<str>,
    pub(crate) label: Box<str>,
    pub(crate) points: Box<[f32]>,
    pub(crate) range: Option<(f32, f32)>,
    pub(crate) visible_range: Option<(f32, f32)>,
    pub(crate) ticks: Option<Vec<(f32, Option<Rc<str>>)>>,
}

#[wasm_bindgen]
impl AxisDef {
    #[wasm_bindgen(constructor)]
    pub fn new(
        key: &str,
        label: &str,
        points: Box<[f32]>,
        range: Option<Box<[f32]>>,
        visible_range: Option<Box<[f32]>>,
        ticks: Option<AxisTicksDef>,
    ) -> Self {
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

        Self {
            key: key.into(),
            label: label.into(),
            points,
            range: range.map(|v| (v[0], v[1])),
            visible_range: visible_range.map(|v| (v[0], v[1])),
            ticks,
        }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisOrder {
    Automatic,
    Custom { order: Box<[String]> },
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ColorScale {
    pub color_space: ColorSpace,
    pub scale: color_scale::ColorScaleDescriptor<'static>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawOrder {
    Unordered,
    Increasing,
    Decreasing,
    SelectedUnordered,
    SelectedIncreasing,
    SelectedDecreasing,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DataColorMode {
    Constant(f32),
    Attribute(String),
    AttributeDensity(String),
    Probability,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Colors {
    pub background: Option<colors::ColorQuery<'static>>,
    pub brush: Option<colors::ColorQuery<'static>>,
    pub unselected: Option<colors::ColorQuery<'static>>,
    pub color_scale: Option<ColorScale>,
    pub draw_order: Option<DrawOrder>,
    pub color_mode: Option<DataColorMode>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Label {
    pub id: String,
    pub color: Option<colors::ColorQuery<'static>>,
    pub selection_bounds: Option<(f32, f32)>,
    pub easing: Option<selection::EasingType>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LabelColorUpdate {
    pub label: String,
    pub color: colors::ColorQuery<'static>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LabelBoundsUpdate {
    pub id: String,
    pub selection_bounds: (f32, f32),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LabelEasingUpdate {
    pub id: String,
    pub easing: selection::EasingType,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LabelVisibleAxesUpdate {
    pub id: String,
    pub visible_axes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Brush {
    pub control_points: Vec<(f32, f32)>,
    pub main_segment_idx: usize,
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

#[derive(Debug)]
enum StateTransactionOperation {
    AddAxis {
        axis: AxisDef,
    },
    RemoveAxis {
        axis: String,
    },
    SetAxisOrder {
        order: AxisOrder,
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
    SetDrawOrder {
        order: DrawOrder,
    },
    SetColorScale {
        color_scale: ColorScale,
    },
    SetDataColorMode {
        color_mode: DataColorMode,
    },
    SetColorBarVisibility {
        visibility: bool,
    },
    AddLabel {
        label: Label,
    },
    RemoveLabel {
        label: String,
    },
    SetLabelColor {
        update: LabelColorUpdate,
    },
    SetLabelSelectionBounds {
        update: LabelBoundsUpdate,
    },
    SetLabelEasing {
        update: LabelEasingUpdate,
    },
    SwitchActiveLabel {
        id: Option<String>,
    },
    SetBrushes {
        brushes: BTreeMap<String, BTreeMap<String, Vec<Brush>>>,
    },
    SetInteractionMode {
        mode: InteractionMode,
    },
    SetDebugOptions {
        options: DebugOptions,
    },
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct StateTransactionBuilder {
    operations: Vec<StateTransactionOperation>,
}

#[wasm_bindgen]
impl StateTransactionBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    #[wasm_bindgen(js_name = addAxis)]
    pub fn add_axis(&mut self, axis: AxisDef) {
        self.operations
            .push(StateTransactionOperation::AddAxis { axis });
    }

    #[wasm_bindgen(js_name = removeAxis)]
    pub fn remove_axis(&mut self, axis: String) {
        self.operations
            .push(StateTransactionOperation::RemoveAxis { axis });
    }

    #[wasm_bindgen(js_name = setAxisOrder)]
    pub fn set_axis_order(&mut self, order: js_sys::Array) {
        let order = if order.is_truthy() {
            let order = order.into_iter().map(|x| x.as_string().unwrap()).collect();
            AxisOrder::Custom { order }
        } else {
            AxisOrder::Automatic
        };

        self.operations
            .push(StateTransactionOperation::SetAxisOrder { order });
    }

    #[wasm_bindgen(js_name = setDefaultColor)]
    pub fn set_default_color(&mut self, element: Element) {
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
            Element::Background => StateTransactionOperation::SetBackgroundColor { color },
            Element::Brush => StateTransactionOperation::SetBrushColor { color },
            Element::Unselected => StateTransactionOperation::SetUnselectedColor { color },
        };

        self.operations.push(event);
    }

    #[wasm_bindgen(js_name = setDefaultDrawOrder)]
    pub fn set_default_draw_order(&mut self) {
        self.set_draw_order(crate::DEFAULT_DRAW_ORDER);
    }

    #[wasm_bindgen(js_name = setColorNamed)]
    pub fn set_color_named(&mut self, element: Element, color: &str) {
        let color = colors::ColorQuery::Named(color.to_string().into());
        let event = match element {
            Element::Background => StateTransactionOperation::SetBackgroundColor { color },
            Element::Brush => StateTransactionOperation::SetBrushColor { color },
            Element::Unselected => StateTransactionOperation::SetUnselectedColor { color },
        };

        self.operations.push(event);
    }

    #[wasm_bindgen(js_name = setColorValue)]
    pub fn set_color_value(&mut self, element: Element, color: ColorDescription) {
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
            Element::Background => StateTransactionOperation::SetBackgroundColor { color },
            Element::Brush => StateTransactionOperation::SetBrushColor { color },
            Element::Unselected => StateTransactionOperation::SetUnselectedColor { color },
        };

        self.operations.push(event);
    }

    #[wasm_bindgen(js_name = setDrawOrder)]
    pub fn set_draw_order(&mut self, order: DrawOrder) {
        self.operations
            .push(StateTransactionOperation::SetDrawOrder { order });
    }

    #[wasm_bindgen(js_name = setDefaultColorScaleColor)]
    pub fn set_default_color_scale_color(&mut self) {
        let scale = crate::DEFAULT_COLOR_SCALE();

        let color_scale = ColorScale {
            color_space: ColorSpace::SRgb,
            scale,
        };
        self.operations
            .push(StateTransactionOperation::SetColorScale { color_scale });
    }

    #[wasm_bindgen(js_name = setColorScaleNamed)]
    pub fn set_color_scale_named(&mut self, name: &str) {
        let scale = color_scale::ColorScaleDescriptor::Named(name.to_string().into());

        let color_scale = ColorScale {
            color_space: ColorSpace::Xyz,
            scale,
        };
        self.operations
            .push(StateTransactionOperation::SetColorScale { color_scale });
    }

    #[wasm_bindgen(js_name = setColorScaleConstant)]
    pub fn set_color_scale_constant(&mut self, color: ColorDescription) {
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
        let scale = color_scale::ColorScaleDescriptor::Constant(color);

        let color_scale = ColorScale {
            color_space: ColorSpace::SRgb,
            scale,
        };
        self.operations
            .push(StateTransactionOperation::SetColorScale { color_scale });
    }

    #[wasm_bindgen(js_name = setColorScaleGradient)]
    pub fn set_color_scale_gradient(&mut self, scale: ColorScaleDescription) {
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

        let scale = color_scale::ColorScaleDescriptor::Gradient(gradient);
        let color_scale = ColorScale { color_space, scale };
        self.operations
            .push(StateTransactionOperation::SetColorScale { color_scale });
    }

    #[wasm_bindgen(js_name = setDefaultSelectedDataColorMode)]
    pub fn set_default_selected_data_color_mode(&mut self) {
        self.operations
            .push(StateTransactionOperation::SetDataColorMode {
                color_mode: crate::DEFAULT_DATA_COLOR_MODE(),
            });
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeConstant)]
    pub fn set_selected_data_color_mode_constant(&mut self, value: f32) {
        if !(0.0..=1.0).contains(&value) {
            panic!("constant must lie in the interval [0, 1], got '{value}'");
        }

        self.operations
            .push(StateTransactionOperation::SetDataColorMode {
                color_mode: DataColorMode::Constant(value),
            });
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeAttribute)]
    pub fn set_selected_data_color_mode_attribute(&mut self, id: &str) {
        self.operations
            .push(StateTransactionOperation::SetDataColorMode {
                color_mode: DataColorMode::Attribute(id.into()),
            });
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeAttributeDensity)]
    pub fn set_selected_data_color_mode_attribute_density(&mut self, id: &str) {
        self.operations
            .push(StateTransactionOperation::SetDataColorMode {
                color_mode: DataColorMode::AttributeDensity(id.into()),
            });
    }

    #[wasm_bindgen(js_name = setSelectedDataColorModeProbability)]
    pub fn set_selected_data_color_mode_probability(&mut self) {
        self.operations
            .push(StateTransactionOperation::SetDataColorMode {
                color_mode: DataColorMode::Probability,
            });
    }

    #[wasm_bindgen(js_name = setColorBarVisibility)]
    pub fn set_color_bar_visibility(&mut self, visibility: bool) {
        self.operations
            .push(StateTransactionOperation::SetColorBarVisibility { visibility });
    }

    #[wasm_bindgen(js_name = addLabel)]
    pub fn add_label(
        &mut self,
        id: String,
        color: Option<ColorDescription>,
        has_selection_bounds: bool,
        selection_bounds_start: f32,
        selection_bounds_end: f32,
        easing_type: Option<String>,
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

        let label = Label {
            id,
            color,
            selection_bounds,
            easing: Some(easing),
        };
        self.operations
            .push(StateTransactionOperation::AddLabel { label });
    }

    #[wasm_bindgen(js_name = removeLabel)]
    pub fn remove_label(&mut self, label: String) {
        self.operations
            .push(StateTransactionOperation::RemoveLabel { label });
    }

    #[wasm_bindgen(js_name = setLabelColor)]
    pub fn set_label_color(&mut self, label: String, color: ColorDescription) {
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

        let update = LabelColorUpdate { label, color };
        self.operations
            .push(StateTransactionOperation::SetLabelColor { update });
    }

    #[wasm_bindgen(js_name = setLabelSelectionBounds)]
    pub fn set_label_selection_bounds(
        &mut self,
        id: String,
        selection_bounds_start: f32,
        selection_bounds_end: f32,
    ) {
        let selection_bounds = (
            selection_bounds_start.clamp(f32::EPSILON, 1.0),
            selection_bounds_end.clamp(f32::EPSILON, 1.0),
        );

        let update = LabelBoundsUpdate {
            id,
            selection_bounds,
        };
        self.operations
            .push(StateTransactionOperation::SetLabelSelectionBounds { update });
    }

    #[wasm_bindgen(js_name = setLabelEasing)]
    pub fn set_label_easing(&mut self, id: String, easing_type: Option<String>) {
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

        let update = LabelEasingUpdate { id, easing };
        self.operations
            .push(StateTransactionOperation::SetLabelEasing { update });
    }

    #[wasm_bindgen(js_name = switchActiveLabel)]
    pub fn switch_active_label(&mut self, id: Option<String>) {
        self.operations
            .push(StateTransactionOperation::SwitchActiveLabel { id });
    }

    #[wasm_bindgen(js_name = setBrushes)]
    pub fn set_brushes(&mut self, brushes: &js_sys::Object) {
        let mut brush_map = BTreeMap::default();
        if !brushes.is_falsy() {
            let entries = js_sys::Object::entries(brushes);
            for entry in entries {
                let entry = entry.unchecked_into::<js_sys::Array>();
                let label = entry.get(0).as_string().unwrap();
                let label_brushes = entry.get(1).unchecked_into::<js_sys::Object>();

                let mut label_map = BTreeMap::default();
                let entries = js_sys::Object::entries(&label_brushes);
                for entry in entries {
                    let entry = entry.unchecked_into::<js_sys::Array>();
                    let axis = entry.get(0).as_string().unwrap();
                    let brushes = entry.get(1).unchecked_into::<js_sys::Array>();

                    let mut brushes_vec = Vec::new();
                    for brush in brushes {
                        let control_points = js_sys::Reflect::get(&brush, &"controlPoints".into())
                            .unwrap()
                            .unchecked_into::<js_sys::Array>();
                        let main_segment_idx =
                            js_sys::Reflect::get(&brush, &"mainSegmentIdx".into())
                                .unwrap()
                                .unchecked_into::<js_sys::Number>();

                        let control_points = control_points
                            .into_iter()
                            .map(|point| {
                                let point = point.unchecked_into::<js_sys::Array>();
                                let x = point.get(0).unchecked_into::<js_sys::Number>().value_of()
                                    as f32;
                                let y = point.get(1).unchecked_into::<js_sys::Number>().value_of()
                                    as f32;
                                (x, y)
                            })
                            .collect::<Vec<_>>();
                        let main_segment_idx = main_segment_idx.value_of() as usize;

                        if !control_points.is_empty() {
                            let brush = Brush {
                                control_points,
                                main_segment_idx,
                            };
                            brushes_vec.push(brush);
                        }
                    }

                    if !brushes_vec.is_empty() {
                        label_map.insert(axis, brushes_vec);
                    }
                }

                if !label_map.is_empty() {
                    brush_map.insert(label, label_map);
                }
            }
        }

        self.operations
            .push(StateTransactionOperation::SetBrushes { brushes: brush_map });
    }

    #[wasm_bindgen(js_name = setInteractionMode)]
    pub fn set_interaction_mode(&mut self, mode: InteractionMode) {
        self.operations
            .push(StateTransactionOperation::SetInteractionMode { mode });
    }

    #[wasm_bindgen(js_name = setDebugOptions)]
    pub fn set_debug_options(&mut self, options: DebugOptions) {
        self.operations
            .push(StateTransactionOperation::SetDebugOptions { options })
    }

    pub fn build(self) -> StateTransaction {
        let mut axis_removals: BTreeSet<String> = Default::default();
        let mut axis_additions: BTreeMap<String, AxisDef> = Default::default();
        let mut order_change: Option<AxisOrder> = Default::default();
        let mut colors_change: Option<Colors> = Default::default();
        let mut color_bar_visibility_change: Option<bool> = Default::default();
        let mut label_removals: BTreeSet<String> = Default::default();
        let mut label_additions: BTreeMap<String, Label> = Default::default();
        let mut label_updates: BTreeMap<String, Label> = Default::default();
        let mut active_label_change: Option<Option<String>> = Default::default();
        let mut brushes_change: Option<BTreeMap<String, BTreeMap<String, Vec<Brush>>>> =
            Default::default();
        let mut interaction_mode_change: Option<InteractionMode> = Default::default();
        let mut debug_options_change: Option<DebugOptions> = Default::default();

        for op in self.operations {
            match op {
                StateTransactionOperation::RemoveAxis { axis } => {
                    let _ = axis_removals.insert(axis);
                }
                StateTransactionOperation::AddAxis { axis } => {
                    axis_additions.insert(axis.key.clone().into(), axis);
                }
                StateTransactionOperation::SetAxisOrder { order } => {
                    order_change = Some(order);
                }
                StateTransactionOperation::SetBackgroundColor { color } => {
                    let c = colors_change.get_or_insert(Colors {
                        background: None,
                        brush: None,
                        unselected: None,
                        draw_order: None,
                        color_scale: None,
                        color_mode: None,
                    });
                    c.background = Some(color);
                }
                StateTransactionOperation::SetBrushColor { color } => {
                    let c = colors_change.get_or_insert(Colors {
                        background: None,
                        brush: None,
                        unselected: None,
                        draw_order: None,
                        color_scale: None,
                        color_mode: None,
                    });
                    c.brush = Some(color);
                }
                StateTransactionOperation::SetUnselectedColor { color } => {
                    let c = colors_change.get_or_insert(Colors {
                        background: None,
                        brush: None,
                        unselected: None,
                        draw_order: None,
                        color_scale: None,
                        color_mode: None,
                    });
                    c.unselected = Some(color);
                }
                StateTransactionOperation::SetDrawOrder { order } => {
                    let c = colors_change.get_or_insert(Colors {
                        background: None,
                        brush: None,
                        unselected: None,
                        draw_order: None,
                        color_scale: None,
                        color_mode: None,
                    });
                    c.draw_order = Some(order);
                }
                StateTransactionOperation::SetColorScale { color_scale } => {
                    let c = colors_change.get_or_insert(Colors {
                        background: None,
                        brush: None,
                        unselected: None,
                        draw_order: None,
                        color_scale: None,
                        color_mode: None,
                    });
                    c.color_scale = Some(color_scale);
                }
                StateTransactionOperation::SetDataColorMode { color_mode } => {
                    let c = colors_change.get_or_insert(Colors {
                        background: None,
                        brush: None,
                        unselected: None,
                        draw_order: None,
                        color_scale: None,
                        color_mode: None,
                    });
                    c.color_mode = Some(color_mode);
                }
                StateTransactionOperation::SetColorBarVisibility { visibility } => {
                    color_bar_visibility_change = Some(visibility);
                }
                StateTransactionOperation::AddLabel { label } => {
                    label_additions.insert(label.id.clone(), label);
                }
                StateTransactionOperation::RemoveLabel { label } => {
                    let _ = label_removals.insert(label);
                }
                StateTransactionOperation::SetLabelColor { update } => {
                    let label = label_updates.entry(update.label.clone()).or_insert(Label {
                        id: update.label,
                        color: None,
                        selection_bounds: None,
                        easing: None,
                    });
                    label.color = Some(update.color)
                }
                StateTransactionOperation::SetLabelSelectionBounds { update } => {
                    let label = label_updates.entry(update.id.clone()).or_insert(Label {
                        id: update.id,
                        color: None,
                        selection_bounds: None,
                        easing: None,
                    });
                    label.selection_bounds = Some(update.selection_bounds);
                }
                StateTransactionOperation::SetLabelEasing { update } => {
                    let label = label_updates.entry(update.id.clone()).or_insert(Label {
                        id: update.id,
                        color: None,
                        selection_bounds: None,
                        easing: None,
                    });
                    label.easing = Some(update.easing);
                }
                StateTransactionOperation::SwitchActiveLabel { id } => {
                    active_label_change = Some(id);
                }
                StateTransactionOperation::SetBrushes { brushes } => {
                    brushes_change = Some(brushes);
                }
                StateTransactionOperation::SetInteractionMode { mode } => {
                    interaction_mode_change = Some(mode);
                }
                StateTransactionOperation::SetDebugOptions { options } => {
                    debug_options_change = Some(options);
                }
            }
        }

        StateTransaction {
            axis_removals,
            axis_additions,
            order_change,
            colors_change,
            color_bar_visibility_change,
            label_removals,
            label_additions,
            label_updates,
            active_label_change,
            brushes_change,
            interaction_mode_change,
            debug_options_change,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct StateTransaction {
    pub(crate) axis_removals: BTreeSet<String>,
    pub(crate) axis_additions: BTreeMap<String, AxisDef>,
    pub(crate) order_change: Option<AxisOrder>,
    pub(crate) colors_change: Option<Colors>,
    pub(crate) color_bar_visibility_change: Option<bool>,
    pub(crate) label_removals: BTreeSet<String>,
    pub(crate) label_additions: BTreeMap<String, Label>,
    pub(crate) label_updates: BTreeMap<String, Label>,
    pub(crate) active_label_change: Option<Option<String>>,
    pub(crate) brushes_change: Option<BTreeMap<String, BTreeMap<String, Vec<Brush>>>>,
    pub(crate) interaction_mode_change: Option<InteractionMode>,
    pub(crate) debug_options_change: Option<DebugOptions>,
}

#[wasm_bindgen]
impl StateTransaction {
    pub fn log(&self) {
        web_sys::console::log_1(&format!("{self:?}").into());
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.axis_removals.is_empty()
            && self.axis_additions.is_empty()
            && self.order_change.is_none()
            && self.colors_change.is_none()
            && self.color_bar_visibility_change.is_none()
            && self.label_removals.is_empty()
            && self.label_additions.is_empty()
            && self.label_updates.is_empty()
            && self.active_label_change.is_none()
            && self.interaction_mode_change.is_none()
            && self.debug_options_change.is_none()
    }
}

pub enum Event {
    Exit,
    Resize {
        width: u32,
        height: u32,
        device_pixel_ratio: f32,
    },
    CommitTransaction {
        transaction: StateTransaction,
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

    /// Commits a new state transaction.
    #[wasm_bindgen(js_name = commitTransaction)]
    pub fn commit_transaction(&self, transaction: StateTransaction) {
        if transaction.is_empty() {
            return;
        }
        self.sender
            .send_blocking(Event::CommitTransaction { transaction })
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
