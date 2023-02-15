#import filter

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Filter {
    r_to_r: f32,
    g_to_r: f32,
    b_to_r: f32,
    a_to_r: f32,
    r_extra: f32,

    r_to_g: f32,
    g_to_g: f32,
    b_to_g: f32,
    a_to_g: f32,
    g_extra: f32,

    r_to_b: f32,
    g_to_b: f32,
    b_to_b: f32,
    a_to_b: f32,
    b_extra: f32,

    r_to_a: f32,
    g_to_a: f32,
    b_to_a: f32,
    a_to_a: f32,
    a_extra: f32,
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
    var src = textureSample(filter::texture, filter::texture_sampler, in.uv);
    var f = filter_args;
    var color = vec4<f32>(
        clamp((f.r_to_r * src.r / src.a) + (f.g_to_r * src.g / src.a) + (f.b_to_r * src.b / src.a) + (f.a_to_r * src.a) + (f.r_extra / 255.0), 0.0, 1.0),
        clamp((f.r_to_g * src.r / src.a) + (f.g_to_g * src.g / src.a) + (f.b_to_g * src.b / src.a) + (f.a_to_g * src.a) + (f.g_extra / 255.0), 0.0, 1.0),
        clamp((f.r_to_b * src.r / src.a) + (f.g_to_b * src.g / src.a) + (f.b_to_b * src.b / src.a) + (f.a_to_b * src.a) + (f.b_extra / 255.0), 0.0, 1.0),
        clamp((f.r_to_a * src.r / src.a) + (f.g_to_a * src.g / src.a) + (f.b_to_a * src.b / src.a) + (f.a_to_a * src.a) + (f.a_extra / 255.0), 0.0, 1.0),
    );
    return vec4<f32>(color.rgb * color.a, color.a);
}
