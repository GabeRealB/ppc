struct Matrices {
    mv_matrix: mat4x4<f32>,
    p_matrix: mat4x4<f32>,
}

struct Axes {
    expanded_val: f32,
    center_x: f32,
    position_x: vec2<f32>,
    range_y: vec2<f32>,
}

struct CurveSample {
    axis: u32,
    color_idx: u32,
    value: f32,
    position: f32
}

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<storage, read> colors: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read> axes: array<Axes>;

@group(0) @binding(3)
var<storage, read> samples: array<CurveSample>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) xyz_color: vec4<f32>,
    @location(1) @interpolate(flat) discard_vertex: u32
}

const CURVE_SEGMENT_WIDTH: f32 = 0.2;
const CURVE_SEGMENT_OFFSET_LOW: f32 = 0.8 * CURVE_SEGMENT_WIDTH;
const CURVE_SEGMENT_OFFSET_HIGH: f32 = 0.05 * CURVE_SEGMENT_WIDTH;

@vertex
fn main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var index_buffer = array<u32, 6>(
        0u,
        1u,
        2u,
        1u,
        3u,
        2u,
    );
    let index = index_buffer[vertex_idx];

    let sample_idx = select(instance_idx, instance_idx + 1u, index <= 1u);
    let sample = samples[sample_idx];
    let axis = axes[sample.axis];

    let x_offset_start = mix(CURVE_SEGMENT_OFFSET_LOW, CURVE_SEGMENT_OFFSET_HIGH, sample.value);
    let x_offset_end = CURVE_SEGMENT_OFFSET_LOW;

    // Use start on even indices and end otherwise
    let x_offset = select(x_offset_start, x_offset_end, (index & 1u) == 1u);
    let axis_left = mix(axis.center_x, axis.position_x.x, axis.expanded_val);
    let axis_y = mix(axis.range_y.x, axis.range_y.y, sample.position);

    let vertex_pos = matrices.p_matrix * matrices.mv_matrix * vec4<f32>(axis_left + x_offset, axis_y, 0.0, 1.0);
    let vertex_color = colors[sample.color_idx];
    let discard_vertex = select(0u, 1u, sample.value > axis.expanded_val);

    return VertexOutput(vertex_pos, vertex_color, discard_vertex);
}