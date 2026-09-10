// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(1) @binding(0) var<uniform> transforms: array<common__Transforms, max_transforms>;
@group(2) @binding(0) var maskee_texture: texture_2d<f32>;
@group(2) @binding(1) var mask_texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@vertex
fn main_vertex(in: common__VertexInput, @builtin(instance_index) instanceIndex: u32) -> VertexOutput {
    let pos = common__globals.global_matrix * (transforms[instanceIndex].world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0));
    let uv = vec2<f32>(in.position.xy);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // dst is the maskee pixel we're masking
    var dst: vec4<f32> = textureSample(maskee_texture, texture_sampler, in.uv);
    // src is the mask pixel that we're using to mask
    var src: vec4<f32> = textureSample(mask_texture, texture_sampler, in.uv);

    return vec4<f32>(dst.rgb * src.a, dst.a * src.a);
}
