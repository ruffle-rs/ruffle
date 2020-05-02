#version 100
precision mediump float;

uniform mat4 view_matrix;
uniform mat4 world_matrix;
uniform vec4 mult_color;
uniform vec4 add_color;
uniform mat3 u_matrix;

uniform int u_gradient_type;
uniform float u_ratios[8];
uniform vec4 u_colors[8];
uniform int u_num_colors;
uniform int u_repeat_mode;
uniform float u_focal_point;

varying vec2 frag_uv;

void main() {
    float t;
    if( u_gradient_type == 0 )
    {
        t = frag_uv.x;
    }
    else if( u_gradient_type == 1 )
    {
        t = length(frag_uv * 2.0 - 1.0);
    }
    else if( u_gradient_type == 2 )
    {
        vec2 uv = frag_uv * 2.0 - 1.0;
        vec2 d = vec2(u_focal_point, 0.0) - uv;
        float l = length(d);
        d /= l;
        t = l / (sqrt(1.0 -  u_focal_point*u_focal_point*d.y*d.y) + u_focal_point*d.x);
    }
    if( u_repeat_mode == 0 )
    {
        // Clamp
        t = clamp(t, 0.0, 1.0);
    }
    else if( u_repeat_mode == 1 )
    {
        // Repeat
        t = fract(t);
    }
    else
    {
        // Mirror
        if( t < 0.0 )
        {
            t = -t;
        }

        if( int(mod(t, 2.0)) == 0 ) {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    }

    // TODO: No non-constant array access in WebGL 1, so the following is kind of painful.
    // We'd probably be better off passing in the gradient as a texture and sampling from there.
    vec4 color;
    float a;
    if( t <= u_ratios[0] ) {
        color = u_colors[0];
    } else if( t <= u_ratios[1] ) {    
        a = (t - u_ratios[0]) / (u_ratios[1] - u_ratios[0]);
        color = mix(u_colors[0], u_colors[1], a);
    } else if( t <= u_ratios[2] ) {    
        a = (t - u_ratios[1]) / (u_ratios[2] - u_ratios[1]);
        color = mix(u_colors[1], u_colors[2], a);
    } else if( t <= u_ratios[3] ) {    
        a = (t - u_ratios[2]) / (u_ratios[3] - u_ratios[2]);
        color = mix(u_colors[2], u_colors[3], a);
    } else if( t <= u_ratios[4] ) {    
        a = (t - u_ratios[3]) / (u_ratios[4] - u_ratios[3]);
        color = mix(u_colors[3], u_colors[4], a);
    } else if( t <= u_ratios[5] ) {    
        a = (t - u_ratios[4]) / (u_ratios[5] - u_ratios[4]);
        color = mix(u_colors[4], u_colors[5], a);
    } else if( t <= u_ratios[6] ) {    
        a = (t - u_ratios[5]) / (u_ratios[6] - u_ratios[5]);
        color = mix(u_colors[5], u_colors[6], a);
    } else if( t <= u_ratios[7] ) {
        a = (t - u_ratios[6]) / (u_ratios[7] - u_ratios[6]);
        color = mix(u_colors[6], u_colors[7], a);
    } else {
        color = u_colors[7];
    }

    gl_FragColor = mult_color * color + add_color;
}
