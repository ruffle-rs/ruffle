#import filter

struct Filter {
    // Secretly a vec2<f32> but within alignment rules
    dir_x: f32,
    dir_y: f32,

    // Full size of the blur kernel (from left to right, ie)
    full_size: f32,

    // Half size of the blur kernel (from center to right, ie) - without center piece
    half_size: f32,
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

    let weight = 1.0 / filter_args.full_size;

    for (var i = 0.0; i < filter_args.full_size; i += 1.0) {
        color += textureSample(texture, texture_sampler, in.uv + direction * (i - filter_args.half_size)) * weight;
    }

    return color;
}
