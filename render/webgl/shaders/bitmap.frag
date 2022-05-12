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

uniform sampler2D u_texture;

varying vec2 frag_uv;

void main() {
    vec4 color = texture2D(u_texture, frag_uv);

    // Unmultiply alpha before apply color transform.
    if( color.a > 0.0 ) {
        color.rgb /= color.a;
        color = mult_color * color + add_color;
        float alpha = saturate(color.a);
        color = vec4(color.rgb * alpha, alpha);
    }

    gl_FragColor = color;
}