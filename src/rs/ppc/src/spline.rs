use crate::lerp::{InverseLerp, Lerp};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Spline {
    range: [f32; 2],
    segments: Vec<SplineSegment>,
}

impl Spline {
    pub fn new(range: [f32; 2]) -> Self {
        if range[0] >= range[1] {
            panic!("invalid spline range")
        }

        Self {
            range,
            segments: vec![SplineSegment::new_constant(0.0, range, None)],
        }
    }

    pub fn clear(&mut self, value: f32) {
        self.segments.clear();
        self.segments
            .push(SplineSegment::new_constant(value, self.range, None));
    }

    pub fn segments(&self) -> &[SplineSegment] {
        &self.segments
    }

    pub fn set_range(&mut self, range: [f32; 2]) {
        if range[0] < self.range[0] {
            self.segments.insert(
                0,
                SplineSegment::new_constant(0.0, [range[0], self.range[0]], None),
            );
        } else if range[0] > self.range[0] {
            let segment = self.segments[0].split_at(range[0], SegmentRemovalOp::RemoveLeft);
            self.segments[0] = segment;
        }

        if range[1] < self.range[1] {
            let last_idx = self.segments.len() - 1;
            let segment = self.segments[last_idx].split_at(range[1], SegmentRemovalOp::RemoveRight);
            self.segments[last_idx] = segment;
        } else if range[1] > self.range[1] {
            self.segments.push(SplineSegment::new_constant(
                0.0,
                [self.range[1], range[1]],
                None,
            ));
        }

        self.range = range;
    }

