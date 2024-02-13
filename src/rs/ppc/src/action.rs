use std::rc::Rc;

use web_sys::PointerEvent;

use crate::{
    axis::Axis,
    coordinates::{Offset, Position, ScreenSpace},
    event::Event,
    lerp::InverseLerp,
    selection::{Direction, EasingType, Selection, SelectionCurveBuilder},
    wasm_bridge::InteractionMode,
};

#[derive(Debug)]
pub struct Action {
    inner: ActionInner,
}

#[derive(Debug)]
enum ActionInner {
    MoveAxis(MoveAxis),
    SelectGroup(SelectGroup),
    CreateBrush(CreateBrush),
    SelectBrush(SelectBrush),
    SelectAxisCP(SelectAxisCP),
    SelectCurveCP(SelectCurveCP),
}

impl Action {
    pub fn new_move_axis(
        axis: Rc<Axis>,
        event: PointerEvent,
        active_label_idx: Option<usize>,
        interaction_mode: InteractionMode,
    ) -> Self {
        Self {
            inner: ActionInner::MoveAxis(MoveAxis::new(
                axis,
                event,
                active_label_idx,
                interaction_mode,
            )),
        }
    }

    pub fn new_select_group(
        axis: Rc<Axis>,
        group_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::SelectGroup(SelectGroup::new(
                axis,
                group_idx,
                active_label_idx,
                easing_type,
            )),
        }
    }

    pub fn new_create_brush(
        axis: Rc<Axis>,
        event: PointerEvent,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::CreateBrush(CreateBrush::new(
                axis,
                event,
                active_label_idx,
                easing_type,
            )),
        }
    }

    pub fn new_select_brush(
        axis: Rc<Axis>,
        selection_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::SelectBrush(SelectBrush::new(
                axis,
                selection_idx,
                active_label_idx,
                easing_type,
            )),
        }
    }

    pub fn new_select_axis_control_point(
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        event: PointerEvent,
    ) -> Self {
        Self {
            inner: ActionInner::SelectAxisCP(SelectAxisCP::new(
                axis,
                selection_idx,
                control_point_idx,
                active_label_idx,
                easing_type,
                event,
            )),
        }
    }

    pub fn new_select_curve_control_point(
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::SelectCurveCP(SelectCurveCP::new(
                axis,
                selection_idx,
                control_point_idx,
                active_label_idx,
                easing_type,
            )),
        }
    }

    pub fn update(&mut self, event: PointerEvent) -> Event {
        match &mut self.inner {
            ActionInner::MoveAxis(e) => e.update(event),
            ActionInner::SelectGroup(e) => e.update(event),
            ActionInner::CreateBrush(e) => e.update(event),
            ActionInner::SelectBrush(e) => e.update(event),
            ActionInner::SelectAxisCP(e) => e.update(event),
            ActionInner::SelectCurveCP(e) => e.update(event),
        }
    }

    pub fn finish(self) -> Event {
        match self.inner {
            ActionInner::MoveAxis(e) => e.finish(),
            ActionInner::SelectGroup(e) => e.finish(),
            ActionInner::CreateBrush(e) => e.finish(),
            ActionInner::SelectBrush(e) => e.finish(),
            ActionInner::SelectAxisCP(e) => e.finish(),
            ActionInner::SelectCurveCP(e) => e.finish(),
        }
    }
}

#[derive(Debug)]
struct MoveAxis {
    axis: Rc<Axis>,
    moved: bool,
    active_label_idx: Option<usize>,
    start_position: Position<ScreenSpace>,
    interaction_mode: InteractionMode,
}

