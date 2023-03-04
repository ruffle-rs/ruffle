/// Shader used for drawing bitmap fills.

#import common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

#if use_push_constants == true
    var<push_constant> pc: common::PushConstants;
    @group(1) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
    @group(1) @binding(1) var texture: texture_2d<f32>;
    @group(1) @binding(2) var texture_sampler: sampler;
#else
    @group(1) @binding(0) var<uniform> transforms: common::Transforms;
    @group(2) @binding(0) var<uniform> colorTransforms: common::ColorTransforms;
    @group(3) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
    @group(3) @binding(1) var texture: texture_2d<f32>;
    @group(3) @binding(2) var texture_sampler: sampler;
#endif

@vertex
fn main_vertex(in: common::VertexInput) -> VertexOutput {
    #if use_push_constants == true
        var transforms = pc.transforms;
    #endif
    let matrix_ = textureTransforms.texture_matrix;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, in.uv);
    #if use_push_constants == true
        var colorTransforms = pc.colorTransforms;
    #endif
    // Texture is premultiplied by alpha.
    // Unmultiply alpha, apply color transform, remultiply alpha.
    if( color.a > 0.0 ) {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = color * colorTransforms.mult_color + colorTransforms.add_color;
        // Apply 'saturate' *after* we re-multiply. If we get a non-premultiplied
        // color like (1.0, 1.0. 1.0, 0.5) from a Stage3D texture, we want
        // this shader to have no effect if we're applying a no-op colorTransforms.
        // By applying the saturation after dividing and multiply by the alpha,
        // we avoid saturing if the intermediate 'unmultiplied' color is temporarily
        // out of range.
        let alpha = saturate(color.a);
        color = vec4<f32>(saturate(color.rgb * alpha), alpha);
    }
    return color;
}
