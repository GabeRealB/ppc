struct Matrices {
    mv_matrix: mat4x4<f32>,
    p_matrix: mat4x4<f32>,
}

struct Config {
    line_width: vec2<f32>,
    high_color: vec3<f32>,
    low_color: vec3<f32>,
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

struct LabelColor {
    color_high: vec4<f32>,
    color_low: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> matrices: Matrices;

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(2)
var<storage, read> axes: array<Axes>;

@group(0) @binding(3)
var<storage, read> selections: array<SelectionLineInfo>;

@group(0) @binding(4)
var<storage> colors: array<LabelColor>;

@group(0) @binding(5)
var probability_curves: texture_2d_array<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec2<f32>,
    @location(1) curve_pos: f32,
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
    let vertex_normal = rotation_matrix * VERTEX_NORMALS_BUFFER[index];
    let vertex_pos = select(line_start, line_end, vec2<bool>(index <= 1u));

    let delta = matrices.mv_matrix * vec4<f32>(vertex_normal * config.line_width, 0.0, 0.0);
    let pos = matrices.mv_matrix * vec4<f32>(vertex_pos, 0.0, 1.0);
    let offset_position = matrices.p_matrix * (pos + delta);
    let curve_pos = select(selection.range.x, selection.range.y, index <= 1u);

    return VertexOutput(offset_position, vertex_normal, curve_pos, instance_idx);
}

@fragment
fn fragment_main(
    @location(0) normal: vec2<f32>,
    @location(1) curve_pos: f32,
    @location(2) @interpolate(flat) instance_idx: u32
) -> @location(0) vec4<f32> {
    let alpha = get_line_alpha(normal);

    let selection = selections[instance_idx];

    let num_samples = textureDimensions(probability_curves).x;
    let sample_position = curve_pos * f32(num_samples - 1u);
    let sample_1_pos = i32(floor(sample_position));
    let sample_2_pos = i32(ceil(sample_position));
    let t = fract(sample_position);

    let texture_array_index = i32(selection.axis);
    let sample_1 = textureLoad(probability_curves, vec2(sample_1_pos, 0), texture_array_index, 0).r;
    let sample_2 = textureLoad(probability_curves, vec2(sample_2_pos, 0), texture_array_index, 0).r;
    let sample = mix(sample_1, sample_2, t);

    if selection.use_color != 0u {
        let color_xyz = select(colors[selection.color_idx].color_high, colors[selection.color_idx].color_low, selection.use_low_color == 1u);
        let color = xyz_to_srgb(color_xyz.rgb);
        return vec4<f32>(color * alpha, alpha);
    } else {
        let gradient = xyz_to_srgb(mix(config.low_color, config.high_color, sample));
        return vec4<f32>(gradient * alpha, alpha);
    }
}