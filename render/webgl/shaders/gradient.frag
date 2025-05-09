#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform mat4 view_matrix;
uniform mat4 world_matrix;
uniform vec4 mult_color;
uniform vec4 add_color;
uniform mat3 u_matrix;

uniform int u_gradient_type;
uniform float u_ratios[16];
uniform vec4 u_colors[16];
uniform int u_repeat_mode;
uniform float u_focal_point;
uniform int u_interpolation;

varying vec2 frag_uv;

vec4 interpolate(float t, float ratio1, float ratio2, vec4 color1, vec4 color2) {
    color1 = clamp(mult_color * color1 + add_color, 0.0, 1.0);
    color2 = clamp(mult_color * color2 + add_color, 0.0, 1.0);
    float a = (t - ratio1) / (ratio2 - ratio1);
    return mix(color1, color2, a);
}

vec3 linear_to_srgb(vec3 linear) {
    vec3 a = 12.92 * linear;
    vec3 b = 1.055 * pow(linear, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linear);
    return mix(a, b, c);
}

void main() {
    float t;
    if (u_gradient_type == 0) {
        t = frag_uv.x;
    } else if (u_gradient_type == 1) {
        t = length(frag_uv * 2.0 - 1.0);
    } else if (u_gradient_type == 2) {
        vec2 uv = frag_uv * 2.0 - 1.0;
        vec2 d = vec2(u_focal_point, 0.0) - uv;
        float l = length(d);
        d /= l;
        t = l / (sqrt(1.0 -  u_focal_point*u_focal_point*d.y*d.y) + u_focal_point*d.x);
    }
    if (u_repeat_mode == 0) {
        // Clamp
        t = clamp(t, 0.0, 1.0);
    } else if (u_repeat_mode == 1) {
        // Repeat
        t = fract(t);
    } else {
        // Mirror
        if (t < 0.0) {
            t = -t;
        }

        if (int(mod(t, 2.0)) == 0) {
            t = fract(t);
        } else {
            t = 1.0 - fract(t);
        }
    }

    // TODO: No non-constant array access in WebGL 1, so the following is kind of painful.
    // We'd probably be better off passing in the gradient as a texture and sampling from there.
    vec4 color;
    if (t <= u_ratios[0]) {
        color = clamp(mult_color * u_colors[0] + add_color, 0.0, 1.0);
    } else if (t <= u_ratios[1]) {
        color = interpolate(t, u_ratios[0], u_ratios[1], u_colors[0], u_colors[1]);
    } else if (t <= u_ratios[2]) {
        color = interpolate(t, u_ratios[1], u_ratios[2], u_colors[1], u_colors[2]);
    } else if (t <= u_ratios[3]) {
        color = interpolate(t, u_ratios[2], u_ratios[3], u_colors[2], u_colors[3]);
    } else if (t <= u_ratios[4]) {
        color = interpolate(t, u_ratios[3], u_ratios[4], u_colors[3], u_colors[4]);
    } else if (t <= u_ratios[5]) {
        color = interpolate(t, u_ratios[4], u_ratios[5], u_colors[4], u_colors[5]);
    } else if (t <= u_ratios[6]) {
        color = interpolate(t, u_ratios[5], u_ratios[6], u_colors[5], u_colors[6]);
    } else if (t <= u_ratios[7]) {
        color = interpolate(t, u_ratios[6], u_ratios[7], u_colors[6], u_colors[7]);
    } else if (t <= u_ratios[8]) {
        color = interpolate(t, u_ratios[7], u_ratios[8], u_colors[7], u_colors[8]);
    } else if (t <= u_ratios[9]) {
        color = interpolate(t, u_ratios[8], u_ratios[9], u_colors[8], u_colors[9]);
    } else if (t <= u_ratios[10]) {
        color = interpolate(t, u_ratios[9], u_ratios[10], u_colors[9], u_colors[10]);
    } else if (t <= u_ratios[11]) {
        color = interpolate(t, u_ratios[10], u_ratios[11], u_colors[10], u_colors[11]);
    } else if (t <= u_ratios[12]) {
        color = interpolate(t, u_ratios[11], u_ratios[12], u_colors[11], u_colors[12]);
    } else if (t <= u_ratios[13]) {
        color = interpolate(t, u_ratios[12], u_ratios[13], u_colors[12], u_colors[13]);
    } else if (t <= u_ratios[14]) {
        color = interpolate(t, u_ratios[13], u_ratios[14], u_colors[13], u_colors[14]);
    } else {
        color = clamp(mult_color * u_colors[14] + add_color, 0.0, 1.0);
    }

    if (u_interpolation != 0) {
        color = vec4(linear_to_srgb(vec3(color)), color.a);
    }

    float alpha = clamp(color.a, 0.0, 1.0);
    gl_FragColor = vec4(color.rgb * alpha, alpha);
}
