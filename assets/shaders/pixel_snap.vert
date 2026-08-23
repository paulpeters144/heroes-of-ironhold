#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec2 viewport;

void main() {
    vec4 pos = Projection * Model * vec4(position, 1);
    vec2 ndc = pos.xy / pos.w;
    vec2 half_vp = 0.5 * viewport;
    vec2 b = ndc * half_vp + half_vp;
    b = floor(b + 0.5);
    ndc = (b - half_vp) / half_vp;
    pos.xy = ndc * pos.w;
    gl_Position = pos;
    color = color0 / 255.0;
    uv = texcoord;
}
