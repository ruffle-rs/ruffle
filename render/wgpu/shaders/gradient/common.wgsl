#define_import_path gradient
#import common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

#if use_push_constants == true
    var<push_constant> pc: common::PushConstants;
    @group(1) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
#else
    @group(1) @binding(0) var<uniform> transforms: common::Transforms;
    @group(2) @binding(0) var<uniform> colorTransforms: common::ColorTransforms;
    @group(3) @binding(0) var<uniform> textureTransforms: common::TextureTransforms;
#endif

struct Gradient {
    focal_point: f32,
    interpolation: i32,
};

#if use_push_constants == true
    @group(1) @binding(1) var<uniform> gradient: Gradient;
    @group(1) @binding(2) var texture: texture_2d<f32>;
    @group(1) @binding(3) var texture_sampler: sampler;
#else
    @group(3) @binding(1) var<uniform> gradient: Gradient;
    @group(3) @binding(2) var texture: texture_2d<f32>;
    @group(3) @binding(3) var texture_sampler: sampler;
#endif

fn find_t(focal_point: f32, uv: vec2<f32>) -> f32 {
    return 0.0;
}

struct GradientVertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,
};

@vertex
fn main_vertex(in: GradientVertexInput) -> VertexOutput {
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
    #if use_push_constants == true
        var colorTransforms = pc.colorTransforms;
    #endif

    // Calculate normalized `t` position in gradient, [0.0, 1.0] being the bounds of the ratios.
    var t: f32 = find_t(gradient.focal_point, in.uv);

    var color = textureSample(texture, texture_sampler, vec2<f32>(t, 0.0));
    if( gradient.interpolation != 0 ) {
        color = common::linear_to_srgb(color);
    }
    let out = saturate(color * colorTransforms.mult_color + colorTransforms.add_color);
    let alpha = saturate(out.a);
    return vec4<f32>(out.rgb * alpha, alpha);
}