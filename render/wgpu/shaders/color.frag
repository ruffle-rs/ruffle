#version 450

// Set 1: shape
layout(set = 1, binding = 1) uniform Colors {
    vec4 mult_color;
    vec4 add_color;
};

layout(location=0) in vec4 frag_color;

layout(location=0) out vec4 out_color;

void main() {
    out_color = mult_color * frag_color + add_color;
}