/// Common WGSL shared among all Ruffle shaders.
/// Ruffle prepends this file onto every shader at runtime.

/// Global uniforms that are constant throughout a frame.
struct Globals {
    // The view matrix determined by the viewport and stage.
    view_matrix: mat4x4<f32>,
};

/// Transform uniforms that are changed per object.
struct Transforms {
    /// The world matrix that transforms this object into stage space.
    world_matrix: mat4x4<f32>,

    /// The multiplicative color transform of this object.
    mult_color: vec4<f32>,

    /// The additive color transform of this object.
    add_color: vec4<f32>,
};

/// Uniforms used by texture draws (bitmaps and gradients).
struct TextureTransforms {
    /// The transform matrix of the gradient or texture.
    /// Transforms from object space to UV space.
    matrix_: mat4x4<f32>,
};

/// The vertex format shared among all shaders.
struct VertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,

    /// The color of this vertex (only used by the color shader).
    @location(1) color: vec4<f32>,
};

/// Common uniform layout shared by all shaders.
@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var<uniform> transforms: Transforms;

/// Converts a color from linear to sRGB color space.
fn linear_to_srgb(linear_: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = linear_.rgb;
    if( linear_.a > 0.0 ) {
        rgb = rgb / linear_.a;
    }
    let a = 12.92 * rgb;
    let b = 1.055 * pow(rgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), rgb);
    return vec4<f32>(mix(a, b, c) * linear_.a, linear_.a);
}

/// Converts a color from sRGB to linear color space.
fn srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = srgb.rgb;
    if( srgb.a > 0.0 ) {
        rgb = rgb / srgb.a;
    }
    let a = rgb / 12.92;
    let b = pow((rgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let c = step(vec3<f32>(0.04045), rgb);
    return vec4<f32>(mix(a, b, c) * srgb.a, srgb.a);
}