    pub fn insert_segment(&mut self, segment: SplineSegment) {
        // If the segment lies completely out of the range of the spline, we ignore it.
        if !segment.covers_range(self.range) {
            return;
        }

        let segment = match segment.bounds {
            [start, end] if start < self.range[0] && end <= self.range[1] => {
                segment.split_at(self.range[0], SegmentRemovalOp::RemoveLeft)
            }
            [start, end] if start >= self.range[0] && end > self.range[1] => {
                segment.split_at(self.range[1], SegmentRemovalOp::RemoveRight)
            }
            [start, end] if start < self.range[0] && end > self.range[1] => segment
                .split_at(self.range[0], SegmentRemovalOp::RemoveLeft)
                .split_at(self.range[1], SegmentRemovalOp::RemoveRight),
            _ => segment,
        };

        if segment.is_empty() {
            return;
        }

        let start_idx = match self
            .segments
            .binary_search_by(|s| s.bounds[0].partial_cmp(&segment.bounds[0]).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };

        let end_idx = match self
            .segments
            .binary_search_by(|s| s.bounds[1].partial_cmp(&segment.bounds[1]).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };

        // When start end end are inverted it indicates that the new segment lies
        // within the segment at end idx.
        if start_idx > end_idx {
            let seg = self.segments[end_idx];
            let left = seg.split_at(segment.bounds[0], SegmentRemovalOp::RemoveRight);
            let right = seg.split_at(segment.bounds[1], SegmentRemovalOp::RemoveLeft);

            if left.is_empty() {
                self.segments[end_idx] = segment;

                if !right.is_empty() {
                    self.segments.insert(end_idx + 1, right);
                }
            } else {
                self.segments[end_idx] = left;
                self.segments.insert(end_idx + 1, segment);

                if !right.is_empty() {
                    self.segments.insert(end_idx + 2, right);
                }
            }

            return;
        }

        // Remove all segments in the range [start, end), as we definitely know
        // that the new segment covers them completely.
        self.segments.drain(start_idx..end_idx);

        // Having removed all segments that are completely covered, we only have
        // to adapt the segments directly preceding and following our new segment.
        let previous_idx = start_idx.wrapping_sub(1);
        let next_idx = start_idx + 1;

        self.segments.insert(start_idx, segment);

        if let Some(seg) = self.segments.get_mut(next_idx) {
            if seg.bounds[0] < segment.bounds[1] {
                *seg = seg.split_at(segment.bounds[1], SegmentRemovalOp::RemoveLeft);
            }

            if seg.is_empty() {
                self.segments.remove(next_idx);
            }
        }

        if let Some(seg) = self.segments.get_mut(previous_idx) {
            if seg.bounds[1] > segment.bounds[0] {
                *seg = seg.split_at(segment.bounds[0], SegmentRemovalOp::RemoveRight);
            }

            if seg.is_empty() {
                self.segments.remove(previous_idx);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SplineSegment {
    pub bounds: [f32; 2],
    pub t_range: [f32; 2],
    pub coefficients: [f32; 4],
}

pub enum SegmentRemovalOp {
    RemoveLeft,
    RemoveRight,
}

impl SplineSegment {
    pub fn new_constant(value: f32, range: [f32; 2], t_range: Option<[f32; 2]>) -> Self {
        Self::new_linear([range[0], value], [range[1], value], t_range)
    }

    pub fn new_linear(p0: [f32; 2], p1: [f32; 2], t_range: Option<[f32; 2]>) -> Self {
        let t_range = t_range.unwrap_or([0.0, 1.0]);
        if t_range[0] >= t_range[1] || t_range[0] < 0.0 || t_range[1] > 1.0 {
            panic!("invalid segment t range '{t_range:?}'")
        }

        // Sort the control points by the x coordinate.
        let (p0, p1) = if p0[0] < p1[0] { (p0, p1) } else { (p1, p0) };
        if p0[0] == p1[0] {
            panic!("each x value must be unique")
        }

        // Fit a polynomial of degree 1
        let a = p1[1] - p0[1];
        let b = p0[1];

        let bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };

        Self {
            bounds,
            t_range,
            coefficients: [0.0, 0.0, a, b],
        }
    }

    #[allow(dead_code)]
    pub fn new_quadratic(
        p0: [f32; 2],
        p1: [f32; 2],
        p2: [f32; 2],
        t_range: Option<[f32; 2]>,
    ) -> Self {
        let t_range = t_range.unwrap_or([0.0, 1.0]);
        if t_range[0] >= t_range[1] || t_range[0] < 0.0 || t_range[1] > 1.0 {
            panic!("invalid segment t range '{t_range:?}'")
        }

        // Sort the control points by the x coordinate.
        let mut control_points = [p0, p1, p2];
        control_points.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        let [p0, p1, p2] = control_points;

        if p0[0] == p1[0] || p1[0] == p2[0] {
            panic!("each x value must be unique")
        }

        // Fit a polynomial of degree 2
        let segment_length = p2[0] - p0[0];
        let segment_start = p0[0];

        let p1x = (p1[0] - segment_start) / segment_length;
        let p1x_2 = p1x * p1x;

        let c = p0[1];
        let b = (((p2[1] - p0[1]) * p1x_2) + p0[1] - p1[1]) / (p1x_2 - p1x);
        let a = p2[1] - b - c;

        let bounds = if t_range == [0.0, 1.0] {
            [p0[0], p2[0]]
        } else {
            [p0[0].lerp(p2[0], t_range[0]), p0[0].lerp(p2[0], t_range[1])]
        };

        Self {
            bounds,
            t_range,
            coefficients: [0.0, a, b, c],
        }
    }

    #[allow(dead_code)]
    pub fn new_cubic(
        p0: [f32; 2],
        p1: [f32; 2],
        p2: [f32; 2],
        p3: [f32; 2],
        t_range: Option<[f32; 2]>,
    ) -> Self {
        let t_range = t_range.unwrap_or([0.0, 1.0]);
        if t_range[0] >= t_range[1] || t_range[0] < 0.0 || t_range[1] > 1.0 {
            panic!("invalid segment t range '{t_range:?}'")
        }

        // Sort the control points by the x coordinate.
        let mut control_points = [p0, p1, p2, p3];
        control_points.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        let [p0, p1, p2, p3] = control_points;

        if p0[0] == p1[0] || p1[0] == p2[0] || p2[0] == p3[0] {
            panic!("each x value must be unique")
        }

        // Fit a polynomial of degree 3
        let segment_length = p3[0] - p0[0];
        let segment_start = p0[0];

        let p1x = (p1[0] - segment_start) / segment_length;
        let p1x_2 = p1x * p1x;
        let p1x_3 = p1x_2 * p1x;

        let p2x = (p2[0] - segment_start) / segment_length;
        let p2x_2 = p2x * p2x;
        let p2x_3 = p2x_2 * p2x;

        let p1_0 = p1[1] - p0[1]; // P1 - P0
        let p2_0 = p2[1] - p0[1]; // P2 - P0
        let p3_0 = p3[1] - p0[1]; // P3 - P0

        let p1x_3_0 = p1x_3 - 1.0; // P1x^3 - 1
        let p1x_3_1 = p1x_3 - p1x; // P1x^3 - P1x
        let p1x_3_2 = p1x_3 - p1x_2; // P1x^3 - P1x^2
        let p2x_2_3 = p2x_2 - p2x_3; // P2x^2 - P2x^3

        let p1xp2x2 = p1x * p2x_2; // P1x * P2x^2
        let p1xp2x3 = p1x * p2x_3; // P1x * P2x^3

        let p1x2p2x = p1x_2 * p2x; // P1x^2 * P2x
        let p1x2p2x3 = p1x_2 * p2x_3; // P1x^2 * P2x^3

        let p1x3p2x = p1x_3 * p2x; // P1x^3 * P2x
        let p1x3p2x2 = p1x_3 * p2x_2; // P1x^3 * P2x^2

        let x = (p2_0 * p1x_3_2) + (p1_0 * p2x_2_3) + (p3_0 * (p1x2p2x3 - p1x3p2x2));
        let y = p1x2p2x3 - p1xp2x3 - p1x3p2x2 + p1xp2x2 + p1x3p2x - p1x2p2x;

        let d = p0[1];
        let c = x / y;
        let b = ((p1[1] - (p1x_3 * p3[1])) + (p1x_3_1 * c) + (p1x_3_0 * d)) / (-p1x_3_2);
        let a = p3[1] - b - c - d;

        let bounds = if t_range == [0.0, 1.0] {
            [p0[0], p3[0]]
        } else {
            [p0[0].lerp(p3[0], t_range[0]), p0[0].lerp(p3[0], t_range[1])]
        };

        Self {
            bounds,
            t_range,
            coefficients: [a, b, c, d],
        }
    }

    pub fn new_ease_in(p0: [f32; 2], p1: [f32; 2], t_range: Option<[f32; 2]>) -> Self {
        if p0[1] == p1[1] {
            return Self::new_linear(p0, p1, t_range);
        }

        let t_range = t_range.unwrap_or([0.0, 1.0]);
        if t_range[0] >= t_range[1] || t_range[0] < 0.0 || t_range[1] > 1.0 {
            panic!("invalid segment t range '{t_range:?}'")
        }

        let bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };

        // We go either from 0 to one (first), or from 1 to 0 (second).
        if p0[1] < p1[1] {
            let min = p0[1];
            let max = p1[1];
            let diff = max - min;

            Self {
                bounds,
                t_range,
                coefficients: [diff, 0.0, 0.0, min], // (P1 - P0) * t^3 + P0
            }
        } else {
            let min = p1[1];
            let max = p0[1];
            let diff = max - min;

            Self {
                bounds,
                t_range,
                coefficients: [-diff, 3.0 * diff, -3.0 * diff, max], // (P0 - P1) * (1-t)^3 + P1 = -(P0 - P1) * t^3 + 3 * (P0 - P1) * t^2 - 3 * (P0 - P1) * t + P0
            }
        }
    }

    pub fn new_ease_out(p0: [f32; 2], p1: [f32; 2], t_range: Option<[f32; 2]>) -> Self {
        if p0[1] == p1[1] {
            return Self::new_linear(p0, p1, t_range);
        }

        let t_range = t_range.unwrap_or([0.0, 1.0]);
        if t_range[0] >= t_range[1] || t_range[0] < 0.0 || t_range[1] > 1.0 {
            panic!("invalid segment t range '{t_range:?}'")
        }

        let bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };

        // We go either from 0 to one (first), or from 1 to 0 (second).
        if p0[1] < p1[1] {
            let min = p0[1];
            let max = p1[1];
            let diff = max - min;

            Self {
                bounds,
                t_range,
                coefficients: [diff, -3.0 * diff, 3.0 * diff, min], // (P1 - P0) * (1 - (1 - t^3)) + P0
            }
        } else {
            let min = p1[1];
            let max = p0[1];
            let diff = max - min;

            Self {
                bounds,
                t_range,
                coefficients: [-diff, 0.0, 0.0, max], // (P0 - P1) * (1-t^3) + P1 = -(P0 - P1) * t^3 + P0
            }
        }
    }

    pub fn new_ease_in_out(p0: [f32; 2], p1: [f32; 2], t_range: Option<[f32; 2]>) -> Box<[Self]> {
        if p0[1] == p1[1] {
            return Box::new([Self::new_linear(p0, p1, t_range)]);
        }

        let t_range = t_range.unwrap_or([0.0, 1.0]);
        if t_range[0] >= t_range[1] || t_range[0] < 0.0 || t_range[1] > 1.0 {
            panic!("invalid segment t range '{t_range:?}'")
        }

        let bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };

        let mid = (p0[0] + p1[0]) / 2.0;
        let mut segments = Vec::new();

        // We go either from 0 to one (first), or from 1 to 0 (second).
        if p0[1] < p1[1] {
            let min = p0[1];
            let max = p1[1];
            let diff = max - min;

            // Check if we need the first segment from t in [0.0, 0.5].
            if bounds[0] <= mid && t_range[0] <= 0.5 {
                let seg_t_range = [t_range[0], 0.5f32.min(t_range[1])];
                let seg_bounds = [bounds[0], p0[0].lerp(p1[0], seg_t_range[1])];
                let seg_coeff = [4.0 * diff, 0.0, 0.0, min]; // (P1 - P0) * (4 * t^3) + P0

                segments.push(Self {
                    bounds: seg_bounds,
                    t_range: seg_t_range,
                    coefficients: seg_coeff,
                });
            }

            // Check if we need the second segment from t in [0.5, 1.0].
            if bounds[0] <= mid && t_range[0] <= 0.5 {
                let seg_t_range = [0.5f32.max(t_range[0]), t_range[1]];
                let seg_bounds = [p0[0].lerp(p1[0], seg_t_range[0]), bounds[1]];
                let seg_coeff = [4.0 * diff, -12.0 * diff, 12.0 * diff, (-3.0 * diff) + min]; // (P1 - P0) * (4 * (t-1)^3 + 1) + P0

                segments.push(Self {
                    bounds: seg_bounds,
                    t_range: seg_t_range,
                    coefficients: seg_coeff,
                });
            }
        } else {
            let min = p1[1];
            let max = p0[1];
            let diff = max - min;

            // Check if we need the first segment from t in [0.0, 0.5].
            if bounds[0] <= mid && t_range[0] <= 0.5 {
                let seg_t_range = [t_range[0], 0.5f32.min(t_range[1])];
                let seg_bounds = [bounds[0], p0[0].lerp(p1[0], seg_t_range[1])];
                let seg_coeff = [-4.0 * diff, 0.0, 0.0, max]; // (P0 - P1) * (1 - (4 * t^3)) + P1

                segments.push(Self {
                    bounds: seg_bounds,
                    t_range: seg_t_range,
                    coefficients: seg_coeff,
                });
            }

            // Check if we need the second segment from t in [0.5, 1.0].
            if bounds[0] <= mid && t_range[0] <= 0.5 {
                let seg_t_range = [0.5f32.max(t_range[0]), t_range[1]];
                let seg_bounds = [p0[0].lerp(p1[0], seg_t_range[0]), bounds[1]];
                let seg_coeff = [-4.0 * diff, 12.0 * diff, -12.0 * diff, (4.0 * diff) + min]; // (P0 - P1) * (-4 * (t-1)^3) + P1

                segments.push(Self {
                    bounds: seg_bounds,
                    t_range: seg_t_range,
                    coefficients: seg_coeff,
                });
            }
        }

        segments.into()
    }

