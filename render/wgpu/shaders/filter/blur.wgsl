#import filter

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Filter {
    blur_x: f32,
    blur_y: f32,
    width: f32,
    height: f32,
}

#if use_push_constants == true
    @group(2) @binding(0) var<uniform> filter_args: Filter;
#else
    @group(4) @binding(0) var<uniform> filter_args: Filter;
#endif

@vertex
fn main_vertex(in: filter::FilterVertexInput) -> filter::VertexOutput {
    return filter::main_vertex(in);
}

@fragment
fn main_fragment(in: filter::VertexOutput) -> @location(0) vec4<f32> {
    var f = filter_args;
    var color = vec4<f32>();
    var sum = 0.0;

    for (var x = -f.blur_x; x <= f.blur_x; x += 1.0) {
        for (var y = -f.blur_y; y <= f.blur_y; y += 1.0) {
            var offset = vec3<f32>(x / f.width, y / f.height, 0.0);
            let sample = textureSample(filter::texture, filter::texture_sampler, in.uv + offset.xy);
            let weight = 1.0;
            color += vec4<f32>(sample.rgb / sample.a, sample.a) * weight;
            sum += weight;
        }
    }
    color /= sum;

    return vec4<f32>(color.rgb * color.a, color.a);
}
