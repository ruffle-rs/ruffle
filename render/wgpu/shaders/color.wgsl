/// Shader used for drawing solid color fills.

#import common

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

#if use_push_constants == true
    var<push_constant> pc: common::PushConstants;
#else
    @group(1) @binding(0) var<uniform> transforms: common::Transforms;
    @group(2) @binding(0) var<uniform> colorTransforms: common::ColorTransforms;
#endif

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    #if use_push_constants == true
        var transforms = pc.transforms;
    #endif
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, in.color);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    #if use_push_constants == true
        var colorTransforms = pc.colorTransforms;
    #endif
    let color = saturate(in.color * colorTransforms.mult_color + colorTransforms.add_color);
    let alpha = saturate(color.a);
    return vec4<f32>(color.rgb * alpha, alpha);
}
