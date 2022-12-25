/// This code is prepended to gradient/common.wgsl and is specialised for storage buffers

struct Gradient {
    colors: array<vec4<f32>,16u>,
    ratios: array<f32,16u>,
    gradient_type: i32,
    num_colors: u32,
    interpolation: i32,
    focal_point: f32,
};

@group(2) @binding(1) var<storage> gradient: Gradient;

fn ratio(i: u32) -> f32 {
    return gradient.ratios[i];
}