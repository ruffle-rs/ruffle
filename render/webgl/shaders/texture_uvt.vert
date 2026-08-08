#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform mat4 view_matrix;
uniform mat4 world_matrix;

attribute vec2 position;
attribute vec3 texture_coords;

varying vec3 frag_uvt;

void main() {
    frag_uvt = texture_coords;
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
}
