#version 450

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view_matrix;
    mat4 world_matrix;
};

layout(set = 0, binding = 1) uniform Colors {
    vec4 mult_color;
    vec4 add_color;
};

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = color * mult_color + add_color;
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}