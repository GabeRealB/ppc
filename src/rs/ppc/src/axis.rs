use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    collections::BTreeMap,
    fmt::Debug,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use wasm_bindgen::JsCast;

use crate::{
    coordinates::{
        Aabb, CoordinateSystem, CoordinateSystemTransformer, Length, LocalSpace, Offset, Position,
        ScreenSpace, ScreenViewTransformer, ViewSpace, ViewWorldTransformer, WorldLocalTransformer,
        WorldSpace,
    },
    lerp::{InverseLerp, Lerp},
    selection::{SelectionCurve, SelectionCurveBuilder},
};

const AXIS_LOCAL_Y_SCALE: f32 = 1.0;
const AXIS_LINE_SIZE_REM: f32 = 0.05;
const AXIS_LINE_PADDING_REM: f32 = 0.1;
const AXIS_TOP_PADDING: f32 = 1.0;
const LOCAL_AXIS_HEIGHT: f32 = 1.0;

const SELECTION_LINE_SIZE_REM: f32 = 0.1;
const SELECTION_LINE_PADDING_REM: f32 = 0.15;
const SELECTION_LINE_MARGIN_REM: f32 = 1.0;

const CURVE_LINE_SIZE_REM: f32 = 0.075;
const DATA_LINE_SIZE_REM: f32 = 0.1;
const CONTROL_POINTS_RADIUS_REM: f32 = 0.3;

const LABEL_PADDING_REM: f32 = 1.0;
const LABEL_MARGIN_REM: f32 = 1.0;

const TICKS_PADDING_REM: f32 = 0.5;

const MIN_CURVE_T: f32 = 0.1;
const MAX_CURVE_T: f32 = 0.95;

#[derive(Debug)]
pub struct AxisArgs {
    label: Rc<str>,
    data: Box<[f32]>,
    range: (f32, f32),
    min_range: (f32, f32),
    visible_range: Option<(f32, f32)>,
    ticks: Option<Vec<(f32, Option<Rc<str>>)>>,
    state: AxisState,
}

impl AxisArgs {
    /// Constructs a new instance with default settings.
    pub fn new(label: &str, data: Box<[f32]>) -> Self {
        let mut data: Vec<_> = data.into();
        data.retain(|x| !x.is_nan());

        let min = data
            .iter()
            .cloned()
            .min_by(|x, y| x.partial_cmp(y).unwrap());
        let max = data
            .iter()
            .cloned()
            .max_by(|x, y| x.partial_cmp(y).unwrap());

        let mut range = min.zip(max).unwrap_or((0.0, 1.0));
        if range.0 == range.1 {
            range.0 -= 0.5;
            range.1 += 0.5;
        }

        let min_range = range;

        Self {
            label: label.into(),
            data: data.into(),
            range,
            min_range,
            visible_range: None,
            ticks: None,
            state: AxisState::Collapsed,
        }
    }

    /// Sets the range of the axis.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        assert!(
            min < max,
            "min must be smaller than max, min = {min}, max = {max}"
        );
        assert!(min.is_finite(), "the minimum must be finite");
        assert!(max.is_finite(), "the maximum must be finite");
        assert!(
            min <= self.min_range.0 && max >= self.min_range.1,
            "the range must be bigger or equal to the min/max of the data, min = {min}, max = {max}, range = {:?}", 
            self.min_range
        );

        self.range = (min, max);
        let (ticks_min, ticks_max) = if let Some(visible_range) = &mut self.visible_range {
            visible_range.0 = visible_range.0.clamp(self.range.0, self.range.1);
            visible_range.1 = visible_range.1.clamp(self.range.0, self.range.1);
            *visible_range
        } else {
            self.range
        };

        if let Some(ticks) = &mut self.ticks {
            ticks.retain(|(x, _)| (ticks_min..=ticks_max).contains(x))
        }

        self
    }

    /// Sets the visible range of the axis.
    pub fn with_visible_range(mut self, min: f32, max: f32) -> Self {
        assert!(
            min < max,
            "min must be smaller than max, min = {min}, max = {max}"
        );
        assert!(min.is_finite(), "the minimum must be finite");
        assert!(max.is_finite(), "the maximum must be finite");
        assert!(
            min >= self.range.0 && max <= self.range.1,
            "the range must be smaller or equal to the data range, min = {min}, max = {max}, range = {:?}",
            self.range
        );

        if let Some(ticks) = &mut self.ticks {
            ticks.retain(|(x, _)| (min..=max).contains(x))
        }

        self.visible_range = Some((min, max));
        self
    }

    pub fn with_ticks(mut self, mut ticks: Vec<(f32, Option<Rc<str>>)>) -> Self {
        let (min, max) = self.visible_range.unwrap_or(self.range);
        ticks.retain(|(x, _)| (min..=max).contains(x));
        self.ticks = Some(ticks);

        self
    }
}

/// A PPC axis.
#[allow(clippy::type_complexity)]
pub struct Axis {
    key: Rc<str>,

    label: Rc<str>,
    min_label: Rc<str>,
    max_label: Rc<str>,

    state: Cell<AxisState>,
    axis_index: Cell<Option<usize>>,

    data: Box<[f32]>,
    data_density: Box<[f32]>,
    data_normalized: Box<[f32]>,

    data_range: (f32, f32),
    visible_data_range: (f32, f32),
    visible_data_range_normalized: (f32, f32),

    ticks: Vec<(f32, Rc<str>)>,
    max_tick_height: Length<LocalSpace>,

    selection_curves: RefCell<Vec<SelectionCurve>>,
    curve_builders: RefCell<Vec<SelectionCurveBuilder>>,

    world_offset: Cell<f32>,

    get_rem_length: Rc<dyn Fn(f32) -> (Length<LocalSpace>, Length<LocalSpace>)>,
    get_text_length: Rc<dyn Fn(&str) -> (Length<LocalSpace>, Length<LocalSpace>)>,

    axes: Weak<RefCell<Axes>>,
    left: RefCell<Option<Rc<Self>>>,
    right: RefCell<Option<Rc<Self>>>,
}

