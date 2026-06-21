// NOTE: The `common.wgsl` source is prepended to this before compilation.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) dst_uv: vec2<f32>,
    @location(1) src_uv: vec2<f32>,
};

struct BlendTransforms {
    world_matrix: mat4x4<f32>,
    dst_uv_transform: vec4<f32>,
    src_uv_transform: vec4<f32>,
};

@group(1) @binding(0) var<uniform> transforms: BlendTransforms;
@group(2) @binding(0) var parent_texture: texture_2d<f32>;
@group(2) @binding(1) var current_texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

fn apply_uv_transform(uv: vec2<f32>, t: vec4<f32>) -> vec2<f32> {
    let transformed = uv * t.zw + t.xy;
    return select(uv, transformed, any(t != vec4<f32>(0.0)));
}

@vertex
fn main_vertex(in: common__VertexInput) -> VertexOutput {
    let pos = common__globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 1.0, 1.0);
    let uv = vec2<f32>((pos.x + 1.0) / 2.0, -((pos.y - 1.0) / 2.0));
    let dst_uv = apply_uv_transform(uv, transforms.dst_uv_transform);
    let src_uv = apply_uv_transform(uv, transforms.src_uv_transform);
    return VertexOutput(pos, dst_uv, src_uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // dst is the parent pixel we're blending onto
    var dst: vec4<f32> = textureSample(parent_texture, texture_sampler, in.dst_uv);
    // src is the pixel that we want to apply
    var src: vec4<f32> = textureSample(current_texture, texture_sampler, in.src_uv);

    if (src.a > 0.0) {
        return vec4<f32>(dst.rgb * src.a, src.a * dst.a);
    } else {
        if (true) {
            // This needs to be in a branch because... reasons. Bug in naga.
            // https://github.com/gfx-rs/naga/issues/2168
            discard;
        }
        return dst;
    }
}
