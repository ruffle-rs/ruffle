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

#if use_storage_buffers == true
    struct Gradient {
        colors: array<vec4<f32>,16u>,
        ratios: array<f32,16u>,
        gradient_type: i32,
        num_colors: u32,
        interpolation: i32,
        focal_point: f32,
    };

    #if use_push_constants == true
        @group(1) @binding(1) var<storage> gradient: Gradient;
    #else
        @group(3) @binding(1) var<storage> gradient: Gradient;
    #endif

    fn ratio(i: u32) -> f32 {
        return gradient.ratios[i];
    }
#else
    struct Gradient {
        colors: array<vec4<f32>, 16>,
        ratios: array<vec4<f32>, 4u>, // secretly array<f32; 16> but this let's us squeeze it into alignment
        gradient_type: i32,
        num_colors: u32,
        interpolation: i32,
        focal_point: f32,
    };

    #if use_push_constants == true
        @group(1) @binding(1) var<uniform> gradient: Gradient;
    #else
        @group(3) @binding(1) var<uniform> gradient: Gradient;
    #endif

    fn ratio(i: u32) -> f32 {
        return gradient.ratios[i / 4u][i % 4u];
    }
#endif

fn find_t(focal_point: f32, uv: vec2<f32>) -> f32 {
    return 0.0;
}

@vertex
fn main_vertex(in: common::VertexInput) -> VertexOutput {
    #if use_push_constants == true
        var transforms = pc.transforms;
    #endif
    let matrix_ = textureTransforms.matrix_;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = common::globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    #if use_push_constants == true
        var colorTransforms = pc.colorTransforms;
    #endif
    let last = gradient.num_colors - 1u;

    // Calculate normalized `t` position in gradient, [0.0, 1.0] being the bounds of the ratios.
    var t: f32 = find_t(gradient.focal_point, in.uv);

    #if gradient_repeat_mode == 1
        // Mirror
        if( t < 0.0 ) {
            t = -t;
        }
        if ( (i32(t) & 1) == 0 ) {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    #endif
    #if gradient_repeat_mode == 2
        // Repeat
        t = fract(t);
    #endif
    #if gradient_repeat_mode == 3
        // Clamp
        t = clamp(t, 0.0, 1.0);
    #endif

    t = clamp(t, ratio(0u), ratio(last));

    // Find the two gradient colors bordering our position.
    var j: u32;
    for( j = 1u; t > ratio(j); j = j + 1u) {
        // Noop
    }
    let i = j - 1u;

    // Lerp between the two colors.
    let a = (t - ratio(i)) / (ratio(j) - ratio(i));
    var color: vec4<f32> = mix(gradient.colors[i], gradient.colors[j], a);
    if( gradient.interpolation != 0 ) {
        color = common::linear_to_srgb(color);
    }
    let out = color * colorTransforms.mult_color + colorTransforms.add_color;
    let alpha = clamp(out.a, 0.0, 1.0);
    return vec4<f32>(out.rgb * alpha, alpha);
}