impl Axis {
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn new(
        key: &str,
        args: AxisArgs,
        axis_index: Option<usize>,
        world_offset: f32,
        num_labels: usize,
        axes: &Rc<RefCell<Axes>>,
        get_rem_length: Rc<dyn Fn(f32) -> (Length<LocalSpace>, Length<LocalSpace>)>,
        get_text_length: Rc<dyn Fn(&str) -> (Length<LocalSpace>, Length<LocalSpace>)>,
    ) -> Self {
        let label = args.label;
        let data = args.data;
        let data_range = args.range;
        let visible_data_range = args.visible_range.unwrap_or(data_range);
        let ticks = args.ticks;
        let state = args.state;

        let data_normalized = data
            .iter()
            .map(|d| d.inv_lerp(data_range.0, data_range.1))
            .collect::<Box<[_]>>();

        // Compute the density of each point by counting the number
        // of points contained within a window. Note: This could be
        // optimized to a complexity of O(N log N) by sorting the data
        // first.
        let data_density = data_normalized
            .iter()
            .map(|&d| {
                const WINDOW_SIZE: f32 = 0.05;
                let window = d - WINDOW_SIZE..=d + WINDOW_SIZE;
                let count = data_normalized
                    .iter()
                    .filter(|&x| window.contains(x))
                    .count() as f64;
                let density = count / data_normalized.len() as f64;
                density as f32
            })
            .collect::<Box<[_]>>();

        let visible_data_range_normalized = (
            visible_data_range.0.inv_lerp(data_range.0, data_range.1),
            visible_data_range.1.inv_lerp(data_range.0, data_range.1),
        );

        let locales = wasm_bindgen::JsValue::undefined().unchecked_into();
        let options = wasm_bindgen::JsValue::undefined().unchecked_into();
        let formatter = js_sys::Intl::NumberFormat::new(&locales, &options);
        let format = formatter.format();

        let min_num = wasm_bindgen::JsValue::from_f64(visible_data_range.0 as f64);
        let max_num = wasm_bindgen::JsValue::from_f64(visible_data_range.1 as f64);
        let min_label = format.call1(&formatter, &min_num).unwrap();
        let max_label = format.call1(&formatter, &max_num).unwrap();

        let min_label = min_label.as_string().unwrap().into();
        let max_label = max_label.as_string().unwrap().into();
        let axes = Rc::downgrade(axes);

        let ticks = if let Some(ticks) = ticks {
            ticks
                .into_iter()
                .map(|(t, label)| {
                    let label = label.unwrap_or_else(|| {
                        let label_v = wasm_bindgen::JsValue::from_f64(t as f64);
                        let label = format.call1(&formatter, &label_v).unwrap();
                        label.as_string().unwrap().into()
                    });

                    (
                        t.inv_lerp(visible_data_range.0, visible_data_range.1),
                        label,
                    )
                })
                .collect::<Vec<_>>()
        } else {
            [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                .into_iter()
                .filter(|t| {
                    (visible_data_range_normalized.0..=visible_data_range_normalized.1).contains(t)
                })
                .map(|t| {
                    let label_v = data_range.0.lerp(data_range.1, t);
                    let label_v = wasm_bindgen::JsValue::from_f64(label_v as f64);
                    let label = format.call1(&formatter, &label_v).unwrap();
                    let label = label.as_string().unwrap().into();
                    (t, label)
                })
                .collect::<Vec<_>>()
        };
        let max_tick_height = ticks
            .iter()
            .map(|(_, tick)| get_text_length(tick).1)
            .max_by(|&l, &r| l.0.total_cmp(&r.0))
            .unwrap_or(Length::new(0.0));

        let selection_curves = (0..num_labels)
            .map(|_| SelectionCurve::new(visible_data_range_normalized.into()))
            .collect();
        let curve_builders = (0..num_labels)
            .map(|_| SelectionCurveBuilder::new())
            .collect();

        Self {
            key: key.into(),
            label,
            min_label,
            max_label,
            state: Cell::new(state),
            axis_index: Cell::new(axis_index),
            data,
            data_density,
            data_normalized,
            data_range,
            visible_data_range,
            visible_data_range_normalized,
            ticks,
            max_tick_height,
            selection_curves: RefCell::new(selection_curves),
            curve_builders: RefCell::new(curve_builders),
            world_offset: Cell::new(world_offset),
            get_rem_length,
            get_text_length,
            axes,
            left: RefCell::new(None),
            right: RefCell::new(None),
        }
    }

    /// Fetches the key of the axis.
    pub fn key(&self) -> Rc<str> {
        self.key.clone()
    }

    /// Fetches the label of the axis.
    pub fn label(&self) -> Rc<str> {
        self.label.clone()
    }

    /// Fetches the label of the minimum element.
    pub fn min_label(&self) -> Rc<str> {
        self.min_label.clone()
    }

    /// Fetches the label of the maximum element.
    pub fn max_label(&self) -> Rc<str> {
        self.max_label.clone()
    }

    /// Fetches the ticks and their positions.
    pub fn ticks(&self) -> &[(f32, Rc<str>)] {
        &self.ticks
    }

    /// Fetches the state of the axis.
    pub fn state(&self) -> AxisState {
        self.state.get()
    }

    /// Checks whether the axis is in a collapsed state.
    pub fn is_collapsed(&self) -> bool {
        matches!(self.state.get(), AxisState::Collapsed)
    }

    /// Checks whether the axis is in an expanded state.
    pub fn is_expanded(&self) -> bool {
        matches!(self.state.get(), AxisState::Expanded)
    }

    /// Checks whether the axis is in a hidden state.
    pub fn is_hidden(&self) -> bool {
        self.axis_index.get().is_none()
    }

    /// Collapses the axis.
    ///
    /// # Panics
    ///
    /// Panics if the axis is not expanded.
    pub fn collapse(&self) {
        assert!(self.is_expanded());
        self.state.set(AxisState::Collapsed);
    }

    /// Expands the axis.
    ///
    /// # Panics
    ///
    /// Panics if the axis is not collapsed.
    pub fn expand(&self) {
        assert!(self.is_collapsed());
        self.state.set(AxisState::Expanded);
    }

    /// Fetches the index of the axis.
    pub fn axis_index(&self) -> Option<usize> {
        self.axis_index.get()
    }

    /// Fetches the data of the axis.
    #[allow(dead_code)]
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Fetches the density of the data.
    pub fn data_density(&self) -> &[f32] {
        &self.data_density
    }

    /// Fetches the normalized data of the axis.
    pub fn data_normalized(&self) -> &[f32] {
        &self.data_normalized
    }

    /// Returns the `min` and `max` value of the data.
    #[allow(dead_code)]
    pub fn data_range(&self) -> (f32, f32) {
        self.data_range
    }

    /// Returns the `min` and `max` value of the visible data.
    #[allow(dead_code)]
    pub fn visible_data_range(&self) -> (f32, f32) {
        self.visible_data_range
    }

    /// Returns the `min` and `max` value of the visible data, normalized in
    /// relation the the `min` and `max` of all data.
    pub fn visible_data_range_normalized(&self) -> (f32, f32) {
        self.visible_data_range_normalized
    }

    /// Borrows the selection curve.
    #[allow(dead_code)]
    pub fn borrow_selection_curve(&self, active_label_idx: usize) -> Ref<'_, SelectionCurve> {
        Ref::map(self.selection_curves.borrow(), |x| &x[active_label_idx])
    }

