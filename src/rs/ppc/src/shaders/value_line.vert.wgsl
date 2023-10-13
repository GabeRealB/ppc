struct Matrices {
    mv_matrix: mat4x4<f32>,
    p_matrix: mat4x4<f32>,
}

struct Config {
    line_width: vec2<f32>,
    selection_threshold: f32,
    // padding: 4 bytes
    unselected_color: vec4<f32>,
}

struct Axes {
    expanded_val: f32,
    center_x: f32,
    position_x: vec2<f32>,
    range_y: vec2<f32>,
}

struct ValueLine {
    curve_idx: u32,
    start_axis: u32,
    start_value: f32,
    end_axis: u32,
    end_value: f32,
}

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(2)
var<storage, read> axes: array<Axes>;

@group(0) @binding(3)
var<storage, read> values: array<ValueLine>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_value: u32,
    @location(2) @interpolate(flat) instance_idx: u32,
}

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

    let value = values[instance_idx];

    let start_axis = axes[value.start_axis];
    let end_axis = axes[value.end_axis];

    let start_x = mix(start_axis.center_x, start_axis.position_x.y, start_axis.expanded_val);
    let end_x = mix(end_axis.center_x, end_axis.position_x.x, end_axis.expanded_val);

    let line_start = vec2<f32>(start_x, mix(start_axis.range_y.x, start_axis.range_y.y, value.start_value));
    let line_end = vec2<f32>(end_x, mix(end_axis.range_y.x, end_axis.range_y.y, value.end_value));

    // let discard_start = (value.start_value < start_axis.range_y.x) || (value.start_value > start_axis.range_y.y);
    // let discard_end = (value.end_value < end_axis.range_y.x) || (value.end_value > end_axis.range_y.y);
    // let discard_value = select(0u, 0u, discard_start || discard_end);
    let discard_value = 0u;

    let line_vector = normalize(line_end - line_start);
    let line_unit_cos = line_vector.x;
    let line_unit_sin = line_vector.y;

    let rotation_matrix = mat2x2<f32>(
        line_unit_cos,
        line_unit_sin,    // column 1: [cos theta, sin theta]
        -line_unit_sin,
        line_unit_cos,   // column 2: [-sin theta, cos theta]
    );
    var vertex_normals = array<vec2<f32>, 4>(
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
    );
    let vertex_normal = rotation_matrix * vertex_normals[index];
    let vertex_pos = select(line_start, line_end, vec2<bool>(index <= 1u));

    let delta = matrices.mv_matrix * vec4<f32>(vertex_normal * config.line_width, 0.0, 0.0);
    let pos = matrices.mv_matrix * vec4<f32>(vertex_pos, 0.0, 1.0);
    let offset_position = matrices.p_matrix * (pos + delta);

    return VertexOutput(offset_position, vertex_normal, discard_value, value.curve_idx);
}