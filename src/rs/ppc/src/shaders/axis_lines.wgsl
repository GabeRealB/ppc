struct Config {
    line_width: vec2<f32>,
    color: vec3<f32>,
}

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

struct AxisLineInfo {
    axis: u32,
    axis_position: f32,
    min_expanded_val: f32,
}

const AXIS_LEFT: u32 = 0u;
const AXIS_CENTER: u32 = 1u;
const AXIS_RIGHT: u32 = 2u;

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(2)
var<storage, read> axes: array<Axes>;

@group(0) @binding(3)
var<storage, read> line_info: array<AxisLineInfo>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_line: u32,
}

const FEATHER: f32 = 0.5;
const ONE_MINUS_FEATHER: f32 = 1.0 - FEATHER;

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var INDEX_BUFFER = array<u32, 6>(0u, 1u, 2u, 1u, 3u, 2u);
    var VERTEX_NORMALS_BUFFER = array<vec2<f32>, 4>(
        vec2<f32>(0.0, -1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, -1.0),
        vec2<f32>(0.0, 1.0),
    );

    let index = INDEX_BUFFER[vertex_idx];
    let line = line_info[instance_idx];

    let axis = axes[line.axis];
    var line_start = vec2<f32>(0.0, axis.range_y.x);
    var line_end = vec2<f32>(0.0, axis.range_y.y);
    var discard_line = axis.expanded_val < line.min_expanded_val;

    let left_bound = select(axis.position_x.x, axis.center_x, line.axis_position >= 0.0);
    let right_bound = select(axis.position_x.y, axis.center_x, line.axis_position <= 0.0);

    let mix_factor = select(abs(line.axis_position), 1.0 + line.axis_position, line.axis_position < 0);
    let position = mix(left_bound, right_bound, mix_factor);
    line_start.x = position;
    line_end.x = position;

    let line_vector = normalize(line_end - line_start);
    let line_unit_cos = line_vector.x;
    let line_unit_sin = line_vector.y;

    let rotation_matrix = mat2x2<f32>(
        line_unit_cos,
        line_unit_sin,    // column 1: [cos theta, sin theta]
        -line_unit_sin,
        line_unit_cos,   // column 2: [-sin theta, cos theta]
    );
    let vertex_normal = rotation_matrix * VERTEX_NORMALS_BUFFER[index];
    let vertex_pos = select(line_start, line_end, vec2<bool>(index <= 1u));

    let line_width = select(config.line_width, config.line_width * 2.0, line.axis_position == 0);
    let delta = matrices.mv_matrix * vec4<f32>(vertex_normal * line_width, 0.0, 0.0);
    let pos = matrices.mv_matrix * vec4<f32>(vertex_pos, 0.0, 1.0);
    let offset_position = matrices.p_matrix * (pos + delta);

    return VertexOutput(offset_position, vertex_normal, select(0u, 1u, discard_line));
}

@fragment
fn fragment_main(
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_line: u32,
) -> @location(0) vec4<f32> {
    if discard_line != 0u {
        discard;
    }

    let distance = length(normal);
    var alpha = 0.0;

    if distance <= ONE_MINUS_FEATHER {
        alpha = 1.0;
    } else if distance <= 1.0 {
        let t = (distance - FEATHER) / ONE_MINUS_FEATHER;
        alpha = mix(1.0, 0.0, t);
    }

    let color = config.color;
    return vec4<f32>(color * alpha, alpha);
}