    /// Borrows the selection curve mutably.
    pub fn borrow_selection_curve_mut(
        &self,
        active_label_idx: usize,
    ) -> RefMut<'_, SelectionCurve> {
        RefMut::map(self.selection_curves.borrow_mut(), |x| {
            &mut x[active_label_idx]
        })
    }

    /// Borrows the curve builder.
    pub fn borrow_selection_curve_builder(
        &self,
        active_label_idx: usize,
    ) -> Ref<'_, SelectionCurveBuilder> {
        Ref::map(self.curve_builders.borrow(), |x| &x[active_label_idx])
    }

    /// Borrows the curve builder mutably.
    pub fn borrow_selection_curve_builder_mut(
        &self,
        active_label_idx: usize,
    ) -> RefMut<'_, SelectionCurveBuilder> {
        RefMut::map(self.curve_builders.borrow_mut(), |x| {
            &mut x[active_label_idx]
        })
    }

    /// Signals that the axis must allocate another selection curve and selection curve builder for the new label.
    pub fn push_label(&self) {
        self.selection_curves.borrow_mut().push(SelectionCurve::new(
            self.visible_data_range_normalized.into(),
        ));
        self.curve_builders
            .borrow_mut()
            .push(SelectionCurveBuilder::new());
    }

    /// Removes the selection curve and selection curve builder assigned to a label.
    pub fn remove_label(&self, label_idx: usize) {
        self.selection_curves.borrow_mut().remove(label_idx);
        self.curve_builders.borrow_mut().remove(label_idx);
    }

    /// Returns the bounding box of the axis.
    pub fn bounding_box(&self, active_label_idx: Option<usize>) -> Aabb<LocalSpace> {
        let label_bb = self.label_bounding_box();
        let axis_line_bb = self.axis_line_bounding_box();
        let curves_bb = self.curves_bounding_box();

        let mut min_x = label_bb
            .start()
            .x
            .min(axis_line_bb.start().x)
            .min(curves_bb.start().x);
        let mut max_x = label_bb
            .end()
            .x
            .max(axis_line_bb.end().x)
            .max(curves_bb.end().x);

        if let Some(active_label_idx) = active_label_idx {
            let selections_bb = self.selections_bounding_box(active_label_idx);
            max_x = max_x.max(selections_bb.end().x);
        }

        min_x = min_x.clamp(-0.4, 0.4);
        max_x = max_x.clamp(-0.4, 0.4);

        let start = Position::<LocalSpace>::new((min_x, 0.0));
        let end = Position::<LocalSpace>::new((max_x, 1.0));

        Aabb::new(start, end)
    }

    /// Returns the bounding box of the axis line.
    pub fn axis_line_bounding_box(&self) -> Aabb<LocalSpace> {
        let (mut start, mut end) = self.axis_line_range();

        let (axis_width, _) = (self.get_rem_length)(
            AXIS_LINE_PADDING_REM + AXIS_LINE_PADDING_REM + AXIS_LINE_SIZE_REM,
        );
        let half_width = axis_width / Length::new(2.0);
        let width_offset = Offset::<LocalSpace>::new((1.0, 0.0)) * half_width;

        start -= width_offset;
        end += width_offset;

        Aabb::new(start, end)
    }

    pub fn curves_bounding_box(&self) -> Aabb<LocalSpace> {
        let start = if self.is_expanded() {
            Position::new((-0.4, 0.0))
        } else {
            Position::new((0.0, 1.0))
        };
        let end = Position::new((0.0, 1.0));
        Aabb::new(start, end)
    }

    pub fn selections_bounding_box(&self, active_label_idx: usize) -> Aabb<LocalSpace> {
        let curve_builders = self.curve_builders.borrow();
        let max_rank = if !self.is_expanded() {
            0
        } else {
            curve_builders[active_label_idx].max_rank()
        };

        let (control_point_radius_w, _) = self.axes().borrow().control_points_radius_local();

        let start_x = -control_point_radius_w.0;
        let end_x = self.selection_offset_at_rank(max_rank).x + control_point_radius_w.0;

        let start = Position::new((start_x, 0.0));
        let end = Position::new((end_x, 1.0));
        Aabb::new(start, end)
    }

    /// Returns the bounding box of the axis label.
    pub fn label_bounding_box(&self) -> Aabb<LocalSpace> {
        const POSITION_X: f32 = 0.0;

        let (label_width, label_height) = (self.get_text_length)(&self.label);
        let (_, top_padding) = (self.get_rem_length)(AXIS_TOP_PADDING);
        let (padding_width, padding_height) = (self.get_rem_length)(AXIS_LINE_PADDING_REM);

        let start = Position::new((
            POSITION_X - padding_width.0 - (label_width.0 / 2.0),
            LOCAL_AXIS_HEIGHT
                - top_padding.0
                - padding_height.0
                - label_height.0
                - padding_height.0,
        ));
        let end = Position::new((
            POSITION_X + padding_width.0 + (label_width.0 / 2.0),
            LOCAL_AXIS_HEIGHT - top_padding.0,
        ));

        Aabb::new(start, end)
    }

    pub fn curve_offset_at_curve_value(&self, curve_value: f32) -> Offset<LocalSpace> {
        let t = MIN_CURVE_T.lerp(MAX_CURVE_T, curve_value);
        let x_offset = 0.0.lerp(-0.4, t);
        Offset::new((x_offset, 0.0))
    }

    pub fn selection_offset_at_rank(&self, rank: usize) -> Offset<LocalSpace> {
        let (width, _) = (self.get_rem_length)(SELECTION_LINE_SIZE_REM);
        let (padding, _) = (self.get_rem_length)(SELECTION_LINE_PADDING_REM);
        let (margin, _) = (self.get_rem_length)(SELECTION_LINE_MARGIN_REM);

        let x_offset = (rank as f32) * (width + padding + padding + margin).0;
        Offset::new((x_offset, 0.0))
    }

    pub fn selection_rank_at_position(
        &self,
        position: &Position<LocalSpace>,
        active_label_idx: usize,
    ) -> Option<usize> {
        let curve_builders = self.curve_builders.borrow();
        let max_rank = curve_builders[active_label_idx].max_rank();
        let (control_point_radius_w, _) = self.axes().borrow().control_points_radius_local();

        for i in 0..=max_rank {
            let rank_middle = self.selection_offset_at_rank(i).x;
            let rank_start = rank_middle - control_point_radius_w.0;
            let rank_end = rank_middle + control_point_radius_w.0;

            if (rank_start..=rank_end).contains(&position.x) {
                return Some(i);
            }

            if position.x < rank_start {
                return None;
            }
        }

        None
    }

    /// Returns the extends of the expanded axis lines.
    pub fn expanded_extends(&self, active_label_idx: Option<usize>) -> Aabb<LocalSpace> {
        let curve_builders = self.curve_builders.borrow();
        let max_rank = active_label_idx
            .map(|active_label_idx| curve_builders[active_label_idx].max_rank())
            .unwrap_or(0);

        let end_x = self.selection_offset_at_rank(max_rank).x;

        let start = Position::new((-0.4, 0.0));
        let end = Position::new((end_x, 1.0));
        Aabb::new(start, end)
    }

    /// Returns the range of the axis line.
    pub fn axis_line_range(&self) -> (Position<LocalSpace>, Position<LocalSpace>) {
        const POSITION_X: f32 = 0.0;
        let (_, top_padding) = (self.get_rem_length)(AXIS_TOP_PADDING);
        let (_, label_padding) = (self.get_rem_length)(LABEL_PADDING_REM);
        let (_, label_margin) = (self.get_rem_length)(LABEL_MARGIN_REM);

        let (_, min_label_height) = (self.get_text_length)(&self.min_label);
        let (_, max_label_height) = (self.get_text_length)(&self.max_label);
        let (_, label_height) = (self.get_text_length)(&self.label);

        let start = min_label_height + label_margin;
        let end = Length::new(LOCAL_AXIS_HEIGHT)
            - top_padding
            - label_padding
            - label_height
            - label_padding
            - max_label_height
            - label_margin;

        let start = start.lerp(end, self.visible_data_range_normalized.0);
        let end = start.lerp(end, self.visible_data_range_normalized.1);

        (
            Position::new((POSITION_X, start.0)),
            Position::new((POSITION_X, end.0)),
        )
    }

    /// Returns the local position of the label.
    pub fn label_position(&self) -> Position<LocalSpace> {
        const POSITION_X: f32 = 0.0;

        let (_, top_padding) = (self.get_rem_length)(AXIS_TOP_PADDING);
        let (_, label_height) = (self.get_text_length)(&self.label);
        let (_, padding_height) = (self.get_rem_length)(AXIS_LINE_PADDING_REM);

        Position::new((
            POSITION_X,
            LOCAL_AXIS_HEIGHT - top_padding.0 - padding_height.0 - label_height.0,
        ))
    }

    /// Returns the local position of the min label.
    pub fn min_label_position(&self) -> Position<LocalSpace> {
        let (_, label_margin) = (self.get_rem_length)(LABEL_MARGIN_REM);
        let (_, min_label_height) = (self.get_text_length)(&self.min_label);

        let (start, _) = self.axis_line_range();

        Position::new((start.x, start.y - label_margin.0 - min_label_height.0))
    }

    /// Returns the local position of the max label.
    pub fn max_label_position(&self) -> Position<LocalSpace> {
        let (_, label_margin) = (self.get_rem_length)(LABEL_MARGIN_REM);
        let (_, max_label_height) = (self.get_text_length)(&self.max_label);

        let (_, end) = self.axis_line_range();

        Position::new((end.x, end.y + label_margin.0 + max_label_height.0))
    }

    pub fn ticks_range(&self, expanded: bool) -> (Position<LocalSpace>, Position<LocalSpace>) {
        let (start, end) = self.axis_line_range();

        let (start, end) = if expanded {
            let extends = self.curves_bounding_box();
            let (_, start_y) = start.extract();
            let (_, end_y) = end.extract();

            let (x, _) = extends.start().extract();

            (Position::new((x, start_y)), Position::new((x, end_y)))
        } else {
            (start, end)
        };

        let ticks_padding = (self.get_rem_length)(TICKS_PADDING_REM).0;
        let offset = Offset::new((ticks_padding.0, self.max_tick_height.0 / 2.0));

        let start = start - offset;
        let end = end - offset;

        (start, end)
    }

    /// Returns a transformer to map between the world space and local space.
    pub fn space_transformer(
        &self,
    ) -> impl CoordinateSystemTransformer<WorldSpace, LocalSpace>
           + CoordinateSystemTransformer<LocalSpace, WorldSpace> {
        WorldLocalTransformer::new(self.world_offset.get(), AXIS_LOCAL_Y_SCALE)
    }

    /// Sets the world offset of the axis.
    pub fn set_world_offset(&self, offset: f32) {
        self.world_offset.set(offset)
    }

    /// Returns the world offset of the axis.
    pub fn world_offset(&self) -> f32 {
        self.world_offset.get()
    }

    /// Returns the left neighbor of the axis.
    pub fn left_neighbor(&self) -> Option<Rc<Self>> {
        self.left.borrow().clone()
    }

    /// Sets the left neighbor of the axis.
    pub fn set_left_neighbor(&self, axis: Option<&Rc<Self>>) {
        *self.left.borrow_mut() = axis.cloned();
    }

    /// Returns the left neighbor of the axis.
    pub fn right_neighbor(&self) -> Option<Rc<Self>> {
        self.right.borrow().clone()
    }

    /// Sets the left neighbor of the axis.
    pub fn set_right_neighbor(&self, axis: Option<&Rc<Self>>) {
        *self.right.borrow_mut() = axis.cloned();
    }

    pub fn swap_axis_order_left(this: &Rc<Self>) -> bool {
        if let Some(left) = this.left_neighbor() {
            let left_left = left.left_neighbor();
            let right = this.right_neighbor();

            if let Some(left_left) = &left_left {
                left_left.set_right_neighbor(Some(this));
            }

            this.set_left_neighbor(left_left.as_ref());
            this.set_right_neighbor(Some(&left));

            left.set_world_offset(left.world_offset() + 1.0);
            left.set_left_neighbor(Some(this));
            left.set_right_neighbor(right.as_ref());

            if let Some(right) = right {
                right.set_left_neighbor(Some(&left));
            }

            let axes = this.axes();
            let mut axes = axes.borrow_mut();
            if axes.is_first_visible_axis(&left) {
                axes.set_first_visible_axis(this.clone());
            }

            if axes.is_last_visible_axis(this) {
                axes.set_last_visible_axis(left.clone());
            }

            true
        } else {
            false
        }
    }

    pub fn swap_axis_order_right(this: &Rc<Self>) -> bool {
        if let Some(right) = this.right_neighbor() {
            let left = this.left_neighbor();
            let right_right = right.right_neighbor();

            if let Some(left) = &left {
                left.set_right_neighbor(Some(&right));
            }

            right.set_world_offset(right.world_offset() - 1.0);
            right.set_left_neighbor(left.as_ref());
            right.set_right_neighbor(Some(this));

            this.set_left_neighbor(Some(&right));
            this.set_right_neighbor(right_right.as_ref());

            if let Some(right_right) = right_right {
                right_right.set_left_neighbor(Some(this));
            }

            let axes = this.axes();
            let mut axes = axes.borrow_mut();
            if axes.is_first_visible_axis(this) {
                axes.set_first_visible_axis(right.clone());
            }

            if axes.is_last_visible_axis(&right) {
                axes.set_last_visible_axis(this.clone());
            }

            true
        } else {
            false
        }
    }

    /// Returns the [`Axes`] object this axis is assigned to.
    pub fn axes(&self) -> Rc<RefCell<Axes>> {
        self.axes
            .upgrade()
            .expect("an axis should outlive the axes object")
    }
}

