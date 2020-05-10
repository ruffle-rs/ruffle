#version 450

layout(set = 0, binding = 2) uniform Colors {
    vec4 mult_color;
    vec4 add_color;
};

layout(std430, set = 0, binding = 3) buffer Gradient {
    int u_gradient_type;
    uint u_num_colors;
    int u_repeat_mode;
    float u_focal_point;
    float u_ratios[16];
    vec4 u_colors[16];
};

layout(location=0) in vec2 frag_uv;

layout(location=0) out vec4 out_color;

void main() {
    vec4 color;
    int last = int(int(u_num_colors) - 1);
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
        if( (int(t)&1) == 0 ) {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    }
    int i = 0;
    int j = 1;
    t = clamp(t, u_ratios[0].x, u_ratios[last].x);
    while( t > u_ratios[j].x )
    {
        i = j;
        j++;
    }
    float a = (t - u_ratios[i].x) / (u_ratios[j].x - u_ratios[i].x);
    color = mix(u_colors[i], u_colors[j], a);
    out_color = mult_color * color + add_color;
}