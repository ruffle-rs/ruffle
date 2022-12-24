/// This code is prepended to gradient/common.wgsl and is specialised for uniform buffers

struct Gradient {
    colors: array<vec4<f32>, 16>,
    ratios: array<vec4<f32>, 4u>, // secretly array<f32; 16> but this let's us squeeze it into alignment
    gradient_type: i32,
    num_colors: u32,
    repeat_mode: i32,
    interpolation: i32,
    focal_point: f32,
};

@group(2) @binding(1) var<uniform> gradient: Gradient;

fn ratio(i: u32) -> f32 {
    return gradient.ratios[i / 4u][i % 4u];
}
