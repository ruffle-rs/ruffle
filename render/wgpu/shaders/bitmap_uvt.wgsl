// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texture_coords: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec3<f32>,
};

@group(1) @binding(0) var<uniform> transforms: common__Transforms;
@group(2) @binding(0) var<uniform> textureTransforms: common__TextureTransforms;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

override late_saturate: bool = false;

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    let position = common__globals.view_matrix * transforms.world_matrix *
        vec4<f32>(in.position, 0.0, 1.0);
    return VertexOutput(position, in.texture_coords);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.texture_coords.z == 0.0) {
        discard;
    }
    let uv = in.texture_coords.xy / in.texture_coords.z;
    var color: vec4<f32> = textureSample(texture, texture_sampler, uv);
    // Texture is premultiplied by alpha.
    // Unmultiply alpha, apply color transform, remultiply alpha.
    if (color.a > 0.0) {
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
