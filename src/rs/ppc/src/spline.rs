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
            let seg = self.segments.remove(end_idx);
            let segments = seg.get_maximum_segments(&segment);

            for (i, segment) in segments.into_iter().enumerate() {
                if !segment.is_empty() {
                    self.segments.insert(end_idx + i, segment);
                }
            }
            return;
        }

        let start_idx = start_idx.saturating_sub(1);
        let end_idx = end_idx.saturating_add(1);

        let mut new_segments = vec![];
        let segments = self.segments.drain(start_idx..end_idx);
        for s in segments {
            if !segment.covers_range(s.bounds) {
                new_segments.push(s);
                continue;
            }

            let mut maximum_segments = s.get_maximum_segments(&segment);
            if s.bounds[1] < segment.bounds[1] {
                maximum_segments.pop();
            }
            new_segments.extend(maximum_segments);
        }

        if let Some(last) = new_segments.last() {
            if last.bounds[1] < segment.bounds[1] {
                let rest = segment.split_at(last.bounds[1], SegmentRemovalOp::RemoveLeft);
                new_segments.push(rest);
            }
        }

        self.segments.splice(start_idx..start_idx, new_segments);
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
    const PRECISION: f32 = 1e-5;

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

        let mut bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };
        if (0.0..=Self::PRECISION).contains(&bounds[0]) {
            bounds[0] = 0.0;
        }
        if (1.0 - Self::PRECISION..=1.0).contains(&bounds[1]) {
            bounds[1] = 1.0;
        }

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

        let mut bounds = if t_range == [0.0, 1.0] {
            [p0[0], p2[0]]
        } else {
            [p0[0].lerp(p2[0], t_range[0]), p0[0].lerp(p2[0], t_range[1])]
        };
        if (0.0..=Self::PRECISION).contains(&bounds[0]) {
            bounds[0] = 0.0;
        }
        if (1.0 - Self::PRECISION..=1.0).contains(&bounds[1]) {
            bounds[1] = 1.0;
        }

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

        let mut bounds = if t_range == [0.0, 1.0] {
            [p0[0], p3[0]]
        } else {
            [p0[0].lerp(p3[0], t_range[0]), p0[0].lerp(p3[0], t_range[1])]
        };
        if (0.0..=Self::PRECISION).contains(&bounds[0]) {
            bounds[0] = 0.0;
        }
        if (1.0 - Self::PRECISION..=1.0).contains(&bounds[1]) {
            bounds[1] = 1.0;
        }

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

        let mut bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };
        if (0.0..=Self::PRECISION).contains(&bounds[0]) {
            bounds[0] = 0.0;
        }
        if (1.0 - Self::PRECISION..=1.0).contains(&bounds[1]) {
            bounds[1] = 1.0;
        }

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

        let mut bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };
        if (0.0..=Self::PRECISION).contains(&bounds[0]) {
            bounds[0] = 0.0;
        }
        if (1.0 - Self::PRECISION..=1.0).contains(&bounds[1]) {
            bounds[1] = 1.0;
        }

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

        let mut bounds = if t_range == [0.0, 1.0] {
            [p0[0], p1[0]]
        } else {
            [p0[0].lerp(p1[0], t_range[0]), p0[0].lerp(p1[0], t_range[1])]
        };
        if (0.0..=Self::PRECISION).contains(&bounds[0]) {
            bounds[0] = 0.0;
        }
        if (1.0 - Self::PRECISION..=1.0).contains(&bounds[1]) {
            bounds[1] = 1.0;
        }

        let mid = (p0[0] + p1[0]) / 2.0;
        let mut segments = Vec::new();

        // We go either from 0 to one (first), or from 1 to 0 (second).
        if p0[1] < p1[1] {
            let min = p0[1];
            let max = p1[1];
            let diff = max - min;

            // Check if we need the first segment from t in [0.0, 0.5].
            if (p0[0]..=mid).contains(&bounds[0]) && (0.0..=0.5).contains(&t_range[0]) {
                let seg_t_range = [t_range[0], 0.5f32.min(t_range[1])];
                let mut seg_bounds = [bounds[0], p0[0].lerp(p1[0], seg_t_range[1])];
                let seg_coeff = [4.0 * diff, 0.0, 0.0, min]; // (P1 - P0) * (4 * t^3) + P0
                if (1.0 - Self::PRECISION..=1.0).contains(&seg_bounds[1]) {
                    seg_bounds[1] = 1.0;
                }

                segments.push(Self {
                    bounds: seg_bounds,
                    t_range: seg_t_range,
                    coefficients: seg_coeff,
                });
            }

            // Check if we need the second segment from t in [0.5, 1.0].
            if (mid..=p1[0]).contains(&bounds[1]) && (0.5..=1.0).contains(&t_range[1]) {
                let seg_t_range = [0.5f32.max(t_range[0]), t_range[1]];
                let mut seg_bounds = [p0[0].lerp(p1[0], seg_t_range[0]), bounds[1]];
                let seg_coeff = [4.0 * diff, -12.0 * diff, 12.0 * diff, (-3.0 * diff) + min]; // (P1 - P0) * (4 * (t-1)^3 + 1) + P0
                if (0.0..=Self::PRECISION).contains(&seg_bounds[0]) {
                    seg_bounds[0] = 0.0;
                }

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
            if (p0[0]..=mid).contains(&bounds[0]) && (0.0..=0.5).contains(&t_range[0]) {
                let seg_t_range = [t_range[0], 0.5f32.min(t_range[1])];
                let mut seg_bounds = [bounds[0], p0[0].lerp(p1[0], seg_t_range[1])];
                let seg_coeff = [-4.0 * diff, 0.0, 0.0, max]; // (P0 - P1) * (1 - (4 * t^3)) + P1
                if (1.0 - Self::PRECISION..=1.0).contains(&seg_bounds[1]) {
                    seg_bounds[1] = 1.0;
                }

                segments.push(Self {
                    bounds: seg_bounds,
                    t_range: seg_t_range,
                    coefficients: seg_coeff,
                });
            }

            // Check if we need the second segment from t in [0.5, 1.0].
            if (mid..=p1[0]).contains(&bounds[1]) && (0.5..=1.0).contains(&t_range[1]) {
                let seg_t_range = [0.5f32.max(t_range[0]), t_range[1]];
                let mut seg_bounds = [p0[0].lerp(p1[0], seg_t_range[0]), bounds[1]];
                let seg_coeff = [-4.0 * diff, 12.0 * diff, -12.0 * diff, (4.0 * diff) + min]; // (P0 - P1) * (-4 * (t-1)^3) + P1
                if (0.0..=Self::PRECISION).contains(&seg_bounds[0]) {
                    seg_bounds[0] = 0.0;
                }

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

    pub fn normalize_t_range(&self) -> Self {
        let bounds = self.bounds;
        let x0 = self.t_range[0];
        let len = self.t_range[1] - self.t_range[0];
        let [a, b, c, d] = self.coefficients;

        // Expand g(x) = f(x+x0)
        let a_t = a;
        let b_t = b + (3.0 * a * x0);
        let c_t = c + (x0 * ((2.0 * b) + (3.0 * a * x0)));
        let d_t = d + (x0 * (c + (x0 * (b + (a * x0)))));

        let len2 = len * len;
        let len3 = len2 * len;

        // Expand h(x) = g(x*len)
        let a_s = a_t * len3;
        let b_s = b_t * len2;
        let c_s = c_t * len;
        let d_s = d_t;

        Self {
            bounds,
            t_range: [0.0, 1.0],
            coefficients: [a_s, b_s, c_s, d_s],
        }
    }

    pub fn get_maximum_segments(&self, other: &Self) -> Vec<Self> {
        assert!(self.covers_range(other.bounds));
        assert!(
            !(self.is_empty() && other.is_empty()),
            "can not have two empty segments"
        );

        // Early return, if one segment is empty.
        if self.is_empty() {
            return vec![*other];
        } else if other.is_empty() {
            return vec![*self];
        }

        let mut segments: Vec<Self> = vec![];
        let mut push_segment = |segment: Self| {
            if segment.is_empty() {
                return;
            }

            if let Some(last) = segments.last_mut() {
                if last.coefficients == segment.coefficients
                    && last.bounds[1] == segment.bounds[0]
                    && last.t_range[1] == segment.t_range[0]
                {
                    last.bounds[1] = segment.bounds[1];
                    last.t_range[1] = segment.t_range[1];
                } else {
                    segments.push(segment);
                }
            } else {
                segments.push(segment);
            }
        };

        // Order the segments by the start bound.
        let (first, second) = if self.bounds[0] <= other.bounds[0] {
            (self, other)
        } else {
            (other, self)
        };

        // Extract the uncovered first part and insert it.
        let left = first.split_at(second.bounds[0], SegmentRemovalOp::RemoveRight);
        let first = first.split_at(second.bounds[0], SegmentRemovalOp::RemoveLeft);

        push_segment(left);

        // Early exit, if the rest of the segment is empty.
        if first.is_empty() {
            push_segment(*second);
            return segments;
        }

        // Order the segments by the end bound.
        let (first, second) = if first.bounds[1] <= second.bounds[1] {
            (&first, second)
        } else {
            (second, &first)
        };

        let overlapping_first = *first;
        let overlapping_second = second.split_at(first.bounds[1], SegmentRemovalOp::RemoveRight);

        let right = second.split_at(first.bounds[1], SegmentRemovalOp::RemoveLeft);

        // To find the maximum of two polynomials we first normalize them to be in the same range.
        // Afterwards we build a third polynomial, by subtracting the second one from the first one.
        // This polynomial will evaluate to a value bigger than 0 at a position x, exactly if the
        // the evaluation of the first polynomial at x is bigger than the value at the same position
        // for the second one. To find the ranges in which one is bigger than the other, we search for
        // the zeros.
        let first_coeff = overlapping_first.normalize_t_range().coefficients;
        let second_coeff = overlapping_second.normalize_t_range().coefficients;
        let difference = Polynomial {
            a3: first_coeff[0] as f64 - second_coeff[0] as f64,
            a2: first_coeff[1] as f64 - second_coeff[1] as f64,
            a1: first_coeff[2] as f64 - second_coeff[2] as f64,
            a0: first_coeff[3] as f64 - second_coeff[3] as f64,
        };

        // The call works only with polynomials of degree > 0.
        // For constants we return None and must determine the maximum
        // manually.
        if let Some(zeros) = difference.zeros() {
            // The difference is valid in the range [0.0, 1.0], so we can
            // identify invalid points by being outside of this range.
            // It can occur either because we unwrap a None to an invalid
            // value, or if the zero lies outside of the range of interest.
            let mut zeros = zeros.map(|x| x.unwrap_or(-1.0));
            zeros.sort_unstable_by(f64::total_cmp);

            let maximum_segments = [
                (0.0, zeros[0]),
                (zeros[0], zeros[1]),
                (zeros[1], zeros[2]),
                (zeros[2], 1.0),
            ];

            let mut last_bound: f64 = 0.0;
            for (x0, x1) in maximum_segments {
                // Skip segments outside or range of [0.0, 1.0].
                if x1 <= last_bound || x0 >= 1.0 || x0 == x1 {
                    continue;
                }

                let x0 = x0.clamp(0.0, 1.0);
                let x1 = x1.clamp(0.0, 1.0);

                // We evaluate the difference at the middle of the segment to
                // determine, if it is larger or smaller than 0.
                let midpoint = (x0 + x1) / 2.0;

                let segment = if difference.evaluate_at(midpoint) <= 0.0 {
                    let [start_bound, end_bound] = overlapping_first.bounds;
                    let segment_start = start_bound.lerp(end_bound, x0 as f32);
                    let segment_end = start_bound.lerp(end_bound, x1 as f32);

                    overlapping_second
                        .split_at(segment_start, SegmentRemovalOp::RemoveLeft)
                        .split_at(segment_end, SegmentRemovalOp::RemoveRight)
                } else {
                    let [start_bound, end_bound] = overlapping_second.bounds;
                    let segment_start = start_bound.lerp(end_bound, x0 as f32);
                    let segment_end = start_bound.lerp(end_bound, x1 as f32);

                    overlapping_first
                        .split_at(segment_start, SegmentRemovalOp::RemoveLeft)
                        .split_at(segment_end, SegmentRemovalOp::RemoveRight)
                };

                push_segment(segment);
                last_bound = x1;
            }
        } else if difference.evaluate_at(0.5) >= 0.0 {
            push_segment(overlapping_first);
        } else {
            push_segment(overlapping_second);
        }

        // Include the non overlapping part of the second segment.
        push_segment(right);
        segments
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Polynomial {
    a3: f64,
    a2: f64,
    a1: f64,
    a0: f64,
}

impl Polynomial {
    fn degree(&self) -> usize {
        if self.a3 != 0.0 {
            3
        } else if self.a2 != 0.0 {
            2
        } else if self.a1 != 0.0 {
            1
        } else {
            0
        }
    }

    fn evaluate_at(&self, x: f64) -> f64 {
        self.a0 + x * (self.a1 + x * (self.a2 + x * self.a3))
    }

    fn zeros(&self) -> Option<[Option<f64>; 3]> {
        match self.degree() {
            0 => None,
            1 => {
                let x = self.zeroes_linear();
                Some([Some(x), None, None])
            }
            2 => {
                let [x1, x2] = self.zeros_quadratic();
                Some([x1, x2, None])
            }
            3 => Some(self.zeros_cubic()),
            _ => unreachable!(),
        }
    }

    fn zeroes_linear(&self) -> f64 {
        assert_eq!(
            self.degree(),
            1,
            "'zeroes_linear' can only be called on linear polynomials"
        );

        // a1 * x + a0 = 0 => x = -a0/a1
        -self.a0 / self.a1
    }

    fn zeros_quadratic(&self) -> [Option<f64>; 2] {
        assert_eq!(
            self.degree(),
            2,
            "'zeros_quadratic' can only be called on quadratic polynomials"
        );

        // Normalize the polynomial.
        let a1 = self.a1 / self.a2;
        let a0 = self.a0 / self.a2;

        // Use the pq-formula. (-a1/2) +- sqrt((a1/2)^2 - a0))
        let p = -0.5 * a1;
        let d = p.powi(2) - a0;
        if d > 0.0 {
            let d = d.sqrt();
            let x1 = p - d;
            let x2 = p + d;
            [Some(x1), Some(x2)]
        } else if d == 0.0 {
            let x1 = p;
            [Some(x1), None]
        } else {
            [None; 2]
        }
    }

    fn zeros_cubic(&self) -> [Option<f64>; 3] {
        assert_eq!(
            self.degree(),
            3,
            "'zeros_cubic' can only be called on cubic polynomials"
        );

        // Taken from https://www.uni-koeln.de/deiters/math/supplement.pdf
        const PRECISION: f64 = 1.0e-7;

        // Normalize
        let w = 1.0 / self.a3;
        let normalized = Self {
            a3: 1.0,
            a2: self.a2 * w,
            a1: self.a1 * w,
            a0: self.a0 * w,
        };

        // root at zero?
        if normalized.a0 == 0.0 {
            // Polynomial division by x
            let quadratic = Self {
                a3: 0.0,
                a2: normalized.a3,
                a1: normalized.a2,
                a0: normalized.a1,
            };
            let [x1, x2] = quadratic.zeros_quadratic();
            return [Some(0.0), x1, x2];
        }

        let x_infl = -normalized.a2 / 3.0;
        let y = normalized.evaluate_at(x_infl);

        // Is inflection point a root?
        if y == 0.0 {
            let c1 = x_infl + normalized.a2;
            let c0 = c1 * x_infl + normalized.a1;

            // Polynomial division by (x - x_infl)
            let quadratic = Self {
                a3: 0.0,
                a2: 1.0,
                a1: c1,
                a0: c0,
            };
            let [x1, x2] = quadratic.zeros_quadratic();
            return [Some(x_infl), x1, x2];
        }

        let d = normalized.a2.powi(2) - (3.0 * normalized.a1);

        // Laguerre-Nair-Samuelson bounds
        let i_slope = d.signum();
        if d == 0.0 {
            let x0 = x_infl - y.cbrt();
            return [Some(x0), None, None];
        }

        let mut x0 = if i_slope == 1.0 {
            let delta = y.signum() * -2.0 / 3.0 * d.sqrt();
            x_infl + delta
        } else {
            x_infl
        };

        // Halley’s method
        loop {
            let y = normalized.a2 + x0;
            let y1 = 2.0 * y + x0;
            let y2 = y1 + 3.0 * x0;
            let y1 = x0 * y1 + normalized.a1;
            let y = (x0 * y + normalized.a1) * x0 + normalized.a0;
            let dx = y * y1 / (y1.powi(2) - 0.5 * y * y2);
            x0 -= dx;

            // Terminate when the error is less than 1.0e-18.
            if dx.abs() <= PRECISION * x0.abs() {
                break;
            }
        }

        let newton1 = |normalized: &Self, x: f64| -> f64 {
            let y = normalized.a2 + x;
            let y1 = 2.0 * y + x;
            let y1 = x * y1 + normalized.a1;
            let y = (x * y + normalized.a1) * x + normalized.a0;

            if y1 != 0.0 {
                x - (y / y1)
            } else {
                x
            }
        };

        let eq_quadratic = |quadratic: &Self, normalized: &Self| -> [Option<f64>; 2] {
            let p = -0.5 * quadratic.a1;
            let d = p.powi(2) - quadratic.a0;
            if d >= 0.0 {
                let d = d.sqrt();
                if p < 0.0 {
                    let x1 = newton1(normalized, p - d);
                    let x2 = p + d;
                    [Some(x1), Some(x2)]
                } else {
                    let x1 = p - d;
                    let x2 = newton1(normalized, p + d);
                    [Some(x1), Some(x2)]
                }
            } else {
                [None, None]
            }
        };

        if i_slope == 1.0 {
            // Polynomial division by (x - x_infl)
            let c1 = x0 + normalized.a2;
            let c0 = c1 * x0 + normalized.a1;
            let quadratic = Self {
                a3: 0.0,
                a2: 1.0,
                a1: c1,
                a0: c0,
            };
            let [x1, x2] = eq_quadratic(&quadratic, &normalized);
            [Some(x0), x1, x2]
        } else {
            [Some(x0), None, None]
        }
    }
}
