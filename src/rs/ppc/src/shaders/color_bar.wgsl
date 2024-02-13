struct ColorBarBounds {
    start: f32,
    end: f32,
}

@group(0) @binding(0)
var color_scale: texture_2d<f32>;

@group(0) @binding(1)
var<uniform> color_scale_bounds: ColorBarBounds;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color_scale_value: f32,
}

const OUT_OF_BOUNDS_FACTOR = 0.4;
// const INDEX_BUFFER = array<u32, 6>(0u, 1u, 2u, 1u, 3u, 2u);
// const VERTEX_BUFFER = array<vec2<f32>, 4>(
//     vec2<f32>(-1.0, -1.0),
//     vec2<f32>(1.0, -1.0),
//     vec2<f32>(-1.0, 1.0),
//     vec2<f32>(1.0, 1.0),
// );

const XYZ_SRGB_CONVERSION_MATRIX = mat3x3<f32>(
    vec3<f32>(3.240812398895283, -0.9692430170086407, 0.055638398436112804),
    vec3<f32>(-1.5373084456298136, 1.8759663029085742, -0.20400746093241362),
    vec3<f32>(-0.4985865229069666, 0.04155503085668564, 1.0571295702861434),
);

fn xyz_to_srgb(color: vec3<f32>) -> vec3<f32> {
    let linear_srgb = XYZ_SRGB_CONVERSION_MATRIX * color.xyz;
    let a = 12.92 * linear_srgb;
    let b = 1.055 * pow(linear_srgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), linear_srgb);
    let srgb = mix(a, b, c);
    return srgb;
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    var INDEX_BUFFER = array<u32, 6>(0u, 1u, 2u, 1u, 3u, 2u);
    var VERTEX_BUFFER = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, 1.0),
    );

    let index = INDEX_BUFFER[vertex_idx];
    let vertex = VERTEX_BUFFER[index];

    let position = vec4<f32>(vertex, 0.0, 1.0);
    let color_scale_value = select(0.0, 1.0, index >= 2u);
    return VertexOutput(position, color_scale_value);
}

@fragment
fn fragment_main(@location(0) color_scale_value: f32) -> @location(0) vec4<f32> {
    let sample_in_bounds_0 = color_scale_bounds.start <= color_scale_value;
    let sample_in_bounds_1 = color_scale_value <= color_scale_bounds.end;
    let sample_in_bounds = sample_in_bounds_0 && sample_in_bounds_1;

    let num_samples = textureDimensions(color_scale).x;
    let sample_position = color_scale_value * f32(num_samples - 1u);
    let sample_1 = i32(floor(sample_position));
    let sample_2 = i32(ceil(sample_position));
    let t = fract(sample_position);

    let color_1 = textureLoad(color_scale, vec2(sample_1, 0), 0);
    let color_2 = textureLoad(color_scale, vec2(sample_2, 0), 0);
    let color = mix(color_1, color_2, t);

    let alpha_factor = select(OUT_OF_BOUNDS_FACTOR, 1.0, sample_in_bounds);
    let color_alpha = color.a * alpha_factor;
    let color_srgb = xyz_to_srgb(color.rgb);
    return vec4<f32>(color_srgb * color_alpha, color_alpha);
}