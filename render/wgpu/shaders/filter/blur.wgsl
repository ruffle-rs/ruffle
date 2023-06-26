#import filter

struct Filter {
    blur_x: f32,
    blur_y: f32,
    width: f32,
    height: f32,
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
    var f = filter_args;
    var color = vec4<f32>();
    var sum = 0.0;

    let blur_x = f.blur_x / 2.0;
    let blur_y = f.blur_y / 2.0;

    for (var x = -blur_x; x <= blur_x; x += 0.5) {
        for (var y = -blur_y; y <= blur_y; y += 0.5) {
            var offset = vec3<f32>(x / f.width, y / f.height, 0.0);
            let sample = textureSample(texture, texture_sampler, in.uv + offset.xy);
            let weight = 1.0;
            color += sample * weight;
            sum += weight;
        }
    }
    color /= sum;

    return color;
}
