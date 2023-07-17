#import filter

struct Filter {
    // Secretly a vec2<f32> but within alignment rules
    dir_x: f32,
    dir_y: f32,

    // Full size of the blur kernel (from left to right, ie). Must be a whole integer.
    full_size: f32,

    // Weight of the left most sample (odd width has a weight of 1, even width has 0.5)
    left_weight: f32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

#if use_push_constants == true
    var<push_constant> filter_args: Filter;
#else
    @group(0) @binding(2) var<uniform> filter_args: Filter;
#endif

@vertex
fn main_vertex(in: filter::VertexInput) -> filter::VertexOutput {
    var result = filter::main_vertex(in);

    // Shift the uv over to the leftmost sample
    let direction = vec2<f32>(filter_args.dir_x, filter_args.dir_y);
    result.uv -= (direction * floor(filter_args.full_size / 2.0));

    return result;
}

@fragment
fn main_fragment(in: filter::VertexOutput) -> @location(0) vec4<f32> {
    let direction = vec2<f32>(filter_args.dir_x, filter_args.dir_y);

    // We always start off with the left edge. Everything else is optional.
    var center_length = filter_args.full_size - filter_args.left_weight;
    var total = textureSample(texture, texture_sampler, in.uv) * filter_args.left_weight;

    if (filter_args.full_size % 2.0 == 0.0) {
        // If the width is even, we have a right edge of a fixed weight and offset
        center_length -= 1.5;
        total += textureSample(texture, texture_sampler, in.uv + (direction * (filter_args.full_size - 0.75))) * 1.5;
    }

    // At this point, the center_length must be a whole number, divisible by 2.
    var center = vec4<f32>();
    for (var i = 0.0; i < center_length; i += 2.0) {
        // The center of the kernel is always going to be 1,1 weight pairs. We can just sample between the two pixels.
        center += textureSample(texture, texture_sampler, in.uv + (direction * (1.5 + i)));
    }
    total += center * 2.0;

    // The sum of every weight is full_size
    return total / filter_args.full_size;
}
