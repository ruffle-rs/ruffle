#import gradient
#import common

override fn gradient::find_t(focal_point: f32, uv: vec2<f32>) -> f32 {
    return uv.x;
}

@vertex
fn main_vertex(in: common::VertexInput) -> gradient::VertexOutput {
    return gradient::main_vertex(in);
}

@fragment
fn main_fragment(in: gradient::VertexOutput) -> @location(0) vec4<f32> {
    return gradient::main_fragment(in);
}