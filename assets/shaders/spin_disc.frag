#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;
uniform vec4 _Time;
uniform lowp vec4 tint;
uniform lowp float alpha;

void main() {
    vec4 tex = texture2D(Texture, uv);
    if (tex.a < 0.02) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }

    // Centered coords, radius and angle around the shield's center.
    vec2 p = uv - 0.5;
    float r = length(p);
    float ang = atan(p.y, p.x);

    // Continuous rotation: the angle streams faster with time.
    float spin = _Time.x * 10.0;
    float a = ang + spin;

    // Rotating fan streaks — the "blades" of the spinning shield.
    float spokes = 0.5 + 0.5 * sin(a * 6.0);

    // Spiral arms that sweep outward, so the spin reads as angular motion
    // rather than a static starburst.
    float swirl = 0.5 + 0.5 * sin(a * 3.0 - r * 14.0 + spin * 2.0);

    // Radial energy pulse surging from the hub to the rim.
    float pulse = 0.5 + 0.5 * sin(r * 20.0 - _Time.x * 16.0);

    // Rim glow around the outer edge.
    float rim = smoothstep(0.52, 0.10, r);

    // Weight the motion terms toward the rim so the disc keeps a bright,
    // readable hub and the spin shows at the edge.
    float motion = spokes * (0.30 + 0.70 * r) + swirl * 0.45 * r + pulse * 0.30;

    // Blend the steel toward the energy color and layer the motion glow.
    vec3 base = mix(tex.rgb, tint.rgb, 0.35 + 0.30 * motion);
    vec3 color = base + tint.rgb * (rim * 0.8 + motion * 0.7);
    color = max(color, tex.rgb * 0.6);

    gl_FragColor = vec4(color, tex.a * alpha);
}
