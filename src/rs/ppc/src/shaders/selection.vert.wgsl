struct Matrices {
    mv_matrix: mat4x4<f32>,
    p_matrix: mat4x4<f32>,
}

struct Config {
    line_width: vec2<f32>,
    // padding: 8 bytes
    high_color: vec3<f32>,
    // padding: 4 bytes
    low_color: vec3<f32>,
    // padding: 4 bytes
}

struct Axes {
    expanded_val: f32,
    center_x: f32,
    position_x: vec2<f32>,
    range_y: vec2<f32>,
}

struct SelectionLineInfo {
    axis: u32,
    use_color: u32,
    use_left: u32,
    offset_x: f32,
    color_idx: u32,
    use_low_color: u32,
    range: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(2)
var<storage, read> axes: array<Axes>;

@group(0) @binding(3)
var<storage, read> selections: array<SelectionLineInfo>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec2<f32>,
    @location(1) curve_pos: f32,
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

    let selection = selections[instance_idx];
    let axis = axes[selection.axis];

    let use_left_position = selection.use_left != 0u;

    let x_pos = select(axis.center_x, mix(axis.center_x, axis.position_x.x, axis.expanded_val), use_left_position) + selection.offset_x;
    let line_start = vec2<f32>(x_pos, mix(axis.range_y.x, axis.range_y.y, selection.range.x));
    let line_end = vec2<f32>(x_pos, mix(axis.range_y.x, axis.range_y.y, selection.range.y));

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
    let curve_pos = select(selection.range.x, selection.range.y, index <= 1u);

    return VertexOutput(offset_position, vertex_normal, curve_pos, instance_idx);
}