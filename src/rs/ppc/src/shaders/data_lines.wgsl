struct Matrices {
    mv_matrix: mat4x4<f32>,
    p_matrix: mat4x4<f32>,
}

struct Config {
    line_width: vec2<f32>,
    selection_bounds: vec2<f32>,
    color_probabilities: u32,
    unselected_color: vec4<f32>,
}

struct Axes {
    expanded_val: f32,
    center_x: f32,
    position_x: vec2<f32>,
    range_y: vec2<f32>,
}

struct DataLine {
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
var<storage, read> values: array<DataLine>;

@group(0) @binding(4)
var<storage, read> color_values: array<f32>;

@group(0) @binding(5)
var<storage, read> probabilities: array<f32>;

@group(0) @binding(6)
var color_scale: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_value: u32,
    @location(2) @interpolate(flat) instance_idx: u32,
}

const INDEX_BUFFER = array<u32, 6>(0u, 1u, 2u, 1u, 3u, 2u);
const VERTEX_NORMALS_BUFFER = array<vec2<f32>, 4>(
    vec2<f32>(0.0, -1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, -1.0),
    vec2<f32>(0.0, 1.0),
);

fn get_line_alpha(normal: vec2<f32>) -> f32 {
    const feather: f32 = 0.5;
    const one_minus_feather: f32 = 1.0 - feather;

    let distance = length(normal);
    if distance <= one_minus_feather {
        return 1.0;
    } else if distance <= 1.0 {
        let t = (distance - feather) / one_minus_feather;
        return mix(1.0, 0.0, t);
    }

    return 0.0;
}

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
    let vertex_normal = rotation_matrix * VERTEX_NORMALS_BUFFER[index];
    let vertex_pos = select(line_start, line_end, vec2<bool>(index <= 1u));

    let delta = matrices.mv_matrix * vec4<f32>(vertex_normal * config.line_width, 0.0, 0.0);
    let pos = matrices.mv_matrix * vec4<f32>(vertex_pos, 0.0, 1.0);
    let offset_position = matrices.p_matrix * (pos + delta);

    return VertexOutput(offset_position, vertex_normal, discard_value, value.curve_idx);
}

@fragment
fn fragment_main(
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_value: u32,
    @location(2) @interpolate(flat) instance_idx: u32
) -> @location(0) vec4<f32> {
    if discard_value != 0u {
        discard;
    }

    let alpha = get_line_alpha(normal);

    let color_value = color_values[instance_idx];
    let probability = probabilities[instance_idx];

    let num_samples = textureDimensions(color_scale).x;
    let sample_position = select(color_value, probability, config.color_probabilities == 1u) * f32(num_samples - 1u);
    let sample_1_pos = i32(floor(sample_position));
    let sample_2_pos = i32(ceil(sample_position));
    let t = fract(sample_position);

    let sample_1 = textureLoad(color_scale, vec2(sample_1_pos, 0), 0);
    let sample_2 = textureLoad(color_scale, vec2(sample_2_pos, 0), 0);
    let color_scale_color = mix(sample_1, sample_2, t);

    let sample_in_bounds_0 = config.selection_bounds.x <= probability;
    let sample_in_bounds_1 = probability <= config.selection_bounds.y;
    let color_selection = vec4<bool>(sample_in_bounds_0 && sample_in_bounds_1);
    let color = select(config.unselected_color, color_scale_color, color_selection);

    let color_alpha = color.a;
    let color_srgb = xyz_to_srgb(color.rgb);

    return vec4<f32>(color_srgb * alpha * color_alpha, alpha * color_alpha);
}