@group(0) @binding(0)
var<storage, read_write> output: array<f32>;

@group(0) @binding(1)
var<storage, read> input: array<f32>;

@group(0) @binding(2)
var<uniform> num_datums: u32;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    if global_id.x >= arrayLength(&output) {
        return;
    }

    let start = global_id.x % num_datums;

    // Use a simple for loop for the reduction. Isn't really efficient,
    // but may not even make that much of a difference given, that we
    // expect `num_datums` to be big.
    let iterations = arrayLength(&input) / arrayLength(&output);
    var partial_mul = 1.0;
    for (var i = 0u; i < iterations; i++) {
        partial_mul *= input[start + (i * num_datums)];
    }

    output[global_id.x] = partial_mul;
}