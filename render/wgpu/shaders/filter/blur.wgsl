// NOTE: The `shader_filter_common.wgsl` source is prepended to this before compilation.

struct Filter {
    // Secretly a vec2<f32> but within alignment rules.
    // One of these is always 0.0, the other is 1.0.
    dir_x: f32,
    dir_y: f32,

    // Full size of the blur kernel.
    full_size: f32,

    // The number of trivially sampleable pixel pairs in the middle.
    m: f32,

    // This is m * 2.0: # of trivially sampleable pixels (not pairs) in the middle.
    m2: f32,

    // The weight of the first sampled pixel - computed as alpha in Rust.
    first_weight: f32,

    // These control the fused sampling of the last pixel pair.
    last_offset: f32,
    last_weight: f32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

@group(0) @binding(2) var<uniform> filter_args: Filter;

@vertex
fn main_vertex(in: filter__VertexInput) -> filter__VertexOutput {
    var result = filter__main_vertex(in);

    let direction = vec2<f32>(filter_args.dir_x, filter_args.dir_y);
    // Pre-shifting the UV coords to put the center of the first trivially
    // sampled (with 1.0 weight, "in the center") pixel at 0.0.
    result.uv -= direction * filter_args.m;

    return result;
}

@fragment
fn main_fragment(in: filter__VertexOutput) -> @location(0) vec4<f32> {
    let direction = vec2<f32>(filter_args.dir_x, filter_args.dir_y);

    var total = vec4<f32>(0.0);

    // The first (potentially fractional) pixel, to the left of the trivial pixel pairs.
    total += textureSample(texture, texture_sampler, in.uv - direction) * filter_args.first_weight;

    var center = vec4<f32>();
    for (var i = 0.5; i < filter_args.m2; i += 2.0) {
        // The center of the kernel is always going to be 1,1 weight pairs.
        // We can just sample between the two pixels and multiply by 2.0.
        // The +0.5 offset is baked right into i. This doesn't affect the
        // iteration (which has a granularity of 2.0, and is open-ended),
        // but saves an addition here in the loop body.
        center += textureSample(texture, texture_sampler, in.uv + direction * i);
    }
    total += center * 2.0;

    // The last pixel pair, the second of which may have fractional weight, sampled together.
    let last_location = in.uv + direction * (filter_args.m2 + filter_args.last_offset);
    total += textureSample(texture, texture_sampler, last_location) * filter_args.last_weight;

    // The sum of every weight is full_size.
    let result = total / filter_args.full_size;

    // This rounding imitates the fixed-point computations of FP, improving emulation accuracy.
    return floor(result * 255.0) / 255.0;
}
