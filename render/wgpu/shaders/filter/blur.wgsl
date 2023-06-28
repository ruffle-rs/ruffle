#import filter

struct Filter {
    // Secretly a vec2<f32> but within alignment rules
    dir_x: f32,
    dir_y: f32,

    // Size of the blur kernel
    size: f32,

    _padding: f32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> filter_args: Filter;

@vertex
fn main_vertex(in: filter::VertexInput) -> filter::VertexOutput {
    return filter::main_vertex(in);
}

@fragment
fn main_fragment(in: filter::VertexOutput) -> @location(0) vec4<f32> {
    let direction = vec2<f32>(filter_args.dir_x, filter_args.dir_y);
    var color = vec4<f32>();

    let num_samples = 2.0 * filter_args.size + 1.0;
    let weight = 1.0 / num_samples;

    for (var i = 0.0; i < num_samples; i += 1.0) {
        color += textureSample(texture, texture_sampler, in.uv + direction * (i - filter_args.size)) * weight;
    }

    return color;
}
