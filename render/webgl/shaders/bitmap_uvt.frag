#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec4 mult_color;
uniform vec4 add_color;
uniform sampler2D u_texture;

varying vec3 frag_uvt;

void main() {
    if (frag_uvt.z == 0.0) {
        discard;
    }
    vec4 color = texture2D(u_texture, frag_uvt.xy / frag_uvt.z);

    // Unmultiply alpha before applying the color transform.
    if (color.a > 0.0) {
        color.rgb /= color.a;
        color = clamp(mult_color * color + add_color, 0.0, 1.0);
        float alpha = clamp(color.a, 0.0, 1.0);
        color = vec4(color.rgb * alpha, alpha);
    }

    gl_FragColor = color;
}
