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

    pub fn get_selection(&self, index: usize) -> &Selection {
        &self.selections[index]
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

    pub fn get_selection_control_points(&self) -> Box<[(usize, f32)]> {
        let mut control_points = Vec::new();
        for (info, selection) in self.selection_infos.iter().zip(&self.selections) {
            let rank = info.rank;
            for segment in &selection.segments {
                match segment {
                    SelectionSegment::Primary { range, .. } => {
                        control_points.extend([(rank, range[0]), (rank, range[1])])
                    }
                    SelectionSegment::EasingLeft { end_pos, .. }
                    | SelectionSegment::EasingRight { end_pos, .. } => {
                        control_points.push((rank, *end_pos))
                    }
                }
            }
        }

        control_points.into()
    }

    pub fn get_curve_control_points(&self) -> Box<[Vec<[f32; 2]>]> {
        let mut control_points = Vec::new();
        for selection in &self.selections {
            let mut cp = Vec::new();
            for segment in &selection.segments {
                match segment {
                    SelectionSegment::Primary { range, values } => {
                        cp.push([range[0], values[0]]);
                        cp.push([range[1], values[1]])
                    }
                    SelectionSegment::EasingLeft {
                        end_pos, end_value, ..
                    }
                    | SelectionSegment::EasingRight {
                        end_pos, end_value, ..
                    } => {
                        cp.push([*end_pos, *end_value]);
                    }
                }
            }
            if !cp.is_empty() {
                control_points.push(cp);
            }
        }
        control_points.into()
    }

    pub fn get_selection_containing(&self, value: f32, rank: usize) -> Option<usize> {
        self.selection_infos
            .iter()
            .enumerate()
            .filter(|&(_, info)| {
                info.rank == rank && (info.range[0]..=info.range[1]).contains(&value)
            })
            .map(|(i, _)| i)
            .next()
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
    segments: Vec<SelectionSegment>,
}

impl Selection {
    pub fn new(start: [f32; 2], end: [f32; 2]) -> Self {
        if start[1] > end[1] {
            panic!("invalid selection range")
        }

        if !(0.0..=1.0).contains(&start[0]) || !(0.0..=1.0).contains(&end[0]) {
            panic!("invalid probability for selection")
        }

        Self {
            primary_segment_idx: 0,
            segments: vec![SelectionSegment::Primary {
                range: [start[0], end[0]],
                values: [start[1], end[1]],
            }],
        }
    }

    pub fn from_segments(segments: Vec<SelectionSegment>, primary_segment: usize) -> Self {
        Self {
            primary_segment_idx: primary_segment,
            segments,
        }
    }

    pub fn segment_containing(&self, value: f32) -> Option<usize> {
        (0..self.segments.len())
            .find(|&i| self.lower_bound(i) <= value && self.upper_bound(i) >= value)
    }

    pub fn get_selection_range(&self) -> [f32; 2] {
        let start = self.lower_bound(0);
        let end = self.upper_bound(self.segments.len() - 1);

        [start, end]
    }

    pub fn segment_is_primary(&self, segment_idx: usize) -> bool {
        segment_idx == self.primary_segment_idx
    }

    pub fn segment_is_point(&self, segment_idx: usize) -> bool {
        self.lower_bound(segment_idx) == self.upper_bound(segment_idx)
    }

    pub fn num_segments(&self) -> usize {
        self.segments.len()
    }

    pub fn primary_segment_idx(&self) -> usize {
        self.primary_segment_idx
    }

    pub fn lower_bound(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[0],
            SelectionSegment::EasingLeft { end_pos, .. } => *end_pos,
            SelectionSegment::EasingRight { .. } => match &self.segments[segment_idx - 1] {
                SelectionSegment::Primary { range, .. } => range[1],
                SelectionSegment::EasingLeft { .. } => unreachable!(),
                SelectionSegment::EasingRight { end_pos, .. } => *end_pos,
            },
        }
    }

    pub fn set_lower_bound(&mut self, segment_idx: usize, bound: f32) {
        let mut bound = bound.min(self.upper_bound(segment_idx));
        if segment_idx != 0 {
            bound = bound.max(self.lower_bound(segment_idx - 1));
        }

        match &mut self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[0] = bound,
            SelectionSegment::EasingLeft { end_pos, .. } => *end_pos = bound,
            SelectionSegment::EasingRight { .. } => match &mut self.segments[segment_idx - 1] {
                SelectionSegment::Primary { range, .. } => range[1] = bound,
                SelectionSegment::EasingLeft { .. } => unreachable!(),
                SelectionSegment::EasingRight { end_pos, .. } => *end_pos = bound,
            },
        }
    }

    pub fn lower_value(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { values, .. } => values[0],
            SelectionSegment::EasingLeft { end_value, .. } => *end_value,
            SelectionSegment::EasingRight { .. } => match &self.segments[segment_idx - 1] {
                SelectionSegment::Primary { values, .. } => values[1],
                SelectionSegment::EasingLeft { .. } => unreachable!(),
                SelectionSegment::EasingRight { end_value, .. } => *end_value,
            },
        }
    }

    pub fn set_lower_value(&mut self, segment_idx: usize, value: f32) {
        let value = value.clamp(0.0, 1.0);
        match &mut self.segments[segment_idx] {
            SelectionSegment::Primary { values, .. } => values[0] = value,
            SelectionSegment::EasingLeft { end_value, .. } => *end_value = value,
            SelectionSegment::EasingRight { .. } => match &mut self.segments[segment_idx - 1] {
                SelectionSegment::Primary { values, .. } => values[1] = value,
                SelectionSegment::EasingLeft { .. } => unreachable!(),
                SelectionSegment::EasingRight { end_value, .. } => *end_value = value,
            },
        }
    }

    pub fn upper_bound(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[1],
            SelectionSegment::EasingLeft { .. } => match &self.segments[segment_idx + 1] {
                SelectionSegment::Primary { range, .. } => range[0],
                SelectionSegment::EasingLeft { end_pos, .. } => *end_pos,
                SelectionSegment::EasingRight { .. } => unreachable!(),
            },
            SelectionSegment::EasingRight { end_pos, .. } => *end_pos,
        }
    }

    pub fn set_upper_bound(&mut self, segment_idx: usize, bound: f32) {
        let mut bound = bound.max(self.lower_bound(segment_idx));
        if segment_idx != self.segments.len() - 1 {
            bound = bound.min(self.upper_bound(segment_idx + 1));
        }

        match &mut self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[1] = bound,
            SelectionSegment::EasingLeft { .. } => match &mut self.segments[segment_idx + 1] {
                SelectionSegment::Primary { range, .. } => range[0] = bound,
                SelectionSegment::EasingLeft { end_pos, .. } => *end_pos = bound,
                SelectionSegment::EasingRight { .. } => unreachable!(),
            },
            SelectionSegment::EasingRight { end_pos, .. } => *end_pos = bound,
        }
    }

    pub fn upper_value(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { values, .. } => values[1],
            SelectionSegment::EasingLeft { .. } => match &self.segments[segment_idx + 1] {
                SelectionSegment::Primary { values, .. } => values[0],
                SelectionSegment::EasingLeft { end_value, .. } => *end_value,
                SelectionSegment::EasingRight { .. } => unreachable!(),
            },
            SelectionSegment::EasingRight { end_value, .. } => *end_value,
        }
    }

    pub fn set_upper_value(&mut self, segment_idx: usize, value: f32) {
        let value = value.clamp(0.0, 1.0);
        match &mut self.segments[segment_idx] {
            SelectionSegment::Primary { values, .. } => values[1] = value,
            SelectionSegment::EasingLeft { .. } => match &mut self.segments[segment_idx + 1] {
                SelectionSegment::Primary { values, .. } => values[0] = value,
                SelectionSegment::EasingLeft { end_value, .. } => *end_value = value,
                SelectionSegment::EasingRight { .. } => unreachable!(),
            },
            SelectionSegment::EasingRight { end_value, .. } => *end_value = value,
        }
    }

    pub fn remove_segment(&mut self, segment_idx: usize) {
        assert_ne!(
            segment_idx, self.primary_segment_idx,
            "can not remove primary segment"
        );

        if segment_idx < self.primary_segment_idx {
            self.primary_segment_idx -= 1;
        }
        self.segments.remove(segment_idx);
    }

    pub fn insert_segment(&mut self, segment_idx: usize) {
        if segment_idx <= self.primary_segment_idx {
            let end_pos = self.lower_bound(segment_idx);
            let end_value = if segment_idx == 0 {
                0.0
            } else {
                (self.lower_value(segment_idx) + self.lower_value(segment_idx - 1)) / 2.0
            };

            self.primary_segment_idx += 1;
            self.segments.insert(
                segment_idx,
                SelectionSegment::EasingLeft { end_pos, end_value },
            );
        } else {
            let end_pos = self.upper_bound(segment_idx - 1);
            let end_value = if segment_idx == self.segments.len() {
                0.0
            } else {
                (self.upper_value(segment_idx) + self.upper_value(segment_idx - 1)) / 2.0
            };

            self.segments.insert(
                segment_idx,
                SelectionSegment::EasingRight { end_pos, end_value },
            );
        }
    }

    pub fn offset(&mut self, offset: f32) {
        for segment in &mut self.segments {
            match segment {
                SelectionSegment::Primary {
                    range: [start_pos, end_pos],
                    ..
                } => {
                    *start_pos += offset;
                    *end_pos += offset;
                }
                SelectionSegment::EasingLeft { end_pos, .. } => *end_pos += offset,
                SelectionSegment::EasingRight { end_pos, .. } => *end_pos += offset,
            }
        }
    }

    pub fn to_spline_segments(
        &self,
        [min, max]: [f32; 2],
        easing_type: EasingType,
    ) -> Box<[SplineSegment]> {
        let mut segments = Vec::new();

        for i in 0..self.segments.len() {
            if self.upper_bound(i) < min {
                continue;
            } else if self.lower_bound(i) > max {
                break;
            }

            let start @ [l, _] = [self.lower_bound(i), self.lower_value(i)];
            let end @ [u, _] = [self.upper_bound(i), self.upper_value(i)];

            if l == u {
                continue;
            }

            let t_range = [
                if l < min { min.inv_lerp(l, u) } else { 0.0 },
                if u > max { max.inv_lerp(l, u) } else { 1.0 },
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
                EasingType::Linear => {
                    segments.push(SplineSegment::new_linear(start, end, Some(t_range)))
                }
                EasingType::EaseIn => {
                    segments.push(SplineSegment::new_ease_in(start, end, Some(t_range)))
                }
                EasingType::EaseOut => {
                    segments.push(SplineSegment::new_ease_out(start, end, Some(t_range)))
                }
                EasingType::EaseInOut => segments.extend(Vec::from(
                    SplineSegment::new_ease_in_out(start, end, Some(t_range)),
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SelectionSegment {
    Primary { range: [f32; 2], values: [f32; 2] },
    EasingLeft { end_pos: f32, end_value: f32 },
    EasingRight { end_pos: f32, end_value: f32 },
}
