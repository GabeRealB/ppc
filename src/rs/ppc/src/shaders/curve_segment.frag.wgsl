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

@fragment
fn main(
    @location(0) xyz_color: vec4<f32>,
    @location(1) @interpolate(flat) discard_vertex: u32
) -> @location(0) vec4<f32> {
    if discard_vertex == 1u {
        discard;
    }

    let color = xyz_to_srgb(xyz_color.xyz);
    return vec4<f32>(color * xyz_color.a, xyz_color.a);
}