/// Common WGSL shared among all Ruffle shaders.
/// Ruffle prepends this file onto every shader at runtime.

/// Global uniforms that are constant throughout a frame.
[[block]]
struct Globals {
    // The view matrix determined by the viewport and stage.
    view_matrix: mat4x4<f32>;
};

/// Transform uniforms that are changed per object.
[[block]]
struct Transforms {
    /// The world matrix that transforms this object into stage space.
    world_matrix: mat4x4<f32>;

    /// The multiplicative color transform of this object.
    mult_color: vec4<f32>;

    /// The additive color transform of this object.
    add_color: vec4<f32>;
};

/// Uniforms used by texture draws (bitmaps and gradients).
[[block]]
struct TextureTransforms {
    /// The transform matrix of the gradient or texture.
    /// Transforms from object space to UV space.
    matrix: mat4x4<f32>;
};

/// The vertex format shared among all shaders.
struct VertexInput {
    /// The position of the vertex in object space.
    [[location(0)]] position: vec2<f32>;

    /// The color of this vertex (only used by the color shader).
    [[location(1)]] color: vec4<f32>;
};

/// Common uniform layout shared by all shaders.
[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(1), binding(0)]]
var<uniform> transforms: Transforms;

/// Converts a color from linear to sRGB color space.
fn linear_to_srgb(linear: vec4<f32>) -> vec4<f32> {
    let a = 12.92 * linear.rgb;
    let b = 1.055 * pow(linear.rgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), linear.rgb);
    return vec4<f32>(mix(a, b, c), linear.a);
}

/// Converts a color from sRGB to linear color space.
fn srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    let a = srgb.rgb / 12.92;
    let b = pow((srgb.rgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let c = step(vec3<f32>(0.04045), srgb.rgb);
    return vec4<f32>(mix(a, b, c), srgb.a);
}
