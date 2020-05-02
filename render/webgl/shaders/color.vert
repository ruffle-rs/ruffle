#version 100
precision mediump float;

uniform mat4 view_matrix;
uniform mat4 world_matrix;
uniform vec4 mult_color;
uniform vec4 add_color;

attribute vec2 position;
attribute vec4 color;
varying vec4 frag_color;

void main() {
    frag_color = color * mult_color + add_color;
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
}
