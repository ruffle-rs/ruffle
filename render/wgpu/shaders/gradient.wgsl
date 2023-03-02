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
    shape: i32,
    repeat: i32,
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

fn find_t(uv: vec2<f32>) -> f32 {
    if (gradient.shape == 1) {
        // linear
        return uv.x;
    } if (gradient.shape == 2) {
        // radial
        return length(uv * 2.0 - 1.0);
    } else {
        // focal
        let uv = uv * 2.0 - 1.0;
        var d: vec2<f32> = vec2<f32>(gradient.focal_point, 0.0) - uv;
        let l = length(d);
        d = d / l;
        return l / (sqrt(1.0 - gradient.focal_point * gradient.focal_point * d.y * d.y) + gradient.focal_point * d.x);
    }
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    #if use_push_constants == true
        var colorTransforms = pc.colorTransforms;
    #endif

    // Calculate normalized `t` position in gradient, [0.0, 1.0] being the bounds of the ratios.
    var t: f32 = find_t(in.uv);

    if (gradient.repeat == 1) {
        // Pad
        t = saturate(t);
    } else if (gradient.repeat == 2) {
        // Reflect
        if( t < 0.0 ) {
            t = -t;
        }
        if ( (i32(t) & 1) == 0 ) {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    } else if (gradient.repeat == 3) {
        // Repeat
        t = fract(t);
    }

    var color = textureSample(texture, texture_sampler, vec2<f32>(t, 0.0));
    if( gradient.interpolation != 0 ) {
        color = common::linear_to_srgb(color);
    }
    let out = saturate(color * colorTransforms.mult_color + colorTransforms.add_color);
    let alpha = saturate(out.a);
    return vec4<f32>(out.rgb * alpha, alpha);
}