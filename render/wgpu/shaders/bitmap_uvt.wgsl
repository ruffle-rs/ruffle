/// Shader used for drawing bitmap fills.
/// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uvt: vec3<f32>,
};

@group(1) @binding(0) var<uniform> transforms: common__Transforms;
// This is unused, but we bind it anyway to minimize the diff from 'bitmap.wgsl'
@group(2) @binding(0) var<uniform> textureTransforms: common__TextureTransforms;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
override late_saturate: bool = false;

struct VertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,
    @location(1) uvt: vec3<f32>,
};

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    var pos = common__globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0, 1.0);
    pos /= in.uvt.z;
    return VertexOutput(pos, in.uvt);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, vec2<f32>(in.uvt.x, in.uvt.y));
    // Texture is premultiplied by alpha.
    // Unmultiply alpha, apply color transform, remultiply alpha.
    if( color.a > 0.0 ) {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = color * transforms.mult_color + transforms.add_color;
        if (!late_saturate) {
            color = saturate(color);
        }
        color = vec4<f32>(color.rgb * color.a, color.a);
        if (late_saturate) {
            color = saturate(color);
        }
    }
    return color;
}
