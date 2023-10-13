use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fmt::Debug,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::{
    coordinates::{
        Aabb, CartesianLength, CartesianOffset, CartesianPosition, CoordinateSystemTransformer,
        Length, LocalSpace, Offset, Position, ViewSpace, WorldLocalTransformer, WorldSpace,
    },
    lerp::Lerp,
};

const AXIS_LOCAL_Y_SCALE: f32 = 0.8;
const AXIS_LINE_WIDTH_REM: f32 = 0.05;
const AXIS_LINE_PADDING_REM: f32 = 0.0;

const LABEL_PADDING_REM: f32 = 1.0;
const LABEL_MARGIN_REM: f32 = 1.0;

pub struct AxisArgs {
    label: Rc<str>,
    datums: Box<[f32]>,
    range: (f32, f32),
    min_range: (f32, f32),
    visible_range: (f32, f32),
    state: AxisState,
}

impl AxisArgs {
    /// Constructs a new instance with default settings.
    pub fn new(label: &str, datums: &[f32]) -> Self {
        let datums = datums.iter().cloned().filter(|x| !x.is_nan());

        let min = datums.clone().min_by(|x, y| x.partial_cmp(y).unwrap());
        let max = datums.clone().max_by(|x, y| x.partial_cmp(y).unwrap());

        let mut range = min.zip(max).unwrap_or((0.0, 1.0));
        if range.0 == range.1 {
            range.0 -= 0.5;
            range.1 += 0.5;
        }

        let min_range = range;
        let visible_range = range;

        Self {
            label: label.into(),
            datums: datums.collect(),
            range,
            min_range,
            visible_range,
            state: AxisState::Collapsed,
        }
    }

    /// Sets the range of the axis.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        assert!(min < max, "min must be smaller than max");
        assert!(min.is_finite(), "the minimum must be finite");
        assert!(max.is_finite(), "the maximum must be finite");
        assert!(
            min <= self.min_range.0 && max >= self.min_range.1,
            "the range must be bigger or equal to the min/max of the datums"
        );

        self.range = (min, max);
        self.visible_range.0 = self.visible_range.0.clamp(self.range.0, self.range.1);
        self.visible_range.1 = self.visible_range.1.clamp(self.range.0, self.range.1);
        self
    }

    /// Sets the visible range of the axis.
    pub fn with_visible_range(mut self, min: f32, max: f32) -> Self {
        assert!(min < max, "min must be smaller than max");
        assert!(min.is_finite(), "the minimum must be finite");
        assert!(max.is_finite(), "the maximum must be finite");
        assert!(
            min >= self.range.0 && max <= self.range.1,
            "the range must be smaller or equal to the datums range"
        );

        self.visible_range = (min, max);
        self
    }

    /// Marks the axis as hidden.
    pub fn hidden(mut self) -> Self {
        self.state = AxisState::Hidden;
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
    axis_index: Option<usize>,

    datums: Box<[f32]>,
    datums_normalized: Box<[f32]>,

    datums_range: (f32, f32),
    visible_datums_range: (f32, f32),
    visible_datums_range_normalized: (f32, f32),

    world_offset: Cell<f32>,

    get_rem_length: Rc<dyn Fn(f32) -> (Length<LocalSpace>, Length<LocalSpace>)>,
    get_text_length: Rc<dyn Fn(&str) -> (Length<LocalSpace>, Length<LocalSpace>)>,

    axes: Weak<RefCell<Axes>>,
    left: RefCell<Option<Weak<Self>>>,
    right: RefCell<Option<Weak<Self>>>,
}

