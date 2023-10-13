struct LineInfo {
    min_expanded_val: f32,
    _padding: f32,
    start_args: vec2<f32>,
    end_args: vec2<f32>,
    offset_start: vec2<f32>,
    offset_end: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read_write> output: array<LineInfo>;

@group(0) @binding(1)
var probability_curves: texture_2d_array<f32>;

const curve_segment_width: f32 = 0.2;

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

    let min_expanded_val = 1.0;
    let start_args = vec2<f32>(bitcast<f32>(curve_idx), f32(start_texel_pos) / f32(num_line_segments - 1u));
    let end_args = vec2<f32>(bitcast<f32>(curve_idx), f32(end_texel_pos) / f32(num_line_segments - 1u));
    let offset_start = vec2<f32>(mix(0.8, 0.05, start_texel) * curve_segment_width, 0.0);
    let offset_end = vec2<f32>(mix(0.8, 0.05, end_texel) * curve_segment_width, 0.0);

    output[global_id.x] = LineInfo(min_expanded_val, 0.0, start_args, end_args, offset_start, offset_end);
}