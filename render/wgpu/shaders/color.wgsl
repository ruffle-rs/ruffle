/// Shader used for drawing solid color fills.

#import common

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
fn main_vertex(in: common::VertexInput) -> VertexOutput {
    #if use_push_constants == true
        var transforms = pc.transforms;
    #endif
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, in.color);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    #if use_push_constants == true
        var colorTransforms = pc.colorTransforms;
    #endif
    if( color.a > 0.0 ) {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = color * colorTransforms.mult_color + colorTransforms.add_color;
        let alpha = clamp(color.a, 0.0, 1.0);
        color = vec4<f32>(color.rgb * alpha, alpha);
    }
    return color;
}
