struct CurveLineInfo {
    x_t_values: vec2<f32>,
    y_t_values: vec2<f32>,
    axis: u32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<CurveLineInfo>;

@group(0) @binding(1)
var probability_curves: texture_2d_array<f32>;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    if global_id.x >= arrayLength(&output) {
        return;
    }

    let num_line_segments = u32(textureDimensions(probability_curves).x);
    let curve_idx = i32(global_id.x / (num_line_segments - 1u));
    let start_texel_pos = i32(global_id.x) % i32(num_line_segments - 1u);
    let end_texel_pos = start_texel_pos + 1;

    let start_texel = textureLoad(probability_curves, vec2<i32>(start_texel_pos, 0), curve_idx, 0).r;
    let end_texel = textureLoad(probability_curves, vec2<i32>(end_texel_pos, 0), curve_idx, 0).r;

    let x_t_values = mix(vec2<f32>(0.1), vec2<f32>(0.95), vec2<f32>(start_texel, end_texel));
    let y_t_values = vec2<f32>(
        f32(start_texel_pos) / f32(num_line_segments - 1u),
        f32(end_texel_pos) / f32(num_line_segments - 1u),
    );
    let axis = u32(curve_idx);

    output[global_id.x] = CurveLineInfo(x_t_values, y_t_values, axis);
}