impl Axis {
    fn new(
        key: &str,
        label: &str,
        datums: &[f32],
        range: Option<(f32, f32)>,
        visible_range: Option<(f32, f32)>,
    ) -> Self {
        todo!();

        Self {
            key: key.into(),
            label: label.into(),
            min_label: (),
            max_label: (),
            state: (),
            axis_index: (),
            datums: (),
            datums_normalized: (),
            datums_range: (),
            visible_datums_range: (),
            visible_datums_range_normalized: (),
            world_offset: (),
            get_rem_length: (),
            get_text_length: (),
            axes: (),
            left: (),
            right: (),
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
        matches!(self.state.get(), AxisState::Hidden)
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
        self.axis_index
    }

    /// Fetches the datums of the axis.
    pub fn datums(&self) -> &[f32] {
        &self.datums
    }

    /// Fetches the normalized datums of the axis.
    pub fn datums_normalized(&self) -> &[f32] {
        &self.datums_normalized
    }

    /// Returns the `min` and `max` value of the datums.
    pub fn datums_range(&self) -> (f32, f32) {
        self.datums_range
    }

    /// Returns the `min` and `max` value of the visible datums.
    pub fn visible_datums_range(&self) -> (f32, f32) {
        self.visible_datums_range
    }

    /// Returns the `min` and `max` value of the visible datums, normalized in
    /// relation the the `min` and `max` of all datums.
    pub fn visible_datums_range_normalized(&self) -> (f32, f32) {
        self.visible_datums_range_normalized
    }

    /// Returns the bounding box of the axis.
    pub fn bounding_box(&self) -> Aabb<LocalSpace> {
        let (axis_width, _) = (self.get_rem_length)(AXIS_LINE_PADDING_REM + AXIS_LINE_WIDTH_REM);
        let (label_width, _) = (self.get_text_length)(&self.label);
        let (padding_width, _) = (self.get_rem_length)(AXIS_LINE_PADDING_REM);
        let label_width = label_width + padding_width + padding_width;

        let width = if axis_width <= label_width {
            label_width
        } else {
            axis_width
        };

        let half_width = width / Length::new(CartesianLength(2.0));
        let width_offset =
            Offset::<LocalSpace>::new(CartesianOffset { x: 1.0, y: 0.0 }) * half_width;

        let mut start = Position::<LocalSpace>::new(CartesianPosition { x: 0.5, y: 0.0 });
        let mut end = Position::<LocalSpace>::new(CartesianPosition { x: 0.5, y: 1.0 });

        start -= width_offset;
        end += width_offset;

        if self.is_expanded() {
            start.x = 0.0;

            // TODO: Handle selection curves.
        }

        start.x = start.x.clamp(0.0, 1.0);
        end.x = end.x.clamp(0.0, 1.0);

        Aabb::new(start, end)
    }

    /// Returns the bounding box of the axis line.
    pub fn axis_line_bounding_box(&self) -> Aabb<LocalSpace> {
        let (mut start, mut end) = self.axis_line_range();

        let (axis_width, _) = (self.get_rem_length)(AXIS_LINE_PADDING_REM + AXIS_LINE_WIDTH_REM);
        let half_width = axis_width / Length::new(CartesianLength(2.0));
        let width_offset =
            Offset::<LocalSpace>::new(CartesianOffset { x: 1.0, y: 0.0 }) * half_width;

        start -= width_offset;
        end += width_offset;

        Aabb::new(start, end)
    }

    /// Returns the bounding box of the axis label.
    pub fn label_bounding_box(&self) -> Aabb<LocalSpace> {
        const POSITION_X: f32 = 0.5;

        let (label_width, label_height) = (self.get_text_length)(&self.label);
        let (padding_width, padding_height) = (self.get_rem_length)(AXIS_LINE_PADDING_REM);

        let start = Position::new(CartesianPosition {
            x: POSITION_X - padding_width.0 - (label_width.0 / 2.0),
            y: 1.0 - padding_height.0 - label_height.0 - padding_height.0,
        });
        let end = Position::new(CartesianPosition {
            x: POSITION_X + padding_width.0 + (label_width.0 / 2.0),
            y: 1.0,
        });

        Aabb::new(start, end)
    }

    /// Returns the range of the axis line.
    pub fn axis_line_range(&self) -> (Position<LocalSpace>, Position<LocalSpace>) {
        const POSITION_X: f32 = 0.5;
        let (_, label_padding) = (self.get_rem_length)(LABEL_PADDING_REM);
        let (_, label_margin) = (self.get_rem_length)(LABEL_MARGIN_REM);

        let (_, min_label_height) = (self.get_text_length)(&self.min_label);
        let (_, max_label_height) = (self.get_text_length)(&self.max_label);
        let (_, label_height) = (self.get_text_length)(&self.label);

        let start = min_label_height + label_margin;
        let end = Length::new(CartesianLength(1.0))
            - label_padding
            - label_height
            - label_padding
            - max_label_height
            - label_margin;

        let start = start.lerp(end, self.visible_datums_range_normalized.0);
        let end = start.lerp(end, self.visible_datums_range_normalized.1);

        (
            Position::new(CartesianPosition {
                x: POSITION_X,
                y: start.0,
            }),
            Position::new(CartesianPosition {
                x: POSITION_X,
                y: end.0,
            }),
        )
    }

    /// Returns the local position of the label.
    pub fn label_position(&self) -> Position<LocalSpace> {
        const POSITION_X: f32 = 0.5;

        let (_, label_height) = (self.get_text_length)(&self.label);
        let (_, padding_height) = (self.get_rem_length)(AXIS_LINE_PADDING_REM);

        Position::new(CartesianPosition {
            x: POSITION_X,
            y: 1.0 - padding_height.0 - label_height.0,
        })
    }

    /// Returns the local position of the min label.
    pub fn min_label_position(&self) -> Position<LocalSpace> {
        let (_, label_margin) = (self.get_rem_length)(LABEL_MARGIN_REM);
        let (_, min_label_height) = (self.get_text_length)(&self.min_label);

        let (start, _) = self.axis_line_range();

        Position::new(CartesianPosition {
            x: start.x,
            y: start.y - label_margin.0 - min_label_height.0,
        })
    }

    /// Returns the local position of the max label.
    pub fn max_label_position(&self) -> Position<LocalSpace> {
        let (_, label_margin) = (self.get_rem_length)(LABEL_MARGIN_REM);
        let (_, max_label_height) = (self.get_text_length)(&self.max_label);

        let (_, end) = self.axis_line_range();

        Position::new(CartesianPosition {
            x: end.x,
            y: end.y + label_margin.0 + max_label_height.0,
        })
    }

    /// Returns a transformer to map between the world space and local space.
    pub fn space_transformer(
        &self,
    ) -> impl CoordinateSystemTransformer<WorldSpace, LocalSpace>
           + CoordinateSystemTransformer<LocalSpace, WorldSpace> {
        WorldLocalTransformer::new(self.world_offset.get(), AXIS_LOCAL_Y_SCALE)
    }

    pub fn set_world_offset(&self, offset: f32) {
        // Clamp the offset.

        self.world_offset.set(offset)
    }

    /// Returns the left neighbor of the axis.
    pub fn left_neighbor(&self) -> Option<Rc<Self>> {
        let left = self.left.borrow().clone()?;
        left.upgrade()
    }

    /// Sets the left neighbor of the axis.
    pub fn set_left_neighbor(&self, axis: Option<&Rc<Self>>) {
        if let Some(axis) = axis {
            *self.left.borrow_mut() = Some(Rc::downgrade(axis));
        } else {
            *self.left.borrow_mut() = None;
        }
    }

    /// Returns the left neighbor of the axis.
    pub fn right_neighbor(&self) -> Option<Rc<Self>> {
        let right = self.right.borrow().clone()?;
        right.upgrade()
    }

    /// Sets the left neighbor of the axis.
    pub fn set_right_neighbor(&self, axis: Option<&Rc<Self>>) {
        if let Some(axis) = axis {
            *self.right.borrow_mut() = Some(Rc::downgrade(axis));
        } else {
            *self.right.borrow_mut() = None;
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
            .field("datums", &self.datums)
            .field("datums_normalized", &self.datums_normalized)
            .field("datums_range", &self.datums_range)
            .field("visible_datums_range", &self.visible_datums_range)
            .field(
                "visible_datums_range_normalized",
                &self.visible_datums_range_normalized,
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
    Hidden,
}

/// A collection of axes.
#[derive(Debug)]
pub struct Axes {
    axes: BTreeMap<String, Rc<Axis>>,
    expanded_axis: Option<Rc<Axis>>,

    num_visible_axes: usize,
    visible_axis_start: Option<Rc<Axis>>,
    visible_axis_end: Option<Rc<Axis>>,

    num_datums: Option<usize>,
    next_axis_index: usize,

    view_bounding_box: Aabb<ViewSpace>,
    world_bounding_box: Aabb<WorldSpace>,
}

impl Axes {
    /// Constructs a new instance.
    pub fn new(view_bounding_box: Aabb<ViewSpace>) -> Self {
        Self {
            axes: Default::default(),
            expanded_axis: None,
            num_visible_axes: 0,
            visible_axis_start: None,
            visible_axis_end: None,
            num_datums: None,
            next_axis_index: 0,
            view_bounding_box,
            world_bounding_box: Aabb::new(
                Position::zero(),
                Position::new(CartesianPosition { x: 1.0, y: 1.0 }),
            ),
        }
    }

    /// Constructs a new instance.
    pub fn new_rc(view_bounding_box: Aabb<ViewSpace>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(view_bounding_box)))
    }

    /// Constructs and inserts a new instance of an [`Axis`].
    pub fn construct_axis(
        &mut self,
        this: &Rc<RefCell<Self>>,
        key: &str,
        label: &str,
        datums: &[f32],
    ) -> Rc<Axis> {
        if !std::ptr::eq(self, this.as_ptr()) {
            panic!("this does not point to the same instance as self");
        }

        if let Some(num_datums) = self.num_datums {
            if num_datums != datums.len() {
                panic!("unexpected number of datums for axis {key:?}, expected {num_datums}, but got {}", datums.len());
            }
        } else {
            self.num_datums = Some(datums.len());
        }

        todo!()
    }

    /// Returns the order of the axes.
    pub fn axes_order(&self) -> Box<[Box<str>]> {
        self.visible_axes().map(|ax| (*ax.key()).into()).collect()
    }

    pub fn set_axes_order(&mut self, order: &[impl AsRef<str>]) {
        if order.len() != self.num_visible_axes
            || order.iter().any(|x| !self.axes.contains_key(x.as_ref()))
        {
            panic!("the provided order must contain all axes");
        }

        for (i, key) in order.iter().enumerate() {
            let key = key.as_ref();
            let axis = self.axes[key].clone();

            // Set the position.
            axis.set_world_offset(i as f32 + 1.0);

            // Set left neighbor.
            if i == 0 {
                axis.set_left_neighbor(None);
            } else {
                let previous = &self.axes[order[i - 1].as_ref()];
                axis.set_left_neighbor(Some(previous));
            }

            if i < order.len() - 1 {
                let next = &self.axes[order[i + 1].as_ref()];
                axis.set_right_neighbor(Some(next));
            } else {
                axis.set_right_neighbor(None);
            }
        }
    }

    /// Returns the axis assigned to the `key`.
    pub fn axis(&self, key: &str) -> Option<Rc<Axis>> {
        self.axes.get(key).cloned()
    }

    /// Returns an iterator over all contained axes.
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
