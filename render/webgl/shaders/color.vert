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

attribute vec2 position;
attribute vec4 color;
varying vec4 frag_color;

void main() {
    frag_color = color * mult_color + add_color;
    float alpha = clamp(frag_color.a, 0.0, 1.0);
    frag_color = vec4(frag_color.rgb * alpha, alpha);
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
}