impl Debug for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Axis")
            .field("key", &self.key)
            .field("label", &self.label)
            .field("min_label", &self.min_label)
            .field("max_label", &self.max_label)
            .field("state", &self.state)
            .field("axis_index", &self.axis_index)
            .field("data", &self.data)
            .field("data_normalized", &self.data_normalized)
            .field("data_range", &self.data_range)
            .field("visible_data_range", &self.visible_data_range)
            .field(
                "visible_data_range_normalized",
                &self.visible_data_range_normalized,
            )
            .field("world_offset", &self.world_offset)
            .field("axes", &self.axes)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish_non_exhaustive()
    }
}

/// State that an [`Axis`] can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisState {
    Collapsed,
    Expanded,
}

pub type RemLengthFunc<T> = dyn Fn(f32) -> Length<T>;
type RemLengthFunc2<T> = dyn Fn(f32) -> (Length<T>, Length<T>);
pub type TextLengthFunc<T> = dyn Fn(&str) -> (Length<T>, Length<T>);

/// A collection of axes.
#[derive(Clone)]
pub struct Axes {
    axes: BTreeMap<String, Rc<Axis>>,

    num_visible_axes: usize,
    visible_axis_start: Option<Rc<Axis>>,
    visible_axis_end: Option<Rc<Axis>>,

