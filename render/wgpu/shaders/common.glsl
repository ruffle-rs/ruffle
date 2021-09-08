vec4 linear_to_srgb(vec4 linear)
{
    vec3 a = 12.92 * linear.rgb;
    vec3 b = 1.055 * pow(linear.rgb, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linear.rgb);
    return vec4(mix(a, b, c), linear.a);
}

vec4 srgb_to_linear(vec4 srgb)
{
    vec3 a = srgb.rgb / 12.92;
    vec3 b = pow((srgb.rgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
    vec3 c = step(vec3(0.04045), srgb.rgb);
    return vec4(mix(a, b, c), srgb.a);
}
