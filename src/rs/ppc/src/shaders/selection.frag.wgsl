struct Config {
    line_width: vec2<f32>,
    // padding: 8 bytes
    high_color: vec3<f32>,
    // padding: 4 bytes
    low_color: vec3<f32>,
    // padding: 4 bytes
}

struct SelectionInfo {
    axis: u32,
    use_color: u32,
    use_left: u32,
    offset_x: f32,
    range: vec2<f32>,
    // padding: 8 bytes
    color: vec3<f32>,
    // padding: 4 bytes
}

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

@group(0) @binding(1)
var<uniform> config: Config;

@group(0) @binding(3)
var<storage> selections: array<SelectionInfo>;

@group(0) @binding(4)
var probability_curves: texture_2d_array<f32>;

@group(0) @binding(5)
var probability_sampler: sampler;

@fragment
fn main(
    @location(0) normal: vec2<f32>,
    @location(1) curve_pos: f32,
    @location(2) @interpolate(flat) instance_idx: u32
) -> @location(0) vec4<f32> {
    let alpha = get_line_alpha(normal);

    let selection = selections[instance_idx];

    let texture_position = vec2<f32>(curve_pos, 0.0);
    let texture_array_index = i32(selection.axis);
    let sample = textureSample(probability_curves, probability_sampler, texture_position, texture_array_index).r;

    if selection.use_color != 0u {
        let color = xyz_to_srgb(selection.color);
        return vec4<f32>(color * alpha, alpha);
    } else {
        let gradient = xyz_to_srgb(mix(config.low_color, config.high_color, sample));
        return vec4<f32>(gradient * alpha, alpha);
    }
}