    num_data_points: Option<usize>,
    next_axis_index: usize,

    coordinate_mappings: Rc<RefCell<AxesCoordinateMappings>>,

    get_rem_length_screen: Rc<RemLengthFunc<ScreenSpace>>,

    get_rem_length_world: Rc<RemLengthFunc2<WorldSpace>>,

    get_rem_length_local: Rc<RemLengthFunc2<LocalSpace>>,
    get_text_length_local: Rc<TextLengthFunc<LocalSpace>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AxesCoordinateMappings {
    view_height: f32,
    view_width: f32,
    world_width: f32,

    view_bounding_box: Aabb<ViewSpace>,
    world_bounding_box: Aabb<WorldSpace>,
}

impl Axes {
    /// Constructs a new instance.
    #[allow(clippy::type_complexity)]
    pub fn new(
        view_bounding_box: Aabb<ViewSpace>,
        get_rem_length_screen: Rc<dyn Fn(f32) -> Length<ScreenSpace>>,
        get_text_length_screen: Rc<dyn Fn(&str) -> (Length<ScreenSpace>, Length<ScreenSpace>)>,
    ) -> Self {
        let (view_width, view_height) = view_bounding_box.size().extract();
        let coordinate_mappings = Rc::new(RefCell::new(AxesCoordinateMappings {
            view_height,
            view_width,
            world_width: 1.0,
            view_bounding_box,
            world_bounding_box: Aabb::new(Position::new((-0.5, 0.0)), Position::new((1.0, 1.0))),
        }));

        let get_rem_length_world = {
            let coordinate_mappings = coordinate_mappings.clone();
            let get_rem_length_screen = get_rem_length_screen.clone();
            Rc::new(move |rem| {
                let mappings = coordinate_mappings.borrow();

                let length = get_rem_length_screen(rem);
                let p0 = Offset::<ScreenSpace>::zero();
                let p1 = Offset::<ScreenSpace>::from_length_at_axis(0, length);
                let p2 = Offset::<ScreenSpace>::from_length_at_axis(1, length);

                let mapper = ScreenViewTransformer::new(mappings.view_height);
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let mapper = ViewWorldTransformer::new(
                    mappings.view_height,
                    mappings.view_width,
                    mappings.world_width,
                    0.5,
                );
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let w = p1 - p0;
                let h = p2 - p0;

                (w.into(), h.into())
            })
        };

        let get_rem_length_local = {
            let coordinate_mappings = coordinate_mappings.clone();
            let get_rem_length_screen = get_rem_length_screen.clone();
            Rc::new(move |rem| {
                let mappings = coordinate_mappings.borrow();

                let length = get_rem_length_screen(rem);
                let p0 = Offset::<ScreenSpace>::zero();
                let p1 = Offset::<ScreenSpace>::from_length_at_axis(0, length);
                let p2 = Offset::<ScreenSpace>::from_length_at_axis(1, length);

                let mapper = ScreenViewTransformer::new(mappings.view_height);
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let mapper = ViewWorldTransformer::new(
                    mappings.view_height,
                    mappings.view_width,
                    mappings.world_width,
                    0.5,
                );
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let mapper = WorldLocalTransformer::new(0.0, AXIS_LOCAL_Y_SCALE);
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let w = p1 - p0;
                let h = p2 - p0;

                (w.into(), h.into())
            })
        };

        let get_text_length_local = {
            let coordinate_mappings = coordinate_mappings.clone();
            let get_text_length_screen = get_text_length_screen.clone();
            Rc::new(move |text: &str| {
                let mappings = coordinate_mappings.borrow();

                let (width, height) = get_text_length_screen(text);
                let p0 = Offset::<ScreenSpace>::zero();
                let p1 = Offset::<ScreenSpace>::from_length_at_axis(0, width);
                let p2 = Offset::<ScreenSpace>::from_length_at_axis(1, height);

                let mapper = ScreenViewTransformer::new(mappings.view_height);
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let mapper = ViewWorldTransformer::new(
                    mappings.view_height,
                    mappings.view_width,
                    mappings.world_width,
                    0.5,
                );
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let mapper = WorldLocalTransformer::new(0.0, AXIS_LOCAL_Y_SCALE);
                let p0 = p0.transform(&mapper);
                let p1 = p1.transform(&mapper);
                let p2 = p2.transform(&mapper);

                let w = p1 - p0;
                let h = p2 - p0;

                (w.into(), h.into())
            })
        };

        Self {
            axes: Default::default(),
            num_visible_axes: 0,
            visible_axis_start: None,
            visible_axis_end: None,
            num_data_points: None,
            next_axis_index: 0,
            coordinate_mappings,
            get_rem_length_screen,
            get_rem_length_world,
            get_rem_length_local,
            get_text_length_local,
        }
    }

