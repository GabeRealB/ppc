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

@group(0) @binding(1)
var<uniform> config: Config;

const red = vec3<f32>(1.0, 0.0, 0.0);

const feather: f32 = 0.5;
const one_minus_feather: f32 = 1.0 - feather;

@fragment
fn main(
    @location(0) normal: vec2<f32>,
    @location(1) @interpolate(flat) discard_line: u32,
    @location(2) @interpolate(flat) instance_idx: u32
) -> @location(0) vec4<f32> {
    if discard_line != 0u {
        discard;
    }

    let distance = length(normal);
    var alpha = 0.0;

    if distance <= one_minus_feather {
        alpha = 1.0;
    } else if distance <= 1.0 {
        let t = (distance - feather) / one_minus_feather;
        alpha = mix(1.0, 0.0, t);
    }

    var color = vec3<f32>(0.0);
    if config.color_mode == 0u {
        color = config.color;
    } else {
        color = config.color;
    }

    return vec4<f32>(color * alpha, alpha);
}