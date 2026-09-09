#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;

void main() {
    vec4 tex = texture2D(Texture, uv);
    gl_FragColor = vec4(1.0, 1.0, 1.0, tex.a);
}
