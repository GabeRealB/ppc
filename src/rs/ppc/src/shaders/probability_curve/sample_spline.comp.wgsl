struct SplineSegment {
    coefficients: vec4<f32>,
    bounds: vec2<f32>,
    t_range: vec2<f32>,
}

@group(0) @binding(0)
var probability_curves: texture_storage_2d<r32float, write>;

@group(0) @binding(1)
var<storage> spline: array<SplineSegment>;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let num_samples = u32(textureDimensions(probability_curves).x);

    if global_id.x >= num_samples {
        return;
    }

    // The position will always be in [0, 1].
    let arc_position = f32(global_id.x) / f32(num_samples);

    // Search for the segment that contains the position.
    var idx = 0u;
    while idx < arrayLength(&spline) {
        if spline[idx].bounds.x <= arc_position && spline[idx].bounds.y >= arc_position {
            break;
        }

        idx++;
    }

    // Check that we found the right segment.
    let segment = spline[idx];

    let t_min = segment.t_range.x;
    let t_max = segment.t_range.y;

    // Transform our position on the spline to a local segment position in [t_min, t_max].
    let t = mix(t_min, t_max, (arc_position - segment.bounds.x) / (segment.bounds.y - segment.bounds.x));
    let t_squared = t * t;
    let t_cubed = t_squared * t;

    let ts = vec4<f32>(t_cubed, t_squared, t, 1.0);     // [t^3, t^2, t^1, 1]
    let tmp = segment.coefficients * ts;                // [a * t^3, b * t^2, c * t^1, d]
    let value = saturate(dot(tmp, vec4<f32>(1.0)));           // a * t^3 + b * t^2 + c * t^1 + d

    let texture_idx = vec2<i32>(i32(global_id.x), 0);
    let sample = vec4<f32>(value, 0.0, 0.0, 1.0);
    textureStore(probability_curves, texture_idx, sample);
}