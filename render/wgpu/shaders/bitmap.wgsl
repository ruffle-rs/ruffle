/// Shader used for drawing bitmap fills.

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[group(2), binding(0)]]
var<uniform> textureTransforms: TextureTransforms;
[[group(2), binding(1)]]
var texture: texture_2d<f32>;
[[group(3), binding(0)]]
var sampler: sampler;

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
    let matrix = textureTransforms.matrix;
    let uv = (mat3x3<f32>(matrix[0].xyz, matrix[1].xyz, matrix[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color: vec4<f32> = textureSample(texture, sampler, in.uv);
    // Texture is premultiplied by alpha.
    // Unmultiply alpha, apply color transform, remultiply alpha.
    if( color.a > 0.0 ) {
        color = vec4<f32>(color.rgb / color.a, color.a);
        color = color * transforms.mult_color + transforms.add_color;
        color = vec4<f32>(color.rgb * color.a, color.a);
    }
    let out = color;
    return output(out);
}
