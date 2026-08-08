/// Shader used by `Graphics.drawTriangles` bitmap fills with UVT data.
/// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct TriangleVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uvt: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // u and v are multiplied by t before interpolation. Dividing in the
    // fragment shader restores Flash's perspective-correct UV mapping.
    @location(0) uvt: vec3<f32>,
};

@group(1) @binding(0) var<uniform> transforms: common__Transforms;
@group(2) @binding(0) var<uniform> textureTransforms: common__TextureTransforms;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@vertex
fn main_vertex(in: TriangleVertexInput) -> VertexOutput {
    let pos = common__globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, vec3<f32>(in.uvt.xy * in.uvt.z, in.uvt.z));
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uvt.xy / in.uvt.z;
    var color: vec4<f32> = textureSample(texture, texture_sampler, uv);
    // Texture is premultiplied by alpha.
    if (color.a > 0.0) {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = saturate(color * transforms.mult_color + transforms.add_color);
        color = vec4<f32>(color.rgb * color.a, color.a);
    }
    return color;
}