    /// Constructs a new instance.
    #[allow(clippy::type_complexity)]
    pub fn new_rc(
        view_bounding_box: Aabb<ViewSpace>,
        get_rem_length_screen: Rc<dyn Fn(f32) -> Length<ScreenSpace>>,
        get_text_length_screen: Rc<dyn Fn(&str) -> (Length<ScreenSpace>, Length<ScreenSpace>)>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(
            view_bounding_box,
            get_rem_length_screen,
            get_text_length_screen,
        )))
    }

    /// Returns the number of data points for each axis.
    pub fn num_data_points(&self) -> usize {
        self.num_data_points.unwrap_or(0)
    }

    /// Returns the number of visible axes.
    pub fn num_visible_axes(&self) -> usize {
        self.num_visible_axes
    }

    /// Constructs and inserts a new instance of an [`Axis`].
    #[allow(clippy::too_many_arguments)]
    pub fn construct_axis(
        &mut self,
        this: &Rc<RefCell<Self>>,
        key: &str,
        label: &str,
        data: Box<[f32]>,
        range: Option<(f32, f32)>,
        visible_range: Option<(f32, f32)>,
        ticks: Option<Vec<(f32, Option<Rc<str>>)>>,
        num_labels: usize,
    ) -> Rc<Axis> {
        if !std::ptr::eq(self, this.as_ptr()) {
            panic!("this does not point to the same instance as self");
        }

        if self.axes.contains_key(key) {
            panic!("axis {key:?} already exists");
        }

        if let Some(num_data_points) = self.num_data_points {
            if num_data_points != data.len() {
                panic!("unexpected number of data points for axis {key:?}, expected {num_data_points}, but got {}", data.len());
            }
        } else {
            self.num_data_points = Some(data.len());
        }

        let mut args = AxisArgs::new(label, data);
        if let Some((min, max)) = range {
            args = args.with_range(min, max);
        }
        if let Some((min, max)) = visible_range {
            args = args.with_visible_range(min, max);
        }
        if let Some(ticks) = ticks {
            args = args.with_ticks(ticks);
        }

        let axis = Rc::new(Axis::new(
            key,
            args,
            None,
            0.0,
            num_labels,
            this,
            self.get_rem_length_local.clone(),
            self.get_text_length_local.clone(),
        ));

        self.axes.insert(key.into(), axis.clone());

        if !axis.is_hidden() {
            self.num_visible_axes += 1;
            let mut mappings = self.coordinate_mappings.borrow_mut();
            mappings.world_width = ((self.num_visible_axes + 1) as f32).max(1.0);
            mappings.world_bounding_box = Aabb::new(
                Position::new((-0.5, 0.0)),
                Position::new((mappings.world_width, 1.0)),
            );
            drop(mappings);

            // Axis is the first visible axis, so we set it to the start and end of the list.
            if self.num_visible_axes == 1 {
                axis.set_world_offset(0.0);
                self.visible_axis_start = Some(axis.clone());
                self.visible_axis_end = Some(axis.clone());
            }
            // Otherwise we append it to the end of the visible axes.
            else {
                let last_axis = self.visible_axis_end.as_ref().unwrap();
                last_axis.set_right_neighbor(Some(&axis));

                let last_axis_offset = last_axis.world_offset();
                let axis_offset = last_axis_offset.floor() + 1.0;
                axis.set_world_offset(axis_offset);

                axis.set_left_neighbor(Some(last_axis));
                self.visible_axis_end = Some(axis.clone());
            }
        }

        axis
    }

    /// Removes an axis from the plot.
    pub fn remove_axis(&mut self, axis: &str) {
        let axis = self.axes.remove(axis).expect("axis is missing");
        if !axis.is_hidden() {
            self.next_axis_index -= 1;
            self.num_visible_axes -= 1;
            let mut mappings = self.coordinate_mappings.borrow_mut();
            mappings.world_width = ((self.num_visible_axes + 1) as f32).max(1.0);
            mappings.world_bounding_box = Aabb::new(
                Position::new((-0.5, 0.0)),
                Position::new((mappings.world_width, 1.0)),
            );
            drop(mappings);

            if let Some(left) = axis.left_neighbor() {
                left.set_right_neighbor(axis.right_neighbor().as_ref());
            } else {
                self.visible_axis_start = axis.right_neighbor();
            }

            if let Some(right) = axis.right_neighbor() {
                right.set_left_neighbor(axis.left_neighbor().as_ref());
            } else {
                self.visible_axis_end = axis.left_neighbor();
            }

            for ax in self.visible_axes() {
                if ax.axis_index() > axis.axis_index() {
                    let new_idx = ax.axis_index().unwrap() - 1;
                    ax.axis_index.set(Some(new_idx));
                }
                if ax.world_offset() > axis.world_offset() {
                    let new_world_offset = ax.world_offset() - 1.0;
                    ax.set_world_offset(new_world_offset);
                }
            }
        }

        if self.axes.is_empty() {
            self.num_data_points = None;
        }
    }

    /// Returns the order of the axes.
    #[allow(dead_code)]
    pub fn axes_order(&self) -> Box<[Box<str>]> {
        self.visible_axes().map(|ax| (*ax.key()).into()).collect()
    }

    pub fn set_axes_order(&mut self, order: &[impl AsRef<str>]) {
        if order.iter().any(|x| !self.axes.contains_key(x.as_ref())) {
            panic!("the provided order references an unknown axis")
        }

        for ax in self.visible_axes() {
            ax.set_world_offset(0.0);
            ax.axis_index.set(None);

            if let Some(left) = ax.left_neighbor() {
                left.set_right_neighbor(None);
            }
            ax.set_left_neighbor(None);
        }
        self.visible_axis_start = None;
        self.visible_axis_end = None;

        self.next_axis_index = order.len();
        self.num_visible_axes = order.len();

        let axes = order
            .iter()
            .map(|ax| self.axes[ax.as_ref()].clone())
            .collect::<Vec<_>>();
        for i in 0..axes.len() {
            let ax = &axes[i];
            ax.set_world_offset(i as f32);
            ax.axis_index.set(Some(i));

            if i != 0 {
                ax.set_left_neighbor(Some(&axes[i - 1]))
            }
            if i != axes.len() - 1 {
                ax.set_right_neighbor(Some(&axes[i + 1]))
            }
        }

        if let Some(first) = axes.first() {
            self.visible_axis_start = Some(first.clone());
        }
        if let Some(last) = axes.last() {
            self.visible_axis_end = Some(last.clone());
        }

        let mut mappings = self.coordinate_mappings.borrow_mut();
        mappings.world_width = ((self.num_visible_axes + 1) as f32).max(1.0);
        mappings.world_bounding_box = Aabb::new(
            Position::new((-0.5, 0.0)),
            Position::new((mappings.world_width, 1.0)),
        );

        if order.len() != self.num_visible_axes
            || order.iter().any(|x| !self.axes.contains_key(x.as_ref()))
        {
            panic!("the provided order must contain all axes");
        }
    }

    /// Returns the axis assigned to the `key`.
    pub fn axis(&self, key: &str) -> Option<Rc<Axis>> {
        self.axes.get(key).cloned()
    }

    /// Sets the bounding box of the view space.
    pub fn set_view_bounding_box(&self, view_bounding_box: Aabb<ViewSpace>) {
        let (view_width, view_height) = view_bounding_box.size().extract();

        let mut mappings = self.coordinate_mappings.borrow_mut();
        mappings.view_bounding_box = view_bounding_box;
        mappings.view_width = view_width;
        mappings.view_height = view_height;
    }

    /// Returns the axis line size.
    pub fn axis_line_size(&self) -> (Length<WorldSpace>, Length<WorldSpace>) {
        (self.get_rem_length_world)(AXIS_LINE_SIZE_REM)
    }

    /// Returns the data line size.
    pub fn data_line_size(&self) -> (Length<WorldSpace>, Length<WorldSpace>) {
        (self.get_rem_length_world)(DATA_LINE_SIZE_REM)
    }

    /// Returns the selections line size.
    pub fn selections_line_size(&self) -> (Length<WorldSpace>, Length<WorldSpace>) {
        (self.get_rem_length_world)(SELECTION_LINE_SIZE_REM)
    }

    /// Returns the curve line size.
    pub fn curve_line_size(&self) -> (Length<WorldSpace>, Length<WorldSpace>) {
        (self.get_rem_length_world)(CURVE_LINE_SIZE_REM)
    }

    pub fn control_points_radius(&self) -> Length<ScreenSpace> {
        (self.get_rem_length_screen)(CONTROL_POINTS_RADIUS_REM)
    }

    fn control_points_radius_local(&self) -> (Length<LocalSpace>, Length<LocalSpace>) {
        (self.get_rem_length_local)(CONTROL_POINTS_RADIUS_REM)
    }

    pub fn element_at_position(
        &self,
        position: Position<ScreenSpace>,
        active_label_idx: Option<usize>,
    ) -> Option<Element> {
        let position = position.transform(&self.space_transformer());
        {
            let mappings = self.coordinate_mappings.borrow();
            if !mappings.world_bounding_box.contains_point(&position) {
                return None;
            }
        }

        let handle_collapsed = |ax: Rc<Axis>, position: Position<LocalSpace>, active_label_idx| {
            // Check if we are hovering a group.
            let bounding_box = ax.selections_bounding_box(active_label_idx);
            if bounding_box.contains_point(&position) {
                let range = ax.axis_line_range();
                let axis_value = position.y.inv_lerp(range.0.y, range.1.y);
                let curve_builder = ax.borrow_selection_curve_builder(active_label_idx);

                let group = curve_builder.get_group_containing(axis_value);
                if let Some(group) = group {
                    drop(curve_builder);
                    return Some(Element::Group {
                        axis: ax,
                        group_idx: group,
                    });
                }
            }

            None
        };

        let handle_expanded = |ax: Rc<Axis>, position, active_label_idx| {
            // Check if we are hovering a selection.
            let bounding_box = ax.selections_bounding_box(active_label_idx);
            if bounding_box.contains_point(&position) {
                if let Some(rank) = ax.selection_rank_at_position(&position, active_label_idx) {
                    let (axis_start, axis_end) = ax.axis_line_range();
                    let control_points = {
                        let curve_builder = ax.borrow_selection_curve_builder(active_label_idx);
                        curve_builder.get_selection_control_points().into_vec()
                    };

                    let (_, control_point_height) = self.control_points_radius_local();
                    let padding = control_point_height.extract::<f32>();

                    for (selection_idx, (selection_rank, control_points)) in
                        control_points.into_iter().enumerate()
                    {
                        if selection_rank != rank {
                            continue;
                        }

                        let (selection_start, selection_end) = (
                            axis_start.lerp(axis_end, *control_points.first().unwrap()),
                            axis_start.lerp(axis_end, *control_points.last().unwrap()),
                        );
                        let (_, start_y) = selection_start.extract();
                        let (_, end_y) = selection_end.extract();

                        let selection_range = start_y..=end_y;
                        let extended_selection_range = start_y - padding..=end_y + padding;

                        if !extended_selection_range.contains(&position.y) {
                            continue;
                        }

                        for (i, point) in control_points.into_iter().enumerate() {
                            if !(0.0..=1.0).contains(&point) {
                                continue;
                            }
                            let (_, y) = axis_start.lerp(axis_end, point).extract();
                            let hovering_range = y - padding..=y + padding;
                            if hovering_range.contains(&position.y) {
                                return Some(Element::AxisControlPoint {
                                    axis: ax,
                                    selection_idx,
                                    control_point_idx: i,
                                });
                            }
                        }

                        if selection_range.contains(&position.y) {
                            return Some(Element::Brush {
                                axis: ax,
                                selection_idx,
                            });
                        }
                    }
                }
            }

            let (cp_radius_w, cp_radius_h) = self.control_points_radius_local();
            let bounding_box = ax.curves_bounding_box();
            if bounding_box.contains_point(&position) {
                let (axis_start, axis_end) = ax.axis_line_range();
                let curve_builder = ax.borrow_selection_curve_builder(active_label_idx);
                for (selection_idx, selection) in curve_builder.selections().iter().enumerate() {
                    for (control_point_idx, &(x, y)) in
                        selection.control_points().iter().enumerate()
                    {
                        let cp_position = axis_start.lerp(axis_end, x);
                        let cp_position = cp_position + ax.curve_offset_at_curve_value(y);

                        let offset = Offset::<LocalSpace>::new((cp_radius_w.0, cp_radius_h.0));
                        let start = cp_position - offset;
                        let end = cp_position + offset;
                        let bb = Aabb::new(start, end);
                        if bb.contains_point(&position) {
                            drop(curve_builder);
                            return Some(Element::CurveControlPoint {
                                axis: ax,
                                selection_idx,
                                control_point_idx,
                            });
                        }
                    }
                }
            }

            None
        };

        for ax in self.visible_axes() {
            let position = position.transform(&ax.space_transformer());

            // Check if we are inside the bounding box of the axis.
            let bounding_box = ax.bounding_box(active_label_idx);
            if !bounding_box.contains_point(&position) {
                continue;
            }

            // Check if we are hovering the label.
            let bounding_box = ax.label_bounding_box();
            if bounding_box.contains_point(&position) {
                return Some(Element::Label { axis: ax });
            }

            let el = if let Some(active_label_idx) = active_label_idx {
                if ax.is_expanded() {
                    handle_expanded(ax.clone(), position, active_label_idx)
                } else {
                    handle_collapsed(ax.clone(), position, active_label_idx)
                }
            } else {
                None
            };

            if el.is_some() {
                return el;
            }

            // Check if we are hovering the axis line.
            let bounding_box = ax.axis_line_bounding_box();
            if bounding_box.contains_point(&position) {
                return Some(Element::AxisLine { axis: ax });
            }

            return None;
        }

        None
    }

    /// Return the t range of the probability curve.
    pub fn curve_t_range(&self) -> (f32, f32) {
        (MIN_CURVE_T, MAX_CURVE_T)
    }

    /// Returns the width of the world space.
    #[allow(dead_code)]
    pub fn world_width(&self) -> f32 {
        let mappings = self.coordinate_mappings.borrow();
        mappings.world_width
    }

    /// Returns a transformer to map between the screen space and world space.
    pub fn space_transformer(
        &self,
    ) -> impl CoordinateSystemTransformer<ScreenSpace, WorldSpace>
           + CoordinateSystemTransformer<WorldSpace, ScreenSpace> {
        struct ScreenWorldTransformer {
            screen: ScreenViewTransformer,
            world: ViewWorldTransformer,
        }

        impl CoordinateSystemTransformer<ScreenSpace, WorldSpace> for ScreenWorldTransformer {
            fn transform_position(
                &self,
                position: <ScreenSpace as CoordinateSystem>::Position,
            ) -> <WorldSpace as CoordinateSystem>::Position {
                let position = <ScreenViewTransformer as CoordinateSystemTransformer<
                    ScreenSpace,
                    ViewSpace,
                >>::transform_position(&self.screen, position);
                <ViewWorldTransformer as CoordinateSystemTransformer<
                    ViewSpace,
                    WorldSpace,
                >>::transform_position(&self.world, position)
            }

            fn transform_offset(
                &self,
                offset: <ScreenSpace as CoordinateSystem>::Offset,
            ) -> <WorldSpace as CoordinateSystem>::Offset {
                let offset = <ScreenViewTransformer as CoordinateSystemTransformer<
                    ScreenSpace,
                    ViewSpace,
                >>::transform_offset(&self.screen, offset);
                <ViewWorldTransformer as CoordinateSystemTransformer<
                    ViewSpace,
                    WorldSpace,
                >>::transform_offset(&self.world, offset)
            }
        }

        impl CoordinateSystemTransformer<WorldSpace, ScreenSpace> for ScreenWorldTransformer {
            fn transform_position(
                &self,
                position: <WorldSpace as CoordinateSystem>::Position,
            ) -> <ScreenSpace as CoordinateSystem>::Position {
                let position = <ViewWorldTransformer as CoordinateSystemTransformer<
                    WorldSpace,
                    ViewSpace,
                >>::transform_position(&self.world, position);
                <ScreenViewTransformer as CoordinateSystemTransformer<
                    ViewSpace,
                    ScreenSpace,
                >>::transform_position(&self.screen, position)
            }

            fn transform_offset(
                &self,
                offset: <WorldSpace as CoordinateSystem>::Offset,
            ) -> <ScreenSpace as CoordinateSystem>::Offset {
                let offset = <ViewWorldTransformer as CoordinateSystemTransformer<
                    WorldSpace,
                    ViewSpace,
                >>::transform_offset(&self.world, offset);
                <ScreenViewTransformer as CoordinateSystemTransformer<
                    ViewSpace,
                    ScreenSpace,
                >>::transform_offset(&self.screen, offset)
            }
        }

        let mappings = self.coordinate_mappings.borrow();
        let screen = ScreenViewTransformer::new(mappings.view_height);
        let world = ViewWorldTransformer::new(
            mappings.view_height,
            mappings.view_width,
            mappings.world_width,
            0.5,
        );

        ScreenWorldTransformer { screen, world }
    }

    /// Returns an iterator over all contained axes.
    #[allow(dead_code)]
    pub fn axes(&self) -> impl Iterator<Item = Rc<Axis>> + '_ {
        self.axes.values().cloned()
    }

    /// Returns an iterator over the visible axes contained.
    pub fn visible_axes(&self) -> VisibleAxes<'_> {
        VisibleAxes {
            start: self.visible_axis_start.clone(),
            end: self.visible_axis_end.clone(),
            len: self.num_visible_axes,
            _phantom: PhantomData,
        }
    }

    pub fn viewport(&self, pixel_ratio: f32) -> ((f32, f32), (f32, f32)) {
        let mappings = self.coordinate_mappings.borrow();
        let (start_x, start_y) = mappings.view_bounding_box.start().extract();
        let (end_x, end_y) = mappings.view_bounding_box.end().extract();

        let width = end_x - start_x;
        let height = end_y - start_y;

        let start = (
            (start_x * pixel_ratio).floor(),
            ((mappings.view_height - end_y) * pixel_ratio).floor(),
        );
        let size = (
            (width * pixel_ratio).floor(),
            (height * pixel_ratio).floor(),
        );
        (start, size)
    }

    fn is_first_visible_axis(&self, axis: &Rc<Axis>) -> bool {
        if let Some(start) = &self.visible_axis_start {
            Rc::ptr_eq(axis, start)
        } else {
            false
        }
    }

    fn is_last_visible_axis(&self, axis: &Rc<Axis>) -> bool {
        if let Some(end) = &self.visible_axis_end {
            Rc::ptr_eq(axis, end)
        } else {
            false
        }
    }

    fn set_first_visible_axis(&mut self, axis: Rc<Axis>) {
        self.visible_axis_start = Some(axis);
    }

    fn set_last_visible_axis(&mut self, axis: Rc<Axis>) {
        self.visible_axis_end = Some(axis);
    }
}

