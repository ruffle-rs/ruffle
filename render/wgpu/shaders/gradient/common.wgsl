struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var<uniform> textureTransforms: TextureTransforms;

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    let matrix_ = textureTransforms.matrix_;
    let uv = (mat3x3<f32>(matrix_[0].xyz, matrix_[1].xyz, matrix_[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let last = gradient.num_colors - 1u;

    // Calculate normalized `t` position in gradient, [0.0, 1.0] being the bounds of the ratios.
    var t: f32 = find_t(gradient.focal_point, in.uv);
    t = normalize_t(t);
    t = clamp(t, ratio(0u), ratio(last));

    // Find the two gradient colors bordering our position.
    var j: u32;
    for( j = 1u; t > ratio(j); j = j + 1u) {
        // Noop
    }
    let i = j - 1u;

    // Lerp between the two colors.
    let a = (t - ratio(i)) / (ratio(j) - ratio(i));
    var color: vec4<f32> = mix(gradient.colors[i], gradient.colors[j], a);
    if( gradient.interpolation != 0 ) {
        color = linear_to_srgb(color);
    }
    let out = color * transforms.mult_color + transforms.add_color;
    let alpha = clamp(out.a, 0.0, 1.0);
    return vec4<f32>(out.rgb * alpha, alpha);
}