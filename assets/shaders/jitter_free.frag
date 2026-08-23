#version 100
#ifdef GL_OES_standard_derivatives
#extension GL_OES_standard_derivatives : enable
#endif
#ifdef GL_ES
precision highp float;
#endif

varying highp vec2 uv;
varying lowp vec4 color;

uniform sampler2D Texture;
uniform vec2 texture_size;

void main() {
    vec2 texel  = 1.0 / texture_size;
    vec2 p      = uv * texture_size;
    vec2 ddxy   = max(fwidth(p), vec2(1e-6));
    vec2 k      = 1.0 / ddxy;

    vec2 m = floor(p + 0.5);
    vec2 d = p - (m - 0.5);
    vec2 w = clamp((d - 0.5) * k + 0.5, 0.0, 1.0);

    vec2 i0 = clamp(m - 1.0, vec2(0.0), texture_size - 1.0);
    vec2 i1 = clamp(m,       vec2(0.0), texture_size - 1.0);

    vec4 c00 = texture2D(Texture, (vec2(i0.x, i0.y) + 0.5) * texel);
    vec4 c10 = texture2D(Texture, (vec2(i1.x, i0.y) + 0.5) * texel);
    vec4 c01 = texture2D(Texture, (vec2(i0.x, i1.y) + 0.5) * texel);
    vec4 c11 = texture2D(Texture, (vec2(i1.x, i1.y) + 0.5) * texel);

    gl_FragColor = color * mix(mix(c00, c10, w.x), mix(c01, c11, w.x), w.y);
}
