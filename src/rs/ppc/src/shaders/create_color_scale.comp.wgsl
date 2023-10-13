struct ScaleElement {
    t: f32,
    // padding: 12 bytes
    color: vec4<f32>,
}

@group(0) @binding(0)
var color_scale: texture_storage_2d<rgba32float, write>;

@group(0) @binding(1)
var<storage> scale: array<ScaleElement>;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let num_samples = u32(textureDimensions(color_scale).x);

    if global_id.x >= num_samples {
        return;
    }

    // The position will always be in [0, 1].
    let t_value = f32(global_id.x) / f32(num_samples);
    let texture_idx = vec2<i32>(i32(global_id.x), 0);

    // Search for the upper and lower color.
    var idx = 0u;
    while idx < arrayLength(&scale) {
        if scale[idx].t >= t_value {
            break;
        }

        idx++;
    }

    if scale[idx].t == t_value {
        textureStore(color_scale, texture_idx, scale[idx].color);
        return;
    }

    let upper = scale[idx];
    let lower = scale[idx - 1u];

    let t = (t_value - lower.t) / (upper.t - lower.t);
    let color = mix(lower.color, upper.color, t);
    textureStore(color_scale, texture_idx, color);
}