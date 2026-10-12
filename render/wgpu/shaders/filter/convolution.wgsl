// NOTE: The `shader_filter_common.wgsl` source is prepended to this before compilation.

/// Four non-zero kernel taps packed into one group.
struct TapGroup {
    /// Per lane: low byte is `dx + 127`, next byte is `dy + 127`.
    offsets: vec4<i32>,
    weights: vec4<f32>,
};

struct Filter {
    /// Top-left corner of the filtered region, in whole source texels.
    source_point: vec2<f32>,
    /// Size of the filtered region, in whole source texels.
    source_size: vec2<f32>,
    /// Size of the entire source texture, in whole source texels.
    texture_size: vec2<f32>,
    divisor: f32,
    bias: f32,
    /// Non-zero when pixels outside of the source region should use
    /// `default_color` instead of the closest edge pixel.
    use_default_color: f32,
    /// Non-zero when the alpha channel should be left untouched.
    preserve_alpha: f32,
    /// Number of valid entries in `taps` (zero-weight matrix entries are not
    /// packed at all).
    tap_count: u32,
    _pad_scalar: f32,
    /// Padding, to keep `default_color` 16-byte aligned. Arrays in the uniform
    /// address space must be 16-byte aligned, so a vec4 is used here.
    _pad_vec: vec4<f32>,
    default_color: vec4<f32>,
    /// Packed non-zero kernel taps of the effective (at most 15x15) kernel.
    taps: array<TapGroup, 64u>,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var<uniform> filter_args: Filter;

@vertex
fn main_vertex(in: filter__VertexInput) -> filter__VertexOutput {
    return filter__main_vertex(in);
}

/// Source textures hold premultiplied alpha, while `ConvolutionFilter` weights
/// apply to straight colours (Flash's results for a 1x1 identity kernel return
/// the source RGB untouched even for semi-transparent pixels), so every tap is
/// converted before being weighted.
fn unpremultiply(color: vec4<f32>) -> vec4<f32> {
    if (color.a <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return vec4<f32>(color.rgb / color.a, color.a);
}

@fragment
fn main_fragment(in: filter__VertexOutput) -> @location(0) vec4<f32> {
    let source_point = filter_args.source_point;
    let source_size = filter_args.source_size;
    let preserve_alpha = filter_args.preserve_alpha > 0.5;
    let texture_size = vec2<i32>(filter_args.texture_size);

    // Position of the pixel we're producing, in whole source texels
    let dest = vec2<i32>(floor(in.uv * filter_args.texture_size));

    // Alpha of the pixel being produced, used by `preserveAlpha`.
    let source_alpha = textureLoad(
        texture,
        clamp(dest, vec2<i32>(0, 0), texture_size - vec2<i32>(1, 1)),
        0,
    ).a;

    // Only non-zero kernel entries are packed, so the loop cost is
    // proportional to the number of taps that actually contribute. The taps
    // are whole texels: `textureLoad` takes integer coordinates, needs no
    // sampler, and cannot be affected by mip selection or implicit
    // derivatives.
    var sum = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let group_count = (filter_args.tap_count + 3u) / 4u;
    for (var group_index = 0u; group_index < group_count; group_index += 1u) {
        let group = filter_args.taps[group_index];
        for (var lane = 0u; lane < 4u; lane += 1u) {
            let tap_index = group_index * 4u + lane;
            if (tap_index >= filter_args.tap_count) {
                continue;
            }

            let packed_offset = group.offsets[lane];
            let dx = (packed_offset & 255) - 127;
            let dy = ((packed_offset >> 8) & 255) - 127;
            let tap = dest + vec2<i32>(dx, dy);
            let weight = group.weights[lane];

            var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

            let inside = f32(tap.x) >= source_point.x
                && f32(tap.y) >= source_point.y
                && f32(tap.x) < source_point.x + source_size.x
                && f32(tap.y) < source_point.y + source_size.y;
            if (inside) {
                color = unpremultiply(textureLoad(texture, tap, 0));
            } else if (filter_args.use_default_color > 0.5) {
                color = filter_args.default_color;
            } else {
                // Clamp: reuse the closest pixel inside the source region.
                let clamped = clamp(
                    tap,
                    vec2<i32>(source_point),
                    vec2<i32>(source_point + source_size) - vec2<i32>(1, 1),
                );
                color = unpremultiply(textureLoad(texture, clamped, 0));
            }

            // `preserveAlpha` leaves the alpha channel out of the convolution
            // entirely, so transparent source pixels can't darken it.
            //
            // Taps are scaled back to 0..255 before being weighted: Flash
            // convolves 8-bit integer pixel values, so a tap is always a whole
            // number. Accumulating the normalized 0..1 texel values instead
            // lets rounding error push an average that should land exactly on
            // an integer a hair below it, and the truncation further down then
            // drops it by one -- which is enough to send a feedback simulation
            // built on this filter into a completely different trajectory.
            let tap_value = round(color * 255.0);
            sum += vec4<f32>(tap_value.rgb, select(tap_value.a, 0.0, preserve_alpha)) * weight;
        }
    }

    var divisor = filter_args.divisor;
    if (divisor == 0.0) {
        // Flash treats a zero divisor as 1
        divisor = 1.0;
    }

    // `sum` is in 0..255 units, matching the domain Flash accumulates in.
    var alpha = sum.a / divisor;
    if (preserve_alpha) {
        // The alpha channel is taken from the source pixel instead of the
        // convolution. Note that Flash still applies `bias` to it below.
        alpha = source_alpha * 255.0;
    }

    // `bias` is added to all four channels, including alpha, and the sum is
    // clamped afterwards -- Flash's own output confirms this
    // (`new ConvolutionFilter(1, 1, [1], 1, 40)` turns `0x8000FF00` into
    // `0x8028FF28`, both with and without `preserveAlpha`).
    //
    // Flash accumulates in 8-bit integer arithmetic and truncates, so an
    // average of 42.67 becomes 42 rather than 43. Reproducing that here keeps
    // the results identical instead of differing by an LSB.
    let color = clamp(
        floor(vec4<f32>(sum.rgb / divisor, alpha)) / 255.0
            + vec4<f32>(filter_args.bias / 255.0),
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        vec4<f32>(1.0, 1.0, 1.0, 1.0),
    );

    // Flash convolves straight (unpremultiplied) colours and premultiplies the
    // result, which is what the render target holds. For fully opaque sources
    // -- every bitmap the water-ripple simulation touches, and the common case
    // generally -- this is a no-op and matches Flash bit for bit.
    return vec4<f32>(color.rgb * color.a, color.a);
}