impl MoveAxis {
    fn new(
        axis: Rc<Axis>,
        event: PointerEvent,
        active_label_idx: Option<usize>,
        interaction_mode: InteractionMode,
    ) -> Self {
        let position =
            Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));

        Self {
            axis,
            moved: false,
            active_label_idx,
            start_position: position,
            interaction_mode,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        let offset = {
            let position = Position::<ScreenSpace>::new((event.offset_x() as f32, 0.0));
            if position.x != self.start_position.x {
                self.moved = true;
            }

            let axes = self.axis.axes();
            let axes = axes.borrow();
            let position = position.transform(&axes.space_transformer());
            position.x.clamp(-0.5, axes.num_visible_axes() as f32)
        };

        self.axis.set_world_offset(offset);
        let bounding_box = self
            .axis
            .bounding_box(self.active_label_idx)
            .transform(&self.axis.space_transformer());

        if let Some(left) = self.axis.left_neighbor() {
            let neighbor_bounding_box = left
                .bounding_box(self.active_label_idx)
                .transform(&left.space_transformer());
            match bounding_box.aabb_relation(&neighbor_bounding_box) {
                crate::coordinates::AabbRelation::Disjoint => {}
                _ => {
                    Axis::swap_axis_order_left(&self.axis);
                    return Event::AXIS_POSITION_CHANGE | Event::AXIS_ORDER_CHANGE;
                }
            }
        }

        if let Some(right) = self.axis.right_neighbor() {
            let neighbor_bounding_box = right
                .bounding_box(self.active_label_idx)
                .transform(&right.space_transformer());
            match bounding_box.aabb_relation(&neighbor_bounding_box) {
                crate::coordinates::AabbRelation::Disjoint => {}
                _ => {
                    Axis::swap_axis_order_right(&self.axis);
                    return Event::AXIS_POSITION_CHANGE | Event::AXIS_ORDER_CHANGE;
                }
            }
        }

        Event::AXIS_POSITION_CHANGE
    }

    fn finish(self) -> Event {
        if let Some(left) = self.axis.left_neighbor() {
            self.axis.set_world_offset(left.world_offset() + 1.0);
        } else if let Some(right) = self.axis.right_neighbor() {
            self.axis.set_world_offset(right.world_offset() - 1.0);
        }

        let enable_state_change = matches!(
            self.interaction_mode,
            InteractionMode::Restricted | InteractionMode::Full
        );

        if !self.moved && enable_state_change {
            match self.axis.state() {
                crate::axis::AxisState::Collapsed => self.axis.expand(),
                crate::axis::AxisState::Expanded => self.axis.collapse(),
            }

            Event::AXIS_STATE_CHANGE | Event::AXIS_POSITION_CHANGE
        } else {
            Event::AXIS_POSITION_CHANGE
        }
    }
}

#[derive(Debug)]
struct SelectGroup {
    axis: Rc<Axis>,
    moved: bool,
    offset: f32,
    group_idx: usize,
    active_label_idx: usize,
    easing_type: EasingType,
    curve_builder: SelectionCurveBuilder,
}

impl SelectGroup {
    fn new(
        axis: Rc<Axis>,
        group_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        let curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();

        Self {
            axis,
            moved: false,
            offset: 0.0,
            group_idx,
            active_label_idx,
            easing_type,
            curve_builder,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        if event.movement_y() == 0 {
            return Event::NONE;
        }
        self.moved = true;

        let offset = {
            let axes = self.axis.axes();
            let axes = axes.borrow();
            let offset = Offset::<ScreenSpace>::new((0.0, event.movement_y() as f32));
            let offset = offset.transform(&axes.space_transformer());
            let offset = offset.transform(&self.axis.space_transformer());

            let (axis_start, axis_end) = self.axis.axis_line_range();
            -offset.y / (axis_end.y - axis_start.y).abs()
        };
        self.offset += offset;

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.offset_group(self.group_idx, self.offset);

        let datums_range = self.axis.visible_data_range_normalized().into();
        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        let mut curve_builder = self.curve_builder;
        let datums_range = self.axis.visible_data_range_normalized().into();

        if !self.moved {
            curve_builder.remove_group(self.group_idx);
        } else if self.offset != 0.0 {
            curve_builder.offset_group(self.group_idx, self.offset);
        }

        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }
}

#[derive(Debug)]
struct CreateBrush {
    axis: Rc<Axis>,
    start_axis_value: f32,
    active_label_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl CreateBrush {
    fn new(
        axis: Rc<Axis>,
        event: PointerEvent,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        let curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();

        let axis_value = {
            let axes = axis.axes();
            let axes = axes.borrow();
            let position =
                Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));
            let position = position.transform(&axes.space_transformer());
            let position = position.transform(&axis.space_transformer());

            let (axis_start, axis_end) = axis.axis_line_range();
            position.y.inv_lerp(axis_start.y, axis_end.y)
        };

