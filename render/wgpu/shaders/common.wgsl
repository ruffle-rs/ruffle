/// Common WGSL shared among many Ruffle shaders.
/// This is prepended to a lot of shaders at runtime before compiling them.
/// The `common__` identifier prefix serves as pseudo-namespacing.

/// Global uniforms that are constant throughout a frame.
struct common__Globals {
    // The view matrix determined by the viewport and stage.
    view_matrix: mat4x4<f32>,
};

/// Transform uniforms that are changed per object.
struct common__Transforms {
    /// The world matrix that transforms this object into stage space.
    world_matrix: mat4x4<f32>,

    /// The multiplicative color transform of this object.
    mult_color: vec4<f32>,

    /// The additive color transform of this object.
    add_color: vec4<f32>,
};

/// Uniforms used by texture draws (bitmaps and gradients).
struct common__TextureTransforms {
    /// The transform matrix of the gradient or texture.
    /// Transforms from object space to UV space.
    texture_matrix: mat4x4<f32>,
};

/// The vertex format shared among most shaders.
struct common__VertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,
};

/// Common uniform layout shared by all shaders.
@group(0) @binding(0) var<uniform> common__globals: common__Globals;

/// Converts a color from linear to sRGB color space.
fn common__linear_to_srgb(linear_: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = linear_.rgb;
    if (linear_.a > 0.0) {
        rgb = rgb / linear_.a;
    }
    let a = 12.92 * rgb;
    let b = 1.055 * pow(rgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), rgb);
    return vec4<f32>(mix(a, b, c) * linear_.a, linear_.a);
}

/// Converts a color from sRGB to linear color space.
fn common__srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = srgb.rgb;
    if (srgb.a > 0.0) {
        rgb = rgb / srgb.a;
    }
    let a = rgb / 12.92;
    let b = pow((rgb + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let c = step(vec3<f32>(0.04045), rgb);
    return vec4<f32>(mix(a, b, c) * srgb.a, srgb.a);
}
