struct Matrices {
    mv_matrix: mat4x4<f32>,
    p_matrix: mat4x4<f32>,
}

struct Config {
    label: u32,
    active_label: u32,
    min_curve_t: f32,
}

struct Axes {
    expanded_val: f32,
    center_x: f32,
    position_x: vec2<f32>,
    range_y: vec2<f32>,
}

struct LabelColor {
    color_high: vec4<f32>,
    color_low: vec4<f32>,
}

struct CurveLineInfo {
    x_t_values: vec2<f32>,
    y_t_values: vec2<f32>,
    axis: u32,
}

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(2)
var<storage> axes: array<Axes>;

@group(0) @binding(3)
var<storage> curve: array<CurveLineInfo>;

@group(0) @binding(4)
var<storage> colors: array<LabelColor>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) discard_segment: u32,
}

const INDEX_BUFFER = array<u32, 6>(0u, 1u, 2u, 1u, 3u, 2u);

fn xyz_to_srgb(color: vec3<f32>) -> vec3<f32> {
    const conversion_matrix = mat3x3<f32>(
        vec3<f32>(3.240812398895283, -0.9692430170086407, 0.055638398436112804),
        vec3<f32>(-1.5373084456298136, 1.8759663029085742, -0.20400746093241362),
        vec3<f32>(-0.4985865229069666, 0.04155503085668564, 1.0571295702861434),
    );

    let linear_srgb = conversion_matrix * color.xyz;
    let a = 12.92 * linear_srgb;
    let b = 1.055 * pow(linear_srgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), linear_srgb);
    let srgb = mix(a, b, c);
    return srgb;
}

@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    let index = INDEX_BUFFER[vertex_idx];
    let segment = curve[instance_idx];
    let axis = axes[segment.axis];
    let discard_segment = select(
        0u,
        1u,
        axis.expanded_val < max(segment.x_t_values.x, segment.x_t_values.y) || (segment.x_t_values.x == 0.0 && segment.x_t_values.y == 0.0)
    );

    let left_vertex = index % 2u == 0u;
    let top_vertex = index > 1u;

    let curve_value = select(config.min_curve_t, select(segment.x_t_values.y, segment.x_t_values.x, left_vertex), top_vertex);
    let curve_position = select(segment.y_t_values.y, segment.y_t_values.x, left_vertex);

    let x = mix(axis.center_x, axis.position_x.x, curve_value);
    let y = mix(axis.range_y.x, axis.range_y.y, curve_position);
    let pos = matrices.p_matrix * matrices.mv_matrix * vec4<f32>(x, y, 0.0, 1.0);

    return VertexOutput(pos, discard_segment);
}

@fragment
fn fragment_main(
    @location(0) @interpolate(flat) discard_segment: u32
) -> @location(0) vec4<f32> {
    if discard_segment == 1u {
        discard;
    }

    let inactive_color = colors[config.label].color_low;
    let active_color = colors[config.label].color_high;
    let color = select(inactive_color, active_color, config.label == config.active_label);
    let alpha_factor = select(1.0, 0.2, config.label != config.active_label);

    let xyz = color.xyz;
    let a = color.a * alpha_factor;

    let rgb = xyz_to_srgb(xyz);
    return vec4<f32>(rgb * a, a);
}