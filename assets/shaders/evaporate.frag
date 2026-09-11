#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 tint;      // violet rim glow
uniform lowp float progress; // banishment 0..1
uniform lowp float edge;     // tear edge softness
uniform lowp float time;     // seconds, drives void shimmer
uniform lowp float ripple;   // void shimmer strength

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123); }

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

// Sharp ridges -> jagged tear lines.
float ridge(vec2 p) { return 1.0 - abs(2.0 * noise(p) - 1.0); }

void main() {
    vec4 color = texture2D(Texture, uv);

    // Vertical rips in reality; two octaves so several tears open across the body.
    float r = max(ridge(uv * vec2(7.0, 3.5)), ridge(uv * vec2(11.0, 5.0) + 5.0));
    float tear = smoothstep(1.0 - progress * 1.15, 1.0 - progress * 1.15 + edge * 0.5, r);
    float rim = tear * (1.0 - tear) * 4.0;

    // The void behind the rip: near-black with a faint violet shimmer.
    float shimmer = noise(uv * 9.0 + vec2(0.0, -time * 0.7));
    vec3 voidCol = mix(vec3(0.01, 0.0, 0.03), vec3(0.12, 0.02, 0.28), shimmer * ripple);

    // What is left of the demon dims and dissolves into grains.
    float n = hash(floor(uv * 48.0));
    float dissolve = smoothstep(0.0, 1.0, progress * 1.7 - n);
    vec3 demon = color.rgb * (1.0 - progress * 0.4);

    // Composite: dimmed demon, violet rims along the tears, void inside.
    vec3 col = demon + tint.rgb * rim * 1.6;
    col = mix(col, voidCol, tear);

    // Torn areas show the void (opaque); the rest dissolves away.
    float alpha = mix(color.a * (1.0 - dissolve), color.a, tear);
    // Final beat: the tears close and everything fades.
    alpha *= 1.0 - smoothstep(0.82, 1.0, progress);

    gl_FragColor = vec4(col, alpha);
}