    pub fn split_at(&self, position: f32, op: SegmentRemovalOp) -> Self {
        if !(self.bounds[0]..=self.bounds[1]).contains(&position) {
            panic!("invalid split position");
        }

        // Reconstruct the original bounds.
        let delta_t = self.t_range[1] - self.t_range[0];
        let delta = self.bounds[1] - self.bounds[0];
        let delta_normalized = delta / delta_t;

        let start = self.bounds[0] - (self.t_range[0] * delta_normalized);
        let end = self.bounds[1] + ((1.0 - self.t_range[1]) * delta_normalized);

        let t = position.inv_lerp(start, end);

        match op {
            SegmentRemovalOp::RemoveLeft => Self {
                bounds: [position, self.bounds[1]],
                t_range: [t, self.t_range[1]],
                coefficients: self.coefficients,
            },
            SegmentRemovalOp::RemoveRight => Self {
                bounds: [self.bounds[0], position],
                t_range: [self.t_range[0], t],
                coefficients: self.coefficients,
            },
        }
    }

    pub fn covers_range(&self, range: [f32; 2]) -> bool {
        let r1 = range[0]..=range[1];
        let r2 = self.bounds[0]..=self.bounds[1];

        r1.contains(&self.bounds[0])
            || r1.contains(&self.bounds[1])
            || r2.contains(&range[0])
            || r2.contains(&range[1])
    }

    pub fn is_empty(&self) -> bool {
        self.bounds[0] == self.bounds[1] || self.t_range[0] == self.t_range[1]
    }
}
