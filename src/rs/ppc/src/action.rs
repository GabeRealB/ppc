use std::rc::Rc;

use web_sys::PointerEvent;

use crate::{
    axis::Axis,
    coordinates::{Offset, Position, ScreenSpace},
    event::Event,
    lerp::InverseLerp,
    selection::{EasingType, Selection, SelectionCurveBuilder},
};

#[derive(Debug)]
pub struct Action {
    inner: ActionInner,
}

#[derive(Debug)]
enum ActionInner {
    MoveAxis(MoveAxisAction),
    CreateSelection(CreateSelectionAction),
    SelectSelection(SelectSelectionAction),
    SelectSelectionControlPoint(SelectSelectionControlPointAction),
    SelectCurveControlPoint(SelectCurveControlPointAction),
}

impl Action {
    pub fn new_move_axis_action(
        axis: Rc<Axis>,
        event: PointerEvent,
        active_label_idx: Option<usize>,
    ) -> Self {
        Self {
            inner: ActionInner::MoveAxis(MoveAxisAction::new(axis, event, active_label_idx)),
        }
    }

    pub fn new_create_selection_action(
        axis: Rc<Axis>,
        event: PointerEvent,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::CreateSelection(CreateSelectionAction::new(
                axis,
                event,
                active_label_idx,
                easing_type,
            )),
        }
    }

    pub fn new_select_selection_action(
        axis: Rc<Axis>,
        selection_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::SelectSelection(SelectSelectionAction::new(
                axis,
                selection_idx,
                active_label_idx,
                easing_type,
            )),
        }
    }

    pub fn new_select_selection_control_point_action(
        axis: Rc<Axis>,
        selection_idx: usize,
        segment_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        event: PointerEvent,
    ) -> Self {
        Self {
            inner: ActionInner::SelectSelectionControlPoint(
                SelectSelectionControlPointAction::new(
                    axis,
                    selection_idx,
                    segment_idx,
                    active_label_idx,
                    easing_type,
                    event,
                ),
            ),
        }
    }

    pub fn new_select_curve_control_point_action(
        axis: Rc<Axis>,
        selection_idx: usize,
        segment_idx: usize,
        active_label_idx: usize,
        is_upper: bool,
        easing_type: EasingType,
    ) -> Self {
        Self {
            inner: ActionInner::SelectCurveControlPoint(SelectCurveControlPointAction::new(
                axis,
                selection_idx,
                segment_idx,
                active_label_idx,
                is_upper,
                easing_type,
            )),
        }
    }

    pub fn update(&mut self, event: PointerEvent) -> Event {
        match &mut self.inner {
            ActionInner::MoveAxis(e) => e.update(event),
            ActionInner::CreateSelection(e) => e.update(event),
            ActionInner::SelectSelection(e) => e.update(event),
            ActionInner::SelectSelectionControlPoint(e) => e.update(event),
            ActionInner::SelectCurveControlPoint(e) => e.update(event),
        }
    }

    pub fn finish(self) -> Event {
        match self.inner {
            ActionInner::MoveAxis(e) => e.finish(),
            ActionInner::CreateSelection(e) => e.finish(),
            ActionInner::SelectSelection(e) => e.finish(),
            ActionInner::SelectSelectionControlPoint(e) => e.finish(),
            ActionInner::SelectCurveControlPoint(e) => e.finish(),
        }
    }
}

#[derive(Debug)]
struct MoveAxisAction {
    axis: Rc<Axis>,
    moved: bool,
    active_label_idx: Option<usize>,
    start_position: Position<ScreenSpace>,
}

impl MoveAxisAction {
    fn new(axis: Rc<Axis>, event: PointerEvent, active_label_idx: Option<usize>) -> Self {
        let position =
            Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));

        Self {
            axis,
            moved: false,
            active_label_idx,
            start_position: position,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        let offset = {
            let position =
                Position::<ScreenSpace>::new((event.offset_x() as f32, event.offset_y() as f32));

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

        if !self.moved {
            match self.axis.state() {
                crate::axis::AxisState::Collapsed => self.axis.expand(),
                crate::axis::AxisState::Expanded => self.axis.collapse(),
                crate::axis::AxisState::Hidden => {}
            }

            Event::AXIS_STATE_CHANGE | Event::AXIS_POSITION_CHANGE
        } else {
            Event::AXIS_POSITION_CHANGE
        }
    }
}

#[derive(Debug)]
struct CreateSelectionAction {
    axis: Rc<Axis>,
    start_axis_value: f32,
    active_label_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl CreateSelectionAction {
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
            self.selection.set_lower_bound(0, axis_value);
        } else {
            self.selection.set_upper_bound(0, axis_value);
        }

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.add_selection(self.selection.clone());

