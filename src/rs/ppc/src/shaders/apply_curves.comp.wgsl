@group(0) @binding(0)
var<storage, read_write> output: array<f32>;

@group(0) @binding(1)
var probability_curves: texture_2d_array<f32>;

@group(0) @binding(2)
var<storage, read> data: array<f32>;

@group(0) @binding(3)
var<uniform> num_datums: u32;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    if global_id.x >= arrayLength(&data) {
        return;
    }

    let value = data[global_id.x];
    let texture_idx = value * f32(textureDimensions(probability_curves).x - 1);
    let lower_texel_pos = i32(floor(texture_idx));
    let upper_texel_pos = i32(ceil(texture_idx));
    let t = fract(texture_idx);

    let axis_idx = i32(global_id.x / num_datums);
    let lower_texel = textureLoad(probability_curves, vec2<i32>(lower_texel_pos, 0), axis_idx, 0).r;
    let upper_texel = textureLoad(probability_curves, vec2<i32>(upper_texel_pos, 0), axis_idx, 0).r;

    let curve_value = mix(lower_texel, upper_texel, t);
    output[global_id.x] = curve_value;
}