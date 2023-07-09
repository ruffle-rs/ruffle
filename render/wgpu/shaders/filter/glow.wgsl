#import filter

struct Filter {
    color: vec4<f32>,
    strength: f32,
    inner: u32,
    knockout: u32,
    _pad: f32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> filter_args: Filter;
@group(0) @binding(3) var blurred: texture_2d<f32>;

@vertex
fn main_vertex(in: filter::VertexInput) -> filter::VertexOutput {
    return filter::main_vertex(in);
}

@fragment
fn main_fragment(in: filter::VertexOutput) -> @location(0) vec4<f32> {
    let inner = filter_args.inner > 0u;
    let knockout = filter_args.knockout > 0u;
    let blur = textureSample(blurred, texture_sampler, in.uv).a;
    let dest = textureSample(texture, texture_sampler, in.uv); // TODO: The UVs may not match when we resize the blur texture

    // [NA] It'd be nice to use hardware blending but the operation is too complex :( Only knockouts would work.

    // Start with 1 alpha because we'll be multiplying the whole thing
    var color = vec4<f32>(filter_args.color.r, filter_args.color.g, filter_args.color.b, 1.0);
    if (inner) {
        let alpha = filter_args.color.a * (1.0 - blur) * filter_args.strength;
        if (knockout) {
            color = color * alpha * dest.a;
        } else {
            color = color * alpha * dest.a + dest * (1.0 - alpha);
        }
    } else {
        let alpha = filter_args.color.a * blur * filter_args.strength;
        if (knockout) {
            color = color * alpha * (1.0 - dest.a);
        } else {
            color = color * alpha * (1.0 - dest.a) + dest;
        }
    }

    return color;
}
