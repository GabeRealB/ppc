struct Config {
    line_width: vec2<f32>,
    selection_threshold: f32,
    // padding: 4 bytes
    unselected_color: vec4<f32>,
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

@group(0) @binding(4)
var<storage, read> color_values: array<f32>;

@group(0) @binding(5)
var<storage, read> probabilities: array<f32>;

@group(0) @binding(6)
var color_scale: texture_2d<f32>;

@group(0) @binding(7)
var color_scale_sampler: sampler;

@fragment
fn main(
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

    let texture_position = vec2<f32>(probability, 0.0);
    let color_scale_color = textureSample(color_scale, color_scale_sampler, texture_position);

    let color_selection = vec4<bool>(probability >= config.selection_threshold);
    let color = select(config.unselected_color, color_scale_color, color_selection);

    let color_alpha = color.a;
    let color_srgb = xyz_to_srgb(color.rgb);


    return vec4<f32>(color_srgb * alpha * color_alpha, alpha * color_alpha);
}