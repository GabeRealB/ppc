use std::collections::VecDeque;

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
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum SelectionSegmentInfo {
    Visible { rank: usize, range: [f32; 2] },
    Invisible { rank: usize, range: [f32; 2] },
}

impl SelectionCurveBuilder {
    pub fn new() -> Self {
        Self {
            selections: Vec::new(),
            selection_infos: Vec::new(),
        }
    }

    pub fn add_selection(&mut self, selection: Selection) {
        self.selections.push(selection);
        self.rebuild_selection_infos();
    }

    pub fn remove_selection(&mut self, index: usize) -> Selection {
        let selection = self.selections.remove(index);
        self.rebuild_selection_infos();
        selection
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

    pub fn get_visible_selection_ranges_in_range(&self, [min, max]: [f32; 2]) -> Box<[[f32; 2]]> {
        let mut ranges = Vec::new();
        for info in &self.selection_infos {
            if info.range[0] > max || info.range[1] < min {
                continue;
            }

            for range @ [start, end] in &info.visible_ranges {
                let segment = [start.max(min), end.min(max)];
                if segment[0] > max || segment[1] < min {
                    continue;
                }

                if ranges.is_empty() {
                    ranges.push(segment);
                    continue;
                }

                let start_idx =
                    match ranges.binary_search_by(|[min, _]| min.partial_cmp(start).unwrap()) {
                        Ok(i) => i,
                        Err(i) => i,
                    };

                let end_idx =
                    match ranges.binary_search_by(|[_, max]| max.partial_cmp(end).unwrap()) {
                        Ok(i) => i,
                        Err(i) => i,
                    };

                // When start end end are inverted it indicates that the new segment lies
                // within the segment at end idx.
                if start_idx > end_idx {
                    continue;
                }

                // Remove all segments in the range [start, end), as we definitely know
                // that the new segment covers them completely.
                ranges.drain(start_idx..end_idx);

                // Having removed all segments that are completely covered, we only have
                // to adapt the segments directly preceding and following our new segment.
                let previous_idx = start_idx.wrapping_sub(1);
                let next_idx = start_idx;

                let appended_to_previous =
                    if let Some([previous_start, previous_end]) = ranges.get_mut(previous_idx) {
                        if (*previous_start..=*previous_end).contains(&segment[0]) {
                            *previous_end = segment[1];
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                let appended_to_next =
                    if let Some([previous_start, previous_end]) = ranges.get_mut(next_idx) {
                        if (*previous_start..=*previous_end).contains(&segment[1]) {
                            *previous_start = segment[0];
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                if !(appended_to_previous && appended_to_next) {
                    ranges.insert(start_idx, segment);
                }
            }
        }

        ranges.into()
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

            for [start, end] in &info.visible_ranges {
                let segment = [start.max(min), end.min(max)];
                if segment[0] > max || segment[1] < min {
                    continue;
                }

                segments.push(SelectionSegmentInfo::Visible {
                    rank: info.rank,
                    range: segment,
                });
            }

            for [start, end] in &info.invisible_ranges {
                let segment = [start.max(min), end.min(max)];
                if segment[0] > max || segment[1] < min {
                    continue;
                }

                segments.push(SelectionSegmentInfo::Invisible {
                    rank: info.rank,
                    range: segment,
                });
            }
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

    pub fn build(&self, range: [f32; 2]) -> Option<Spline> {
        if self.selections.is_empty() {
            return None;
        }

        let mut spline = Spline::new(range);
        for selection in &self.selections {
            for &segment in selection.to_spline_segments(range).iter() {
                spline.insert_segment(segment)
            }
        }

        Some(spline)
    }

    fn rebuild_selection_infos(&mut self) {
        self.selection_infos.clear();

        // Determine which selections are covered by other selections.
        for (i, selection) in self.selections.iter().enumerate() {
            let selection_range = selection.get_selection_range();

            // Selections that appear later in the list "cover" prior selections.
            for previous_selection_info in &mut self.selection_infos {
                let r1 = selection_range[0]..=selection_range[1];
                let r2 = previous_selection_info.range[0]..=previous_selection_info.range[1];

                if r1.contains(&previous_selection_info.range[0])
                    || r1.contains(&previous_selection_info.range[1])
                    || r2.contains(&selection_range[0])
                    || r2.contains(&selection_range[1])
                {
                    previous_selection_info.add_coverer(i);
                }
            }

            self.selection_infos
                .push(SelectionInfo::new(selection_range));
        }

        // Determine the ranges that are and are not covered for each selection.
        for i in (0..self.selection_infos.len()).rev() {
            let mut info = std::mem::take(&mut self.selection_infos[i]);

            // Iterate each selection covering this and "chip away" from the
            // visible range.
            let covered_by = std::mem::take(&mut info.covered_by);
            for &coverer_idx in &covered_by {
                let coverer = &self.selection_infos[coverer_idx];
                let covered_range = coverer.range;
                info.remove_visible_range(covered_range);
            }
            info.covered_by = covered_by;

            // Compute the invisible ranges.
            info.compute_invisible_ranges();
            self.selection_infos[i] = info;
        }

        // Determine the rank of each selection.
        for i in (0..self.selection_infos.len()).rev() {
            let mut info = std::mem::take(&mut self.selection_infos[i]);
            info.rank = info
                .covered_by
                .iter()
                .copied()
                .map(|coverer| self.selection_infos[coverer].rank + 1)
                .max()
                .unwrap_or(0);
            self.selection_infos[i] = info;
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
struct SelectionInfo {
    rank: usize,
    range: [f32; 2],
    covered_by: VecDeque<usize>,
    visible_ranges: Vec<[f32; 2]>,
    invisible_ranges: Vec<[f32; 2]>,
}

impl SelectionInfo {
    fn new(range: [f32; 2]) -> Self {
        Self {
            rank: 0,
            range,
            covered_by: VecDeque::new(),
            visible_ranges: vec![range],
            invisible_ranges: vec![],
        }
    }

    fn add_coverer(&mut self, coverer: usize) {
        self.covered_by.push_front(coverer)
    }

    fn remove_visible_range(&mut self, range: [f32; 2]) {
        // If there are no visible ranges left we can simply ignore it.
        if self.visible_ranges.is_empty() {
            return;
        }

        // Clamp the range to the remaining visible range of the selection.
        let min = self.visible_ranges.first().unwrap()[0];
        let max = self.visible_ranges.last().unwrap()[1];
        let range = [range[0].clamp(min, max), range[1].clamp(min, max)];
        if range[0] == range[1] {
            return;
        }

        let start_idx = match self
            .visible_ranges
            .binary_search_by(|s| s[0].partial_cmp(&range[0]).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };

        let end_idx = match self
            .visible_ranges
            .binary_search_by(|s| s[1].partial_cmp(&range[1]).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };

        // When start end end are inverted it indicates that the segment lies
        // within the segment at end idx.
        if start_idx > end_idx {
            let [l, h] = self.visible_ranges[end_idx];
            let left = [l, range[0]];
            let right = [range[1], h];

            if left[0] == left[1] {
                self.visible_ranges[end_idx] = range;

                if right[0] != right[1] {
                    self.visible_ranges.insert(end_idx + 1, right);
                }
            } else {
                self.visible_ranges[end_idx] = left;
                self.visible_ranges.insert(end_idx + 1, range);

                if right[0] != right[1] {
                    self.visible_ranges.insert(end_idx + 2, right);
                }
            }

            return;
        }

        // Remove all segments in the range [start, end), as we definitely know
        // that the segment covers them completely.
        self.visible_ranges.drain(start_idx..end_idx);

        // Having removed all segments that are completely covered, we only have
        // to adapt the segments directly preceding and following the segment.
        let previous_idx = start_idx.wrapping_sub(1);
        let next_idx = start_idx;

        if let Some(seg) = self.visible_ranges.get_mut(next_idx) {
            if seg[0] < range[1] {
                *seg = [range[1], seg[1]];
            }

            if seg[0] == seg[1] {
                self.visible_ranges.remove(next_idx);
            }
        }

        if let Some(seg) = self.visible_ranges.get_mut(previous_idx) {
            if seg[1] > range[0] {
                *seg = [seg[0], range[0]];
            }

            if seg[0] == seg[1] {
                self.visible_ranges.remove(previous_idx);
            }
        }
    }

    fn compute_invisible_ranges(&mut self) {
        if self.visible_ranges.is_empty() {
            self.invisible_ranges.push(self.range);
        } else {
            let mut last_end = self.range[0];

            // If we detect a hole, we close it up.
            for &[start, end] in &self.visible_ranges {
                if start > last_end {
                    self.invisible_ranges.push([last_end, start]);
                }
                last_end = end;
            }

            // Make sure that we covered the entire range.
            if last_end < self.range[1] {
                self.invisible_ranges.push([last_end, self.range[1]]);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Selection {
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
            segments: vec![SelectionSegment::Primary {
                range: [start[0], end[0]],
                values: [start[1], end[1]],
            }],
        }
    }

    pub fn segment_containing(&self, value: f32) -> Option<usize> {
        (0..self.segments.len())
            .find(|&i| self.lower_bound(i) <= value && self.upper_bound(i) >= value)
    }

    pub fn get_selection_range(&self) -> [f32; 2] {
        let start = match self.segments.first() {
            Some(SelectionSegment::Primary { range, .. }) => range[0],
            Some(SelectionSegment::FadingLeft { end_pos, .. }) => *end_pos,
            _ => unreachable!(),
        };

        let end = match self.segments.last() {
            Some(SelectionSegment::Primary { range, .. }) => range[1],
            Some(SelectionSegment::FadingRight { end_pos, .. }) => *end_pos,
            _ => unreachable!(),
        };

        [start, end]
    }

    pub fn segment_is_point(&self, segment_idx: usize) -> bool {
        self.lower_bound(segment_idx) == self.upper_bound(segment_idx)
    }

    pub fn lower_bound(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[0],
            SelectionSegment::FadingLeft { end_pos, .. } => *end_pos,
            SelectionSegment::FadingRight { .. } => match &self.segments[segment_idx - 1] {
                SelectionSegment::Primary { range, .. } => range[1],
                SelectionSegment::FadingLeft { .. } => unreachable!(),
                SelectionSegment::FadingRight { end_pos, .. } => *end_pos,
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
            SelectionSegment::FadingLeft { end_pos, .. } => *end_pos = bound,
            SelectionSegment::FadingRight { .. } => match &mut self.segments[segment_idx - 1] {
                SelectionSegment::Primary { range, .. } => range[1] = bound,
                SelectionSegment::FadingLeft { .. } => unreachable!(),
                SelectionSegment::FadingRight { end_pos, .. } => *end_pos = bound,
            },
        }
    }

    pub fn lower_value(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { values, .. } => values[0],
            SelectionSegment::FadingLeft { end_value, .. } => *end_value,
            SelectionSegment::FadingRight { .. } => match &self.segments[segment_idx - 1] {
                SelectionSegment::Primary { values, .. } => values[1],
                SelectionSegment::FadingLeft { .. } => unreachable!(),
                SelectionSegment::FadingRight { end_value, .. } => *end_value,
            },
        }
    }

    pub fn upper_bound(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[1],
            SelectionSegment::FadingLeft { .. } => match &self.segments[segment_idx + 1] {
                SelectionSegment::Primary { range, .. } => range[0],
                SelectionSegment::FadingLeft { end_pos, .. } => *end_pos,
                SelectionSegment::FadingRight { .. } => unreachable!(),
            },
            SelectionSegment::FadingRight { end_pos, .. } => *end_pos,
        }
    }

    pub fn set_upper_bound(&mut self, segment_idx: usize, bound: f32) {
        let mut bound = bound.max(self.lower_bound(segment_idx));
        if segment_idx != self.segments.len() - 1 {
            bound = bound.max(self.upper_bound(segment_idx + 1));
        }

        match &mut self.segments[segment_idx] {
            SelectionSegment::Primary { range, .. } => range[1] = bound,
            SelectionSegment::FadingLeft { .. } => match &mut self.segments[segment_idx + 1] {
                SelectionSegment::Primary { range, .. } => range[0] = bound,
                SelectionSegment::FadingLeft { end_pos, .. } => *end_pos = bound,
                SelectionSegment::FadingRight { .. } => unreachable!(),
            },
            SelectionSegment::FadingRight { end_pos, .. } => *end_pos = bound,
        }
    }

    pub fn upper_value(&self, segment_idx: usize) -> f32 {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { values, .. } => values[1],
            SelectionSegment::FadingLeft { .. } => match &self.segments[segment_idx + 1] {
                SelectionSegment::Primary { values, .. } => values[0],
                SelectionSegment::FadingLeft { end_value, .. } => *end_value,
                SelectionSegment::FadingRight { .. } => unreachable!(),
            },
            SelectionSegment::FadingRight { end_value, .. } => *end_value,
        }
    }

    pub fn fading_type(&self, segment_idx: usize) -> FadingType {
        match &self.segments[segment_idx] {
            SelectionSegment::Primary { .. } => FadingType::Linear,
            SelectionSegment::FadingLeft { fading_type, .. } => *fading_type,
            SelectionSegment::FadingRight { fading_type, .. } => *fading_type,
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
                SelectionSegment::FadingLeft { end_pos, .. } => *end_pos += offset,
                SelectionSegment::FadingRight { end_pos, .. } => *end_pos += offset,
            }
        }
    }

    pub fn to_spline_segments(&self, [min, max]: [f32; 2]) -> Box<[SplineSegment]> {
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

            match self.fading_type(i) {
                FadingType::Linear => {
                    segments.push(SplineSegment::new_linear(start, end, Some(t_range)))
                }
                FadingType::EaseIn => {
                    segments.push(SplineSegment::new_ease_in(start, end, Some(t_range)))
                }
                FadingType::EaseOut => {
                    segments.push(SplineSegment::new_ease_out(start, end, Some(t_range)))
                }
                FadingType::EaseInOut => segments.extend(Vec::from(
                    SplineSegment::new_ease_in_out(start, end, Some(t_range)),
                )),
            }
        }

        segments.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FadingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SelectionSegment {
    Primary {
        range: [f32; 2],
        values: [f32; 2],
    },
    FadingLeft {
        end_pos: f32,
        end_value: f32,
        fading_type: FadingType,
    },
    FadingRight {
        end_pos: f32,
        end_value: f32,
        fading_type: FadingType,
    },
}
