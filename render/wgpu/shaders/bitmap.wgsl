/// Shader used for drawing bitmap fills.
/// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(1) @binding(0) var<uniform> transforms: common__Transforms;
@group(2) @binding(0) var<uniform> textureTransforms: common__TextureTransforms;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
override late_saturate: bool = false;

@vertex
fn main_vertex(in: common__VertexInput) -> VertexOutput {
    let matrix_ = textureTransforms.texture_matrix;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = common__globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, in.uv);
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
