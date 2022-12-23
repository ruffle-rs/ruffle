/// Shader used for drawing a pending framebuffer onto a parent framebuffer

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BlendOptions {
    mode: i32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

@group(2) @binding(0) var parent_texture: texture_2d<f32>;
@group(2) @binding(1) var current_texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var<uniform> blend: BlendOptions;

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    let pos = globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 1.0, 1.0);
    let uv = vec2<f32>((pos.x + 1.0) / 2.0, -((pos.y - 1.0) / 2.0));
    return VertexOutput(pos, uv);
}

fn blend_func(src: vec3<f32>, dst: vec3<f32>) -> vec3<f32> {
    switch (blend.mode) {
        case 2: { // Screen
            return (dst + src) - (dst * src);
        }
        case 3: { // Lighten
            return max(dst, src);
        }
        case 4: { // Darken
            return min(dst, src);
        }
        case 5: { // Difference
            return abs(dst - src);
        }
        case 8: { // Invert
            return 1.0 - dst;
        }
        case 11: { // Overlay
            var out = src;
            if (dst.r <= 0.5) { out.r = (2.0 * src.r * dst.r); } else { out.r = (1.0 - 2.0 * (1.0 - dst.r) * (1.0 - src.r)); }
            if (dst.g <= 0.5) { out.g = (2.0 * src.g * dst.g); } else { out.g = (1.0 - 2.0 * (1.0 - dst.g) * (1.0 - src.g)); }
            if (dst.b <= 0.5) { out.b = (2.0 * src.b * dst.b); } else { out.b = (1.0 - 2.0 * (1.0 - dst.b) * (1.0 - src.b)); }
            return out;
        }
        case 12: { // Hardlight
            var out = src;
            if (src.r <= 0.5) { out.r = (2.0 * src.r * dst.r); } else { out.r = (1.0 - 2.0 * (1.0 - dst.r) * (1.0 - src.r)); }
            if (src.g <= 0.5) { out.g = (2.0 * src.g * dst.g); } else { out.g = (1.0 - 2.0 * (1.0 - dst.g) * (1.0 - src.g)); }
            if (src.b <= 0.5) { out.b = (2.0 * src.b * dst.b); } else { out.b = (1.0 - 2.0 * (1.0 - dst.b) * (1.0 - src.b)); }
            return out;
        }
        default: {
            return src;
        }
    }
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // dst is the parent pixel we're blending onto
    var dst: vec4<f32> = textureSample(parent_texture, texture_sampler, in.uv);
    // src is the pixel that we want to apply
    var src: vec4<f32> = textureSample(current_texture, texture_sampler, in.uv);

    if (src.a > 0.0) {
        if (blend.mode == 9) { // Alpha
            return vec4<f32>(dst.rgb * src.a, src.a * dst.a);
        } else if (blend.mode == 10) { // Erase
            return vec4<f32>(dst.rgb * (1.0 - src.a), (1.0 - src.a) * dst.a);
        } else {
            return vec4<f32>(src.rgb * (1.0 - dst.a) + dst.rgb * (1.0 - src.a) + src.a * dst.a * blend_func(src.rgb / src.a, dst.rgb/ dst.a), src.a + dst.a * (1.0 - src.a));
        }
    } else {
        if (true) {
            // This needs to be in a branch because... reasons. Bug in naga.
            // https://github.com/gfx-rs/naga/issues/2168
            discard;
        }
        return dst;
    }
}
