#version 450
#include "common.glsl"

// Push constants: matrix + color
layout(set = 1, binding = 0) uniform Transforms {
    mat4 world_matrix;
    vec4 mult_color;
    vec4 add_color;
};

// Set 2: bitmap
layout(set = 2, binding = 1) uniform texture2D t_color;

// Set 3: sampler
layout(set = 3, binding = 0) uniform sampler s_color;

layout(location=0) in vec2 frag_uv;

layout(location=0) out vec4 out_color;

void main() {
    vec4 color = texture(sampler2D(t_color, s_color), frag_uv);

    // Unmultiply alpha before apply color transform.
    if( color.a > 0 ) {
        color.rgb /= color.a;
        color = mult_color * color + add_color;
        color.rgb *= color.a;
    }

    out_color = color;
#ifdef SRGB_RENDER_TARGET
    out_color = srgb_to_linear(out_color);
#endif
}