        let selection = Selection::new([axis_value, 1.0], [axis_value, 1.0]);

        Self {
            axis,
            active_label_idx,
            easing_type,
            selection,
            curve_builder,
            start_axis_value: axis_value,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        if event.movement_y() == 0 {
            return Event::NONE;
        }

        let axis_value = {
            let axes = self.axis.axes();
            let axes = axes.borrow();
            let position =
                Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));
            let position = position.transform(&axes.space_transformer());
            let position = position.transform(&self.axis.space_transformer());

            let (axis_start, axis_end) = self.axis.axis_line_range();
            position.y.inv_lerp(axis_start.y, axis_end.y)
        };

        if axis_value <= self.start_axis_value {
            self.selection.set_control_point_x(0, axis_value);
        } else {
            self.selection.set_control_point_x(1, axis_value);
        }

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.add_selection(self.selection.clone());

        let datums_range = self.axis.visible_data_range_normalized().into();
        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        let mut curve_builder = self.curve_builder;
        let datums_range = self.axis.visible_data_range_normalized().into();

        if self.selection.control_point_x(0) != self.selection.control_point_x(1) {
            curve_builder.add_selection(self.selection);
        }

        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }
}

#[derive(Debug)]
struct SelectBrush {
    axis: Rc<Axis>,
    moved: bool,
    selection_idx: usize,
    active_label_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl SelectBrush {
    fn new(
        axis: Rc<Axis>,
        selection_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        let mut curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();
        let selection = curve_builder.remove_selection(selection_idx);

        Self {
            axis,
            moved: false,
            selection_idx,
            active_label_idx,
            easing_type,
            selection,
            curve_builder,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        self.moved |= event.movement_y() != 0;

        let offset = {
            let axes = self.axis.axes();
            let axes = axes.borrow();
            let offset = Offset::<ScreenSpace>::new((0.0, event.movement_y() as f32));
            let offset = offset.transform(&axes.space_transformer());
            let offset = offset.transform(&self.axis.space_transformer());

            let (axis_start, axis_end) = self.axis.axis_line_range();
            -offset.y / (axis_end.y - axis_start.y).abs()
        };
        self.selection.offset(offset);

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.insert_selection(self.selection.clone(), self.selection_idx);

        let datums_range = self.axis.visible_data_range_normalized().into();
        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        let mut curve_builder = self.curve_builder;
        let datums_range = self.axis.visible_data_range_normalized().into();

        // If we moved the mouse we do add the modified selection.
        if self.moved {
            curve_builder.insert_selection(self.selection, self.selection_idx);
        }

        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }
}

#[derive(Debug)]
enum SelectAxisCP {
    Selected {
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        selection: Selection,
        curve_builder: SelectionCurveBuilder,
    },
    DraggedSingle {
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        selection: Selection,
        curve_builder: SelectionCurveBuilder,
    },
    DraggedSymmetric {
        axis: Rc<Axis>,
        selection_idx: usize,
        lower_x: f32,
        upper_x: f32,
        extending_start: bool,
        control_point_idx_1: usize,
        control_point_idx_2: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        selection: Selection,
        curve_builder: SelectionCurveBuilder,
    },
    Undefined,
}

impl SelectAxisCP {
    fn new(
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        _event: PointerEvent,
    ) -> Self {
        let mut curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();
        let selection = curve_builder.remove_selection(selection_idx);

        Self::Selected {
            axis,
            selection_idx,
            control_point_idx,
            active_label_idx,
            easing_type,
            selection,
            curve_builder,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        if event.movement_y() == 0 {
            return Event::NONE;
        }

        match self {
            SelectAxisCP::Selected {
                axis,
                selection_idx,
                control_point_idx,
                active_label_idx,
                easing_type,
                selection,
                curve_builder,
            } => 'block: {
                let axis_value = {
                    let axes = axis.axes();
                    let axes = axes.borrow();
                    let position = Position::<ScreenSpace>::new((
                        event.offset_x() as f32,
                        event.offset_y() as f32,
                    ));
                    let position = position.transform(&axes.space_transformer());
                    let position = position.transform(&axis.space_transformer());

                    let (axis_start, axis_end) = axis.axis_line_range();
                    position
                        .y
                        .inv_lerp(axis_start.y, axis_end.y)
                        .clamp(0.0, 1.0)
                };

                let move_direction = if event.movement_y() < 0 {
                    Direction::Up
                } else {
                    Direction::Down
                };
                let create_new = event.shift_key();
                let create_symmetric = (event.ctrl_key() || event.alt_key())
                    && (*control_point_idx == 0
                        || *control_point_idx == selection.num_control_points() - 1);

                if create_symmetric {
                    let lower = selection.control_point_x(0);
                    let upper = selection.control_point_x(selection.num_control_points() - 1);
                    let to_outward = (*control_point_idx == 0 && move_direction == Direction::Down)
                        || (*control_point_idx == selection.num_control_points() - 1
                            && move_direction == Direction::Up);

                    let (control_point_idx_1, control_point_idx_2) = if to_outward {
                        let i1 = selection.insert_control_point(lower, Direction::Down);
                        let i2 = selection.insert_control_point(upper, Direction::Up);
                        (i1, i2)
                    } else {
                        let i1 = selection.insert_control_point(lower, Direction::Up);
                        let i2 = selection.insert_control_point(upper, Direction::Down);
                        (i1, i2)
                    };

                    {
                        let mut curve_builder = curve_builder.clone();
                        curve_builder.insert_selection(selection.clone(), *selection_idx);

                        let datums_range = axis.visible_data_range_normalized().into();
                        axis.borrow_selection_curve_mut(*active_label_idx)
                            .set_curve(curve_builder.build(datums_range, *easing_type));
                        *axis.borrow_selection_curve_builder_mut(*active_label_idx) = curve_builder;
                    }

                    let this = std::mem::replace(self, SelectAxisCP::Undefined);
                    match this {
                        SelectAxisCP::Selected {
                            axis,
                            selection_idx,
                            control_point_idx,
                            active_label_idx,
                            easing_type,
                            selection,
                            curve_builder,
                        } => {
                            *self = SelectAxisCP::DraggedSymmetric {
                                axis,
                                selection_idx,
                                lower_x: lower,
                                upper_x: upper,
                                extending_start: control_point_idx == 0,
                                control_point_idx_1,
                                control_point_idx_2,
                                active_label_idx,
                                easing_type,
                                selection,
                                curve_builder,
                            };
                        }
                        _ => unreachable!(),
                    }
                    break 'block;
                }

                if create_new {
                    let control_point_x = selection.control_point_x(*control_point_idx);
                    let axis_value = if axis_value < control_point_x
                        && move_direction == Direction::Up
                        || axis_value > control_point_x && move_direction == Direction::Down
                    {
                        control_point_x
                    } else {
                        axis_value
                    };
                    *control_point_idx = selection.insert_control_point(axis_value, move_direction);
                } else {
                    selection.set_control_point_x(*control_point_idx, axis_value);
                }

                {
                    let mut curve_builder = curve_builder.clone();
                    curve_builder.insert_selection(selection.clone(), *selection_idx);

                    let datums_range = axis.visible_data_range_normalized().into();
                    axis.borrow_selection_curve_mut(*active_label_idx)
                        .set_curve(curve_builder.build(datums_range, *easing_type));
                    *axis.borrow_selection_curve_builder_mut(*active_label_idx) = curve_builder;
                }

                let this = std::mem::replace(self, SelectAxisCP::Undefined);
                match this {
                    SelectAxisCP::Selected {
                        axis,
                        selection_idx,
                        control_point_idx,
                        active_label_idx,
                        easing_type,
                        selection,
                        curve_builder,
                    } => {
                        *self = SelectAxisCP::DraggedSingle {
                            axis,
                            selection_idx,
                            control_point_idx,
                            active_label_idx,
                            easing_type,
                            selection,
                            curve_builder,
                        };
                    }
                    _ => unreachable!(),
                }
            }
            SelectAxisCP::DraggedSingle {
                axis,
                selection_idx,
                control_point_idx,
                active_label_idx,
                easing_type,
                selection,
                curve_builder,
            } => {
                let axis_value = {
                    let axes = axis.axes();
                    let axes = axes.borrow();
                    let position = Position::<ScreenSpace>::new((
                        event.offset_x() as f32,
                        event.offset_y() as f32,
                    ));
                    let position = position.transform(&axes.space_transformer());
                    let position = position.transform(&axis.space_transformer());

                    let (axis_start, axis_end) = axis.axis_line_range();
                    position
                        .y
                        .inv_lerp(axis_start.y, axis_end.y)
                        .clamp(0.0, 1.0)
                };

                selection.set_control_point_x(*control_point_idx, axis_value);

                let mut curve_builder = curve_builder.clone();
                curve_builder.insert_selection(selection.clone(), *selection_idx);

                let datums_range = axis.visible_data_range_normalized().into();
                axis.borrow_selection_curve_mut(*active_label_idx)
                    .set_curve(curve_builder.build(datums_range, *easing_type));
                *axis.borrow_selection_curve_builder_mut(*active_label_idx) = curve_builder;
            }
            SelectAxisCP::DraggedSymmetric {
                axis,
                selection_idx,
                lower_x,
                upper_x,
                extending_start,
                control_point_idx_1,
                control_point_idx_2,
                active_label_idx,
                easing_type,
                selection,
                curve_builder,
            } => {
                let axis_value = {
                    let axes = axis.axes();
                    let axes = axes.borrow();
                    let position = Position::<ScreenSpace>::new((
                        event.offset_x() as f32,
                        event.offset_y() as f32,
                    ));
                    let position = position.transform(&axes.space_transformer());
                    let position = position.transform(&axis.space_transformer());

                    let (axis_start, axis_end) = axis.axis_line_range();
                    position
                        .y
                        .inv_lerp(axis_start.y, axis_end.y)
                        .clamp(0.0, 1.0)
                };

                let (lower, upper) = if *extending_start {
                    let offset = axis_value - *lower_x;
                    (axis_value, *upper_x - offset)
                } else {
                    let offset = axis_value - *upper_x;
                    (*lower_x - offset, axis_value)
                };

                selection.set_control_point_x(*control_point_idx_1, lower);
                selection.set_control_point_x(*control_point_idx_2, upper);

                let mut curve_builder = curve_builder.clone();
                curve_builder.insert_selection(selection.clone(), *selection_idx);

                let datums_range = axis.visible_data_range_normalized().into();
                axis.borrow_selection_curve_mut(*active_label_idx)
                    .set_curve(curve_builder.build(datums_range, *easing_type));
                *axis.borrow_selection_curve_builder_mut(*active_label_idx) = curve_builder;
            }
            SelectAxisCP::Undefined => unreachable!(),
        }

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        match self {
            SelectAxisCP::Selected {
                axis,
                selection_idx,
                control_point_idx,
                active_label_idx,
                easing_type,
                mut selection,
                mut curve_builder,
            } => {
                if selection.num_control_points() != 2 {
                    selection.remove_control_point(control_point_idx);
                    curve_builder.insert_selection(selection, selection_idx);
                }

                let datums_range = axis.visible_data_range_normalized().into();
                axis.borrow_selection_curve_mut(active_label_idx)
                    .set_curve(curve_builder.build(datums_range, easing_type));
                *axis.borrow_selection_curve_builder_mut(active_label_idx) = curve_builder;
            }
            SelectAxisCP::DraggedSingle {
                axis,
                selection_idx,
                active_label_idx,
                easing_type,
                selection,
                mut curve_builder,
                ..
            } => {
                curve_builder.insert_selection(selection, selection_idx);
                let datums_range = axis.visible_data_range_normalized().into();
                axis.borrow_selection_curve_mut(active_label_idx)
                    .set_curve(curve_builder.build(datums_range, easing_type));
                *axis.borrow_selection_curve_builder_mut(active_label_idx) = curve_builder;
            }
            SelectAxisCP::DraggedSymmetric {
                axis,
                selection_idx,
                active_label_idx,
                easing_type,
                selection,
                mut curve_builder,
                ..
            } => {
                curve_builder.insert_selection(selection, selection_idx);
                let datums_range = axis.visible_data_range_normalized().into();
                axis.borrow_selection_curve_mut(active_label_idx)
                    .set_curve(curve_builder.build(datums_range, easing_type));
                *axis.borrow_selection_curve_builder_mut(active_label_idx) = curve_builder;
            }
            SelectAxisCP::Undefined => unreachable!(),
        }

        Event::SELECTIONS_CHANGE
    }
}

#[derive(Debug)]
struct SelectCurveCP {
    axis: Rc<Axis>,
    selection_idx: usize,
    control_point_idx: usize,
    active_label_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl SelectCurveCP {
    fn new(
        axis: Rc<Axis>,
        selection_idx: usize,
        control_point_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        let mut curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();
        let selection = curve_builder.remove_selection(selection_idx);

        Self {
            axis,
            selection_idx,
            control_point_idx,
            active_label_idx,
            easing_type,
            selection,
            curve_builder,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        if event.movement_x() == 0 && event.movement_y() == 0 {
            return Event::NONE;
        }

        let (curve_value, axis_value) = {
            let axes = self.axis.axes();
            let axes = axes.borrow();
            let position =
                Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));
            let position = position.transform(&axes.space_transformer());
            let position = position.transform(&self.axis.space_transformer());

            let max_offset = self.axis.curve_offset_at_curve_value(1.0);
            let min_offset = self.axis.curve_offset_at_curve_value(0.0);
            let axis_center = self.axis.label_position().x;
            let min_curve_position_x = axis_center + min_offset.x;
            let max_curve_position_x = axis_center + max_offset.x;
            let curve_value = position
                .x
                .inv_lerp(min_curve_position_x, max_curve_position_x);

            let (axis_start, axis_end) = self.axis.axis_line_range();
            let axis_value = position
                .y
                .inv_lerp(axis_start.y, axis_end.y)
                .clamp(0.0, 1.0);

            (curve_value, axis_value)
        };

        self.selection
            .set_control_point_x(self.control_point_idx, axis_value);
        self.selection
            .set_control_point_y(self.control_point_idx, curve_value);

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.insert_selection(self.selection.clone(), self.selection_idx);

        let datums_range = self.axis.visible_data_range_normalized().into();
        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        let selection = self.selection;
        let mut curve_builder = self.curve_builder;
        let datums_range = self.axis.visible_data_range_normalized().into();
        curve_builder.insert_selection(selection, self.selection_idx);

        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }
}
