/// Shader used for drawing all flavors of gradients.

struct Gradient {
    colors: array<vec4<f32>,16u>;
    ratios: array<f32,16u>;
    gradient_type: i32;
    num_colors: u32;
    repeat_mode: i32;
    interpolation: i32;
    focal_point: f32;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[group(2), binding(0)]]
var<uniform> textureTransforms: TextureTransforms;
[[group(2), binding(1)]]
var<storage> gradient: Gradient;

[[stage(vertex)]]
fn main_vertex(in: VertexInput) -> VertexOutput {
    let matrix = textureTransforms.matrix;
    let uv = (mat3x3<f32>(matrix[0].xyz, matrix[1].xyz, matrix[2].xyz) * vec3<f32>(in.position, 1.0)).xy;
    let pos = globals.view_matrix * transforms.world_matrix * vec4<f32>(in.position.x, in.position.y, 0.0, 1.0);
    return VertexOutput(pos, uv);
}

[[stage(fragment)]]
fn main_fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let last = gradient.num_colors - 1u;

    // Calculate normalized `t` position in gradient, [0.0, 1.0] being the bounds of the ratios.
    var t: f32;
    switch( gradient.gradient_type ){
        // Radial gradient
        case 1: {
            t = length(in.uv * 2.0 - 1.0);
            break;
        }

        // Focal gradient
        case 2: {
            let uv = in.uv * 2.0 - 1.0;
            var d: vec2<f32> = vec2<f32>(gradient.focal_point, 0.0) - uv;
            let l = length(d);
            d = d / l;
            t = l / (sqrt(1.0 - gradient.focal_point * gradient.focal_point * d.y * d.y) + gradient.focal_point * d.x);
            break;
        }

        // Linear gradient
        default: {
            t = in.uv.x;
            break;
        }
    }

    // Tweak out-of-bounds `t` based on the repeat mode.
    switch( gradient.repeat_mode ){
        // Repeat
        case 1: {
            t = fract(t);
            break;
        }

        // Mirror
        case 2: {
            if( t < 0.0 )
            {
                t = -t;
            }
            if( (i32(t)&1) == 0 ) {
                t = fract(t);
            } else {
                t = 1.0 - fract(t);
            }
            break;
        }

        // Clamp
        default: {
            t = clamp(t, 0.0, 1.0);
            break;
        }
    }
    t = clamp(t, gradient.ratios[0], gradient.ratios[last]);

    // Find the two gradient colors bordering our position.
    var j: u32;
    for( j = 1u; t > gradient.ratios[j]; j = j + 1u) {
        // Noop
    }
    let i = j - 1u;

    // Lerp between the two colors.
    let a = (t - gradient.ratios[i]) / (gradient.ratios[j] - gradient.ratios[i]);
    var color: vec4<f32> = mix(gradient.colors[i], gradient.colors[j], a);
    if( gradient.interpolation != 0 ) {
        color = linear_to_srgb(color);
    }
    let out = color * transforms.mult_color + transforms.add_color;
    return output(out);
}
