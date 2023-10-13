const BOUND_START = 1u;
const BOUND_END = 2u;

const START_BOUND_LEFT = 4u;
const START_BOUND_RIGHT = 8u;

const END_BOUND_LEFT = 16u;
const END_BOUND_RIGHT = 32u;

struct Config {
    line_width: vec2<f32>,
    // Bitset
    // 0 (free line)
    // 1 (start bound to axis): start_args (axis_idx, y_pos[0..1])
    // 2 (end bound to axis): end_args (axis_idx, y_pos[0..1])
    // 4 (use left pos for bound start)
    // 8 (use right pos for bound start)
    // 16 (use left pos for bound end)
    // 32 (use right pos for bound end)
    line_type: u32,
    // 0 (direct)
    // 1 (color map)
    color_mode: u32,
    color: vec3<f32>,
    _padding2: u32,
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

const AXIS_IDX_MASK = (1u << (32u - 4u)) - 1u;
const OVERRIDE_MASK = ~AXIS_IDX_MASK;

const OVERRIDE_BIND_LEFT = (1u << 31u);
const OVERRIDE_BIND_CENTER = (1u << 30u);
const OVERRIDE_BIND_RIGHT = (1u << 29u);

struct LineInfo {
    min_expanded_val: f32,
    _padding: f32,
    start_args: vec2<f32>,
    end_args: vec2<f32>,
    offset_start: vec2<f32>,
    offset_end: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(2)
var<storage, read> axes: array<Axes>;

@group(0) @binding(3)
var<storage, read> line_info: array<LineInfo>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_line: u32,
    @location(2) @interpolate(flat) instance_idx: u32,
}

const right_vector = vec2<f32>(1.0, 0.0);

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
    let line = line_info[instance_idx];

    var line_start = vec2<f32>();
    var line_end = vec2<f32>();
    var discard_line = false;

    if (config.line_type & 1u) == 1u {
        let bindInfo = bitcast<u32>(line.start_args.x);
        let axis_idx = bindInfo & AXIS_IDX_MASK;
        let overrides = bindInfo & OVERRIDE_MASK;
        let axis = axes[axis_idx];

        discard_line = discard_line || axis.expanded_val < line.min_expanded_val;

        let bindLeft = overrides == OVERRIDE_BIND_LEFT || ((overrides == 0u) && (config.line_type & START_BOUND_LEFT) == START_BOUND_LEFT);
        let bindRight = overrides == OVERRIDE_BIND_RIGHT || ((overrides == 0u) && (config.line_type & START_BOUND_RIGHT) == START_BOUND_RIGHT);
        let bindCenter = overrides == OVERRIDE_BIND_CENTER || ((overrides == 0u) && !(bindLeft || bindRight));

        let boundLeftRight = select(axis.position_x.x, axis.position_x.y, bindRight && !bindLeft);
        let boundPosition = select(boundLeftRight, axis.center_x, bindCenter);

        line_start.x = mix(axis.center_x, boundPosition, axis.expanded_val);
        line_start.y = mix(axis.range_y.x, axis.range_y.y, line.start_args.y);
    } else {
        line_start = line.start_args;
    }

    if (config.line_type & 2u) == 2u {
        let bindInfo = bitcast<u32>(line.end_args.x);
        let axis_idx = bindInfo & AXIS_IDX_MASK;
        let overrides = bindInfo & OVERRIDE_MASK;
        let axis = axes[axis_idx];

        discard_line = discard_line || axis.expanded_val < line.min_expanded_val;

        let bindLeft = overrides == OVERRIDE_BIND_LEFT || ((overrides == 0u) && (config.line_type & END_BOUND_LEFT) == END_BOUND_LEFT);
        let bindRight = overrides == OVERRIDE_BIND_RIGHT || ((overrides == 0u) && (config.line_type & END_BOUND_RIGHT) == END_BOUND_RIGHT);
        let bindCenter = overrides == OVERRIDE_BIND_CENTER || ((overrides == 0u) && !(bindLeft || bindRight));

        let boundLeftRight = select(axis.position_x.x, axis.position_x.y, bindRight && !bindLeft);
        let boundPosition = select(boundLeftRight, axis.center_x, bindCenter);

        line_end.x = mix(axis.center_x, boundPosition, axis.expanded_val);
        line_end.y = mix(axis.range_y.x, axis.range_y.y, line.end_args.y);
    } else {
        line_end = line.end_args;
    }

    line_start += line.offset_start;
    line_end += line.offset_end;

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

    return VertexOutput(offset_position, vertex_normal, select(0u, 1u, discard_line), instance_idx);
}