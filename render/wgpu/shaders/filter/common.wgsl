#define_import_path filter
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

/// FIXME: We should import VertexInput from 'common', but a naga_oil bug prevents
/// us from importing 'common' in both this file and the invidual filter shaders (like 'color_matrix.wgsl')
/// Currently, importing 'common' in that way will cause all of the definitions from 'common' to be duplicated,
/// result in a WGSL parse error.
struct FilterVertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,
};

@vertex
fn main_vertex(in: FilterVertexInput) -> VertexOutput {
    #if use_push_constants == true
        var transforms = pc.transforms;
    #endif
    let matrix_ = textureTransforms.texture_matrix;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}
