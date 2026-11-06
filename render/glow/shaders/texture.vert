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

attribute vec2 position;
attribute vec4 color;

varying vec2 frag_uv;

void main() {
    frag_uv = vec2(u_matrix * vec3(position, 1.0));
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
}
