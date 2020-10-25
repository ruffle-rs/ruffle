#version 450

// Set 0: globals
layout(set = 0, binding = 0) uniform Globals {
    mat4 view_matrix;
};

// Set 1: shape
layout(set = 1, binding = 0) uniform Transforms {
    mat4 world_matrix;
};

// Set 2: bitmap or gradient
layout(set = 2, binding = 0) uniform Texture {
    mat4 u_matrix;
};

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec2 frag_uv;

void main() {
    frag_uv = vec2(mat3(u_matrix) * vec3(position, 1.0));
    gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}