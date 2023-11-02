use std::rc::Rc;

use web_sys::PointerEvent;

use crate::{
    axis::Axis,
    coordinates::{Offset, Position, ScreenSpace},
    event::Event,
    lerp::InverseLerp,
    selection::{Selection, SelectionCurveBuilder},
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
    ) -> Self {
        Self {
            inner: ActionInner::CreateSelection(CreateSelectionAction::new(
                axis,
                event,
                active_label_idx,
            )),
        }
    }

    pub fn new_select_selection_action(
        axis: Rc<Axis>,
        selection_idx: usize,
        active_label_idx: usize,
    ) -> Self {
        Self {
            inner: ActionInner::SelectSelection(SelectSelectionAction::new(
                axis,
                selection_idx,
                active_label_idx,
            )),
        }
    }

    pub fn update(&mut self, event: PointerEvent) -> Event {
        match &mut self.inner {
            ActionInner::MoveAxis(e) => e.update(event),
            ActionInner::CreateSelection(e) => e.update(event),
            ActionInner::SelectSelection(e) => e.update(event),
        }
    }

    pub fn finish(self) -> Event {
        match self.inner {
            ActionInner::MoveAxis(e) => e.finish(),
            ActionInner::CreateSelection(e) => e.finish(),
            ActionInner::SelectSelection(e) => e.finish(),
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
    active_label_idx: usize,
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
    start_axis_value: f32,
}

impl CreateSelectionAction {
    fn new(axis: Rc<Axis>, event: PointerEvent, active_label_idx: usize) -> Self {
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
            .set_curve(curve_builder.build(datums_range));
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
            .set_curve(curve_builder.build(datums_range));
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
    selection: Selection,
    curve_builder: SelectionCurveBuilder,
}

impl SelectSelectionAction {
    fn new(axis: Rc<Axis>, selection_idx: usize, active_label_idx: usize) -> Self {
        let mut curve_builder = axis
            .borrow_selection_curve_builder(active_label_idx)
            .clone();
        let selection = curve_builder.remove_selection(selection_idx);

        Self {
            axis,
            moved: false,
            active_label_idx,
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
            .set_curve(curve_builder.build(datums_range));
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
            .set_curve(curve_builder.build(datums_range));
        *self
            .axis
            .borrow_selection_curve_builder_mut(self.active_label_idx) = curve_builder;

        Event::SELECTIONS_CHANGE
    }
}