impl Debug for Axes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Axes")
            .field("axes", &self.axes)
            .field("num_visible_axes", &self.num_visible_axes)
            .field("visible_axis_start", &self.visible_axis_start)
            .field("visible_axis_end", &self.visible_axis_end)
            .field("num_data_points", &self.num_data_points)
            .field("next_axis_index", &self.next_axis_index)
            .field("coordinate_mappings", &self.coordinate_mappings)
            .finish_non_exhaustive()
    }
}

/// An element inside an [`Axes`] instance.
#[derive(Debug, Clone)]
pub enum Element {
    Label {
        axis: Rc<Axis>,
    },
    Group {
        axis: Rc<Axis>,
        group_idx: usize,
    },
    Brush {
        axis: Rc<Axis>,
        selection_idx: usize,
    },
    AxisControlPoint {
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
    },
    CurveControlPoint {
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
    },
    AxisLine {
        axis: Rc<Axis>,
    },
}

/// An iterator over the visible axes.
#[derive(Debug, Clone)]
pub struct VisibleAxes<'a> {
    len: usize,
    start: Option<Rc<Axis>>,
    end: Option<Rc<Axis>>,
    _phantom: PhantomData<&'a Axes>,
}

impl ExactSizeIterator for VisibleAxes<'_> {}

impl Iterator for VisibleAxes<'_> {
    type Item = Rc<Axis>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.start.clone()?;
        self.start = current.right_neighbor();

        self.len -= 1;
        if self.len == 0 {
            self.start = None;
            self.end = None;
        }

        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl DoubleEndedIterator for VisibleAxes<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let current = self.end.clone()?;
        self.end = current.left_neighbor();

        self.len -= 1;
        if self.len == 0 {
            self.start = None;
            self.end = None;
        }

        Some(current)
    }
}
