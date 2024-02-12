use std::collections::BTreeSet;

use crate::{
    lerp::InverseLerp,
    spline::{Spline, SplineSegment},
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SelectionCurve {
    range: [f32; 2],
    is_dirty: bool,
    spline: Spline,
}

impl SelectionCurve {
    pub fn new(range: [f32; 2]) -> Self {
        if range[0] >= range[1] {
            panic!("invalid selection curve range")
        }

        let mut spline = Spline::new(range);
        spline.insert_segment(SplineSegment::new_constant(1.0, range, None));

        Self {
            range,
            is_dirty: true,
            spline,
        }
    }

    pub fn set_curve(&mut self, spline: Option<Spline>) {
        match spline {
            Some(mut spline) => {
                spline.set_range(self.range);
                self.spline = spline;
            }
            None => self.spline.clear(1.0),
        }

        self.is_dirty = true;
    }

    pub fn get_changed_curve(&mut self) -> Option<&Spline> {
        let dirty = self.is_dirty;
        self.is_dirty = false;

        if dirty {
            Some(&self.spline)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct SelectionCurveBuilder {
    selections: Vec<Selection>,
    selection_infos: Vec<SelectionInfo>,
    selection_groups: Vec<SelectionGroup>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct SelectionSegmentInfo {
    pub rank: usize,
    pub range: [f32; 2],
}

impl SelectionCurveBuilder {
    pub fn new() -> Self {
        Self {
            selections: Vec::new(),
            selection_infos: Vec::new(),
            selection_groups: Vec::new(),
        }
    }

    pub fn remove_group(&mut self, group_idx: usize) {
        let group = &self.selection_groups[group_idx];
        for &selection_idx in group.selections.iter().rev() {
            self.selections.remove(selection_idx);
        }
        self.rebuild_selection_infos();
    }

    pub fn offset_group(&mut self, group_idx: usize, offset: f32) {
        let group = &self.selection_groups[group_idx];
        for &selection_idx in &group.selections {
            self.selections[selection_idx].offset(offset);
        }
        self.rebuild_selection_infos();
    }

    pub fn add_selection(&mut self, selection: Selection) {
        self.selections.push(selection);
        self.rebuild_selection_infos();
    }

    pub fn insert_selection(&mut self, selection: Selection, index: usize) {
        self.selections.insert(index, selection);
        self.rebuild_selection_infos();
    }

    pub fn remove_selection(&mut self, index: usize) -> Selection {
        let selection = self.selections.remove(index);
        self.rebuild_selection_infos();
        selection
    }

    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn get_selection_control_points(&self) -> Box<[(usize, Vec<f32>)]> {
        let mut control_points = Vec::new();
        for (info, selection) in self.selection_infos.iter().zip(&self.selections) {
            let rank = info.rank;
            let mut cp = Vec::new();
            for &(x, _) in selection.control_points() {
                cp.push(x);
            }

            if !cp.is_empty() {
                control_points.push((rank, cp));
            }
        }

        control_points.into()
    }

    pub fn get_curve_control_points(&self) -> Box<[Vec<[f32; 2]>]> {
        let mut control_points = Vec::new();
        for selection in &self.selections {
            let mut cp = Vec::new();
            for &control_point in selection.control_points() {
                cp.push(control_point.into());
            }
            if !cp.is_empty() {
                control_points.push(cp);
            }
        }
        control_points.into()
    }

    pub fn get_group_containing(&self, value: f32) -> Option<usize> {
        self.selection_groups
            .iter()
            .enumerate()
            .filter(|&(_, group)| (group.range[0]..=group.range[1]).contains(&value))
            .map(|(i, _)| i)
            .next()
    }

    pub fn get_group_ranges_between(&self, [min, max]: [f32; 2]) -> Box<[[f32; 2]]> {
        let range = min..=max;
        let mut groups = Vec::new();
        for group in &self.selection_groups {
            let [start, end] = group.range;
            if !range.contains(&start) && !range.contains(&end) {
                continue;
            }

            let start = start.clamp(min, max);
            let end = end.clamp(min, max);
            groups.push([start, end]);
        }
        groups.into()
    }

    pub fn get_selection_segment_info_in_range(
        &self,
        [min, max]: [f32; 2],
    ) -> Box<[SelectionSegmentInfo]> {
        let mut segments = Vec::new();
        for info in &self.selection_infos {
            if info.range[0] > max || info.range[1] < min {
                continue;
            }

            let [start, end] = info.range;
            let range = [start.max(min), end.min(max)];
            segments.push(SelectionSegmentInfo {
                rank: info.rank,
                range,
            });
        }

        segments.into()
    }

    pub fn max_rank(&self) -> usize {
        self.selection_infos
            .iter()
            .map(|info| info.rank)
            .max()
            .unwrap_or(0)
    }

    pub fn build(&self, range: [f32; 2], easing_type: EasingType) -> Option<Spline> {
        if self.selections.is_empty() {
            return None;
        }

        let mut spline = Spline::new(range);
        for selection in &self.selections {
            for &segment in selection.to_spline_segments(range, easing_type).iter() {
                spline.insert_segment(segment)
            }
        }

        Some(spline)
    }

    fn rebuild_selection_infos(&mut self) {
        self.selection_infos.clear();
        self.selection_groups.clear();

        // Determine the groups of the selections.
        for (i, selection) in self.selections.iter().enumerate() {
            let selection_range = selection.get_selection_range();
            self.selection_infos
                .push(SelectionInfo::new(0, selection_range));
            SelectionGroup::add_selection_to_groups(&mut self.selection_groups, i, selection_range);
        }

        // Propagate the groups to the selection and compute the ranks.
        for (group_idx, group) in self.selection_groups.iter().enumerate() {
            let selections = group.selections.iter().cloned().collect::<Vec<_>>();
            for (i, &selection) in selections.iter().enumerate() {
                let info: &mut SelectionInfo = &mut self.selection_infos[selection];
                info.group = group_idx;

                let mut rank = 0;
                let [range_start, range_end] = info.range;
                let range = range_start..=range_end;
                for &other in &selections[..i] {
                    let other_info = &self.selection_infos[other];
                    let [other_range_start, other_range_end] = other_info.range;
                    let other_range = other_range_start..=other_range_end;

                    if range.contains(&other_range_start)
                        || range.contains(&other_range_end)
                        || other_range.contains(&range_start)
                        || other_range.contains(&range_end)
                    {
                        rank = rank.max(other_info.rank + 1);
                    }
                }

                self.selection_infos[selection].rank = rank;
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
struct SelectionGroup {
    range: [f32; 2],
    selections: BTreeSet<usize>,
}

impl SelectionGroup {
    fn new(range: [f32; 2], selection: usize) -> Self {
        let mut selections = BTreeSet::new();
        selections.insert(selection);

        Self { range, selections }
    }

    fn add_selection_to_groups(
        groups: &mut Vec<Self>,
        selection_idx: usize,
        selection_range: [f32; 2],
    ) {
        if groups.is_empty() {
            groups.push(Self::new(selection_range, selection_idx));
            return;
        }

        // Find the index of the first and last group, that can contain the range.
        let start_idx = match groups
            .binary_search_by(|g| g.range[1].partial_cmp(&selection_range[0]).unwrap())
        {
            Ok(i) | Err(i) => i,
        };

        let end_idx = match groups
            .binary_search_by(|g| g.range[0].partial_cmp(&selection_range[1]).unwrap())
        {
            Ok(i) | Err(i) => i,
        };

        let mut overlapping = groups.drain(start_idx..end_idx).collect::<Vec<_>>();
        let insertion_idx = match overlapping
            .binary_search_by(|g| g.range[1].partial_cmp(&selection_range[0]).unwrap())
        {
            Ok(i) | Err(i) => i,
        };
        overlapping.insert(insertion_idx, Self::new(selection_range, selection_idx));

        let mut new_groups: Vec<SelectionGroup> = vec![overlapping.remove(0)];
        for mut group in overlapping {
            let range = group.range[0]..=group.range[1];

            let last_group = new_groups.last_mut().unwrap();
            let last_group_range = last_group.range[0]..=last_group.range[1];
            if range.contains(&last_group.range[0])
                || range.contains(&last_group.range[1])
                || last_group_range.contains(&group.range[0])
                || last_group_range.contains(&group.range[1])
            {
                last_group.range[0] = last_group.range[0].min(group.range[0]);
                last_group.range[1] = last_group.range[1].max(group.range[1]);
                last_group.selections.append(&mut group.selections);
            } else {
                new_groups.push(group);
            }
        }

        groups.splice(start_idx..start_idx, new_groups);
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
struct SelectionInfo {
    rank: usize,
    group: usize,
    range: [f32; 2],
}

impl SelectionInfo {
    fn new(group: usize, range: [f32; 2]) -> Self {
        Self {
            rank: 0,
            group,
            range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Selection {
    primary_segment_idx: usize,
    control_points: Vec<(f32, f32)>,
    // segments: Vec<SelectionSegment>,
}

impl Selection {
    pub fn new(start: [f32; 2], end: [f32; 2]) -> Self {
        if start[0] > end[0] {
            panic!("invalid selection range")
        }

        if !(0.0..=1.0).contains(&start[1]) || !(0.0..=1.0).contains(&end[1]) {
            panic!("invalid probability for selection")
        }

        Self {
            primary_segment_idx: 0,
            control_points: vec![start.into(), end.into()],
        }
    }

    pub fn from_control_points(control_points: Vec<(f32, f32)>, primary_segment: usize) -> Self {
        assert!(primary_segment < control_points.len());
        Self {
            primary_segment_idx: primary_segment,
            control_points,
        }
    }

    pub fn segment_containing(&self, value: f32) -> Option<usize> {
        (0..self.num_segments())
            .find(|&i| self.lower_bound(i) <= value && self.upper_bound(i) >= value)
    }

    pub fn get_selection_range(&self) -> [f32; 2] {
        let start = self.control_point_x(0);
        let end = self.control_point_x(self.num_control_points() - 1);

        [start, end]
    }

    pub fn segment_is_primary(&self, segment_idx: usize) -> bool {
        segment_idx == self.primary_segment_idx
    }

    pub fn segment_is_point(&self, segment_idx: usize) -> bool {
        self.lower_bound(segment_idx) == self.upper_bound(segment_idx)
    }

    pub fn num_segments(&self) -> usize {
        self.control_points.len() - 1
    }

    pub fn num_control_points(&self) -> usize {
        self.control_points.len()
    }

    pub fn primary_segment_idx(&self) -> usize {
        self.primary_segment_idx
    }

    pub fn control_point(&self, control_point_idx: usize) -> (f32, f32) {
        self.control_points[control_point_idx]
    }

    pub fn control_points(&self) -> &[(f32, f32)] {
        &self.control_points
    }

    pub fn control_point_x(&self, control_point_idx: usize) -> f32 {
        let (x, _) = self.control_point(control_point_idx);
        x
    }

    pub fn set_control_point_x(&mut self, control_point_idx: usize, mut bound: f32) {
        if control_point_idx != 0 {
            bound = bound.max(self.control_point_x(control_point_idx - 1));
        }
        if control_point_idx < self.num_control_points() - 1 {
            bound = bound.min(self.control_point_x(control_point_idx + 1));
        }
        self.control_points[control_point_idx].0 = bound;
    }

    pub fn control_point_y(&self, control_point_idx: usize) -> f32 {
        let (_, y) = self.control_point(control_point_idx);
        y
    }

    pub fn set_control_point_y(&mut self, control_point_idx: usize, mut value: f32) {
        value = value.clamp(0.0, 1.0);
        self.control_points[control_point_idx].1 = value;
    }

    pub fn lower_bound(&self, segment_idx: usize) -> f32 {
        self.control_point_x(segment_idx)
    }

    pub fn set_lower_bound(&mut self, segment_idx: usize, bound: f32) {
        self.set_control_point_x(segment_idx, bound)
    }

    pub fn lower_value(&self, segment_idx: usize) -> f32 {
        self.control_point_y(segment_idx)
    }

    pub fn set_lower_value(&mut self, segment_idx: usize, value: f32) {
        self.set_control_point_y(segment_idx, value)
    }

    pub fn upper_bound(&self, segment_idx: usize) -> f32 {
        self.control_point_x(segment_idx + 1)
    }

    pub fn set_upper_bound(&mut self, segment_idx: usize, bound: f32) {
        self.set_control_point_x(segment_idx + 1, bound)
    }

    pub fn upper_value(&self, segment_idx: usize) -> f32 {
        self.control_point_y(segment_idx + 1)
    }

    pub fn set_upper_value(&mut self, segment_idx: usize, value: f32) {
        self.set_control_point_y(segment_idx + 1, value)
    }

    pub fn remove_control_point(&mut self, control_point_idx: usize) {
        if control_point_idx <= self.primary_segment_idx {
            self.primary_segment_idx -= 1;
        }
        self.control_points.remove(control_point_idx);
    }

    pub fn insert_control_point(&mut self, x: f32, direction: Direction) -> usize {
        let (idx, y) = if let Some(segment) = self.segment_containing(x) {
            if x == self.control_point_x(segment) && direction == Direction::Down {
                if segment == 0 {
                    (0, 0.0)
                } else {
                    let segment = segment - 1;
                    let y1 = self.control_point_y(segment);
                    let y2 = self.control_point_y(segment + 1);
                    let y = (y1 + y2) / 2.0;
                    (segment + 1, y)
                }
            } else if x == self.control_point_x(segment + 1) && direction == Direction::Up {
                if segment == self.num_segments() - 1 {
                    (self.num_control_points(), 0.0)
                } else {
                    let segment = segment + 1;
                    let y1 = self.control_point_y(segment);
                    let y2 = self.control_point_y(segment + 1);
                    let y = (y1 + y2) / 2.0;
                    (segment + 1, y)
                }
            } else {
                let y1 = self.control_point_y(segment);
                let y2 = self.control_point_y(segment + 1);
                let y = (y1 + y2) / 2.0;
                (segment + 1, y)
            }
        } else if x < self.get_selection_range()[0] {
            (0, 0.0)
        } else {
            (self.num_control_points(), 0.0)
        };

        if idx != self.primary_segment_idx + 1 {
            if idx <= self.primary_segment_idx {
                self.primary_segment_idx += 1;
            }
            self.control_points.insert(idx, (x, y));
        } else {
            if direction == Direction::Up {
                self.primary_segment_idx += 1;
            }
            self.control_points.insert(idx, (x, y));
        }

        idx
    }

    pub fn offset(&mut self, offset: f32) {
        for (x, _) in &mut self.control_points {
            *x += offset;
        }
    }

    pub fn to_spline_segments(
        &self,
        [min, max]: [f32; 2],
        easing_type: EasingType,
    ) -> Box<[SplineSegment]> {
        let mut segments = Vec::new();

        for (i, (cp1, cp2)) in self
            .control_points
            .iter()
            .zip(&self.control_points[1..])
            .enumerate()
        {
            let cp1 @ (lower_bound, _) = *cp1;
            let cp2 @ (upper_bound, _) = *cp2;

            if upper_bound < min {
                continue;
            } else if lower_bound > max {
                break;
            }

            if lower_bound == upper_bound {
                continue;
            }

            let t_range = [
                if lower_bound < min {
                    min.inv_lerp(lower_bound, upper_bound)
                } else {
                    0.0
                },
                if upper_bound > max {
                    max.inv_lerp(lower_bound, upper_bound)
                } else {
                    1.0
                },
            ];

            if t_range[0] == t_range[1] {
                continue;
            }

            let easing_type = if self.segment_is_primary(i) {
                EasingType::Linear
            } else {
                easing_type
            };

            match easing_type {
                EasingType::Linear => segments.push(SplineSegment::new_linear(
                    cp1.into(),
                    cp2.into(),
                    Some(t_range),
                )),
                EasingType::EaseIn => segments.push(SplineSegment::new_ease_in(
                    cp1.into(),
                    cp2.into(),
                    Some(t_range),
                )),
                EasingType::EaseOut => segments.push(SplineSegment::new_ease_out(
                    cp1.into(),
                    cp2.into(),
                    Some(t_range),
                )),
                EasingType::EaseInOut => segments.extend(Vec::from(
                    SplineSegment::new_ease_in_out(cp1.into(), cp2.into(), Some(t_range)),
                )),
            }
        }

        segments.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Up,
    Down,
}
