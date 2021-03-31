#version 450

// Set 0: globals
layout(set = 0, binding = 0) uniform Globals {
    mat4 view_matrix;
};

// Push constants: matrix + color
layout(push_constant) uniform VertexPushConstants {
    mat4 world_matrix;
};

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = color;
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}