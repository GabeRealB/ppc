@group(0) @binding(0)
var color_scale: texture_2d<f32>;

@group(0) @binding(1)
var color_scale_transformed: texture_storage_2d<rgba32float, write>;

// 0 = sRgb Linear
// 1 = Xyz
// 2 = CieLab
// 3 = CieLch
@group(0) @binding(2)
var<uniform> color_space: u32;

@compute @workgroup_size(64)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let num_samples = u32(textureDimensions(color_scale).x);
    if global_id.x >= num_samples {
        return;
    }

    let sample_idx = vec2<i32>(i32(global_id.x), 0);
    let sample = textureLoad(color_scale, sample_idx, 0);

    var transformed = vec4(0.0);
    if color_space == 0u {
        transformed = srgb_to_xyz(sample);
    } else if color_space == 1u {
        transformed = xyz_to_xyz(sample);
    } else if color_space == 2u {
        transformed = cie_lab_to_xyz(sample);
    } else if color_space == 3u {
        transformed = cie_lch_to_xyz(sample);
    }

    textureStore(color_scale_transformed, sample_idx, sample);
}

const SRGB_XYZ_CONVERSION_MATRIX = mat3x3<f32>(
    vec3<f32>(0.4124108464885388, 0.21264934272065283, 0.019331758429150258),
    vec3<f32>(0.3575845678529519, 0.7151691357059038, 0.11919485595098397),
    vec3<f32>(0.18045380393360833, 0.07218152157344333, 0.9503900340503373),
);

fn srgb_to_xyz(rgba: vec4<f32>) -> vec4<f32> {
    let rgb = rgba.rgb;
    let xyz = SRGB_XYZ_CONVERSION_MATRIX * rgb;

    return vec4(xyz, rgba.a);
}

fn xyz_to_xyz(xyza: vec4<f32>) -> vec4<f32> {
    return xyza;
}

const CBRT_EPSILON: f32 = 6.0 / 29.0;
const KAPPA: f32 = 24389.0 / 27.0;
fn cie_lab_to_xyz(laba: vec4<f32>) -> vec4<f32> {
    let l = laba.r;
    let a = laba.g;
    let b = laba.b;

    let fy = (l + 16.0) / 116.0;
    let fx = (a / 500.0) + fy;
    let fz = fy - (b / 200.0);

    let fxz = vec2<f32>(fx, fz);
    let branch_false = (fxz * 116.0 - 16.0) / KAPPA;
    let branch_true = pow(fxz, vec2(3.0));
    let selection = fxz > vec2(CBRT_EPSILON);
    let xz = select(branch_false, branch_true, selection);

    let x = xz.r;
    let y = select(b / KAPPA, pow(fy, 3.0), l > 8.0);
    let z = xz.g;

    return vec4(x, y, z, laba.a);
}

fn cie_lch_to_xyz(lcha: vec4<f32>) -> vec4<f32> {
    let l = lcha.r;
    let c = lcha.g;
    let h = lcha.b;

    let h_rad = radians(h);

    let a = c * cos(h_rad);
    let b = c * sin(h_rad);
    let laba = vec4<f32>(l, a, b, lcha.a);

    return cie_lab_to_xyz(laba);
}
