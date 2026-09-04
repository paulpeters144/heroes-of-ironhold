#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 tint;

void main() {
    vec4 tex = texture2D(Texture, uv);
    float a = tex.a * tint.a;
    gl_FragColor = vec4(tint.rgb, a);
}
