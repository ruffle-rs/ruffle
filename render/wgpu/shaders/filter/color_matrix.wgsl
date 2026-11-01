// NOTE: The `shader_filter_common.wgsl` source is prepended to this before compilation.

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

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> filter_args: Filter;

@vertex
fn main_vertex(in: filter__VertexInput) -> filter__VertexOutput {
    return filter__main_vertex(in);
}

@fragment
fn main_fragment(in: filter__VertexOutput) -> @location(0) vec4<f32> {
    var src = textureSample(texture, texture_sampler, in.uv);
    var f = filter_args;
    // A fully transparent pixel has no color to unmultiply. Flash treats it as
    // transparent black, so the offsets still apply there.
    var rgb = vec3<f32>(0.0);
    if (src.a > 0.0) {
        rgb = src.rgb / src.a;
    }
    var color = vec4<f32>(
        clamp((f.r_to_r * rgb.r) + (f.g_to_r * rgb.g) + (f.b_to_r * rgb.b) + (f.a_to_r * src.a) + (f.r_extra / 255.0), 0.0, 1.0),
        clamp((f.r_to_g * rgb.r) + (f.g_to_g * rgb.g) + (f.b_to_g * rgb.b) + (f.a_to_g * src.a) + (f.g_extra / 255.0), 0.0, 1.0),
        clamp((f.r_to_b * rgb.r) + (f.g_to_b * rgb.g) + (f.b_to_b * rgb.b) + (f.a_to_b * src.a) + (f.b_extra / 255.0), 0.0, 1.0),
        clamp((f.r_to_a * rgb.r) + (f.g_to_a * rgb.g) + (f.b_to_a * rgb.b) + (f.a_to_a * src.a) + (f.a_extra / 255.0), 0.0, 1.0),
    );
    return vec4<f32>(color.rgb * color.a, color.a);
}
