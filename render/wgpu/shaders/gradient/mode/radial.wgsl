#import gradient

override fn gradient::find_t(focal_point: f32, uv: vec2<f32>) -> f32 {
    var t = length(uv * 2.0 - 1.0);
    #if gradient_repeat_mode == 1
        // Mirror
        if( t < 0.0 ) {
            t = -t;
        }
        if ( (i32(t) & 1) == 0 ) {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    #endif
    #if gradient_repeat_mode == 2
        // Repeat
        t = fract(t);
    #endif
    #if gradient_repeat_mode == 3
        // Clamp
        t = clamp(t, 0.0, 1.0);
    #endif
    return t;
}

@vertex
fn main_vertex(in: gradient::GradientVertexInput) -> gradient::VertexOutput {
    return gradient::main_vertex(in);
}

@fragment
fn main_fragment(in: gradient::VertexOutput) -> @location(0) vec4<f32> {
    return gradient::main_fragment(in);
}