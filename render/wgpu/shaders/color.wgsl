/// Shader used for drawing solid color fills.

#import common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

#if use_push_constants == true
    var<push_constant> transforms: common::Transforms;
#else
    @group(1) @binding(0) var<uniform> transforms: common::Transforms;
#endif

@group(2) @binding(0) var<uniform> colorTransforms: common::ColorTransforms;

@vertex
fn main_vertex(in: common::VertexInput) -> VertexOutput {
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    let color = in.color * colorTransforms.mult_color + colorTransforms.add_color;
    let alpha = clamp(color.a, 0.0, 1.0);
    return VertexOutput(pos, vec4<f32>(color.rgb * alpha, alpha));
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