        let datums_range = self.axis.visible_datums_range_normalized().into();
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
        let datums_range = self.axis.visible_datums_range_normalized().into();

        if !self.selection.segment_is_point(0) {
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
struct SelectSelectionAction {
    axis: Rc<Axis>,
    moved: bool,
    active_label_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl SelectSelectionAction {
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
            let offset =
                Offset::<ScreenSpace>::new((event.movement_x() as f32, event.movement_y() as f32));
            let offset = offset.transform(&axes.space_transformer());
            let offset = offset.transform(&self.axis.space_transformer());

            let (axis_start, axis_end) = self.axis.axis_line_range();
            -offset.y / (axis_end.y - axis_start.y).abs()
        };
        self.selection.offset(offset);

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.add_selection(self.selection.clone());

        let datums_range = self.axis.visible_datums_range_normalized().into();
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
        let datums_range = self.axis.visible_datums_range_normalized().into();

        // If we moved the mouse we do add the modified selection.
        if self.moved {
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
struct SelectSelectionControlPointAction {
    axis: Rc<Axis>,
    moved: bool,
    lower_bound: bool,
    active_label_idx: usize,
    segment_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl SelectSelectionControlPointAction {
    fn new(
        axis: Rc<Axis>,
        selection_idx: usize,
        mut segment_idx: usize,
        active_label_idx: usize,
        easing_type: EasingType,
        event: PointerEvent,
    ) -> Self {
        let mut curve_builder = axis
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

        let mut selection = curve_builder.remove_selection(selection_idx);
        let middle =
            (selection.lower_bound(segment_idx) + selection.upper_bound(segment_idx)) / 2.0;
        let lower_bound = axis_value < middle;

        if event.ctrl_key() {
            if !lower_bound {
                segment_idx += 1;
            }
            selection.insert_segment(segment_idx);
        }

        Self {
            axis,
            moved: false,
            lower_bound,
            active_label_idx,
            segment_idx,
            easing_type,
            selection,
            curve_builder,
        }
    }

    fn update(&mut self, event: PointerEvent) -> Event {
        if event.movement_y() == 0 {
            return Event::NONE;
        }
        self.moved = true;

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

        if self.lower_bound {
            self.selection.set_lower_bound(self.segment_idx, axis_value);
        } else {
            self.selection.set_upper_bound(self.segment_idx, axis_value);
        }

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.add_selection(self.selection.clone());

        let datums_range = self.axis.visible_datums_range_normalized().into();
        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        let mut selection = self.selection;
        let mut curve_builder = self.curve_builder;
        let datums_range = self.axis.visible_datums_range_normalized().into();

        let delete_segment = !self.moved || selection.segment_is_point(self.segment_idx);
        if delete_segment {
            if !selection.segment_is_primary(self.segment_idx) {
                selection.remove_segment(self.segment_idx);
                curve_builder.add_selection(selection);
            }
        } else {
            curve_builder.add_selection(selection);
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
struct SelectCurveControlPointAction {
    axis: Rc<Axis>,
    lower_bound: bool,
    active_label_idx: usize,
    segment_idx: usize,
    selection_idx: usize,
    easing_type: EasingType,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl SelectCurveControlPointAction {
    fn new(
        axis: Rc<Axis>,
        selection_idx: usize,
        segment_idx: usize,
        active_label_idx: usize,
        is_upper: bool,
        easing_type: EasingType,
    ) -> Self {
        let mut curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();
        let selection = curve_builder.remove_selection(selection_idx);

        Self {
            axis,
            lower_bound: !is_upper,
            active_label_idx,
            segment_idx,
            selection_idx,
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
            let axis_value = position.y.inv_lerp(axis_start.y, axis_end.y);

            (curve_value, axis_value)
        };

        if self.lower_bound {
            self.selection.set_lower_bound(self.segment_idx, axis_value);
            self.selection
                .set_lower_value(self.segment_idx, curve_value);
        } else {
            self.selection.set_upper_bound(self.segment_idx, axis_value);
            self.selection
                .set_upper_value(self.segment_idx, curve_value);
        }

        let mut curve_builder = self.curve_builder.clone();
        curve_builder.insert_selection(self.selection.clone(), self.selection_idx);

        let datums_range = self.axis.visible_datums_range_normalized().into();
        self.axis
            .borrow_selection_curve_mut(self.active_label_idx)
            .set_curve(curve_builder.build(datums_range, self.easing_type));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }

    fn finish(self) -> Event {
        let mut selection = self.selection;
        let mut curve_builder = self.curve_builder;
        let datums_range = self.axis.visible_datums_range_normalized().into();
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
