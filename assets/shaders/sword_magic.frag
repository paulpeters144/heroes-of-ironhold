#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;
uniform vec4 _Time;
uniform lowp vec4 tint;
uniform lowp float trail_dir;
uniform lowp float flying;
uniform lowp float appear;
uniform lowp float fade;

void main() {
    vec4 tex = texture2D(Texture, uv);
    if (tex.a < 0.02) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }

    // Edge distance: 1.0 on the blade, falling to 0.0 at the silhouette.
    float edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    float rim = smoothstep(0.14, 0.0, edge);

    // Energy pulse sweeping along the blade.
    float pulse = 0.5 + 0.5 * sin(_Time.x * 6.0 + uv.x * 9.0 + uv.y * 5.0);

    // Trailing streaks: strongest behind the sword, grow as it accelerates.
    float trail_pos = mix(1.0 - uv.x, uv.x, step(0.0, trail_dir));
    float streaks = 0.5 + 0.5 * sin(trail_pos * 18.0 - _Time.x * 16.0);
    streaks *= streaks;
    float trail = smoothstep(0.0, 0.55, trail_pos) * (0.35 + 0.65 * flying) * streaks;

    // Materialization: alpha fades in, a bright summon glow bursts around the
    // blade, and shimmering energy sparks gather as the sword appears.
    float a = smoothstep(0.0, 1.0, appear);
    float summon = (1.0 - a) * (1.0 - smoothstep(0.05, 0.45, distance(uv, vec2(0.5))));
    float sparkle = (1.0 - a)
        * (0.5 + 0.5 * sin(uv.x * 40.0 + uv.y * 30.0 + _Time.x * 30.0));

    // Fade-out after striking: alpha falls back to zero, the blade shatters
    // away from its silhouette inward, and the energy color drains out.
    float f = 1.0 - fade;
    float shatter = smoothstep(0.0, 1.0, fade)
        * smoothstep(0.5, 0.05, abs(uv.y - 0.5));

    // Blend the steel toward the energy color and add rim + trail + summon glow.
    vec3 base = mix(tex.rgb, tint.rgb, 0.28 + 0.22 * pulse);
    vec3 color = base
        + tint.rgb * (rim * (0.6 + 0.5 * pulse) + trail * 1.1 + summon * 1.5 + sparkle * 0.8);
    color = max(color, tex.rgb * (0.75 + 0.25 * pulse + summon * 0.6));
    color = mix(color, tint.rgb * 0.2, shatter);

    gl_FragColor = vec4(color, tex.a * a * f);
}