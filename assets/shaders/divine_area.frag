#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;
uniform vec4 _Time;
uniform lowp vec4 tint;
uniform lowp float appear;
uniform lowp float fade;
uniform lowp float alpha;

/// Dashed arc pattern: `count` dashes around the circle, scrolling at `speed`
/// turns per second, with `duty` fraction of each dash lit.
float dash(float ang, float count, float speed, float duty) {
    float turns = ang / 6.2831853;
    return step(1.0 - duty, fract(turns * count + _Time.x * speed));
}

void main() {
    // Centered coords; r is 0 at the center and ~1 at the quad edge. The quad
    // is drawn stretched into the zone's oval, so circles become ellipses
    // matching the footprint.
    vec2 p = uv - 0.5;
    float r = length(p) * 2.0;
    if (r > 1.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }
    float ang = atan(p.y, p.x);
    float t = _Time.x;

    // Soft ground fill, a touch brighter toward the middle.
    float fill = smoothstep(1.0, 0.15, r);

    // Hot pulsing core at the center.
    float core = smoothstep(0.30, 0.0, r) * (0.65 + 0.35 * sin(t * 3.2));

    // Outer dashed rune ring, slowly rotating.
    float band1 = smoothstep(0.045, 0.0, abs(r - 0.90));
    float ring1 = band1 * dash(ang, 28.0, 0.25, 0.55);

    // Inner fine dashed ring, counter-rotating.
    float band2 = smoothstep(0.030, 0.0, abs(r - 0.66));
    float ring2 = band2 * dash(ang, 40.0, -0.35, 0.5);

    // Slow light shafts sweeping around the zone.
    float shafts = pow(0.5 + 0.5 * sin(ang * 7.0 + t * 0.9), 3.0)
        * smoothstep(0.85, 0.25, r) * 0.35;

    // A pulse ring expanding outward every ~1.6s.
    float pr = fract(t * 0.625);
    float pulse_ring = smoothstep(0.06, 0.0, abs(r - pr * 0.9)) * (1.0 - pr) * 0.8;

    // Materialization flash: a bright bloom that dies as `appear` completes.
    float a = smoothstep(0.0, 1.0, appear);
    float summon = (1.0 - a) * smoothstep(0.7, 0.0, r) * 2.0;

    // Fade-out dissolve: the circle breaks apart, rim first, with static
    // noise so it crumbles instead of uniformly dimming.
    float n = fract(sin(floor(ang * 14.0) * 91.7 + floor(r * 14.0) * 37.3) * 43758.55);
    float d = r * 0.55 + n * 0.45;
    float survive = 1.0 - smoothstep(1.0 - fade - 0.25, 1.0 - fade, d);

    float glow = fill * 0.22 + core * 0.7 + ring1 * 0.9 + ring2 * 0.8
        + shafts + pulse_ring + summon;
    glow *= survive;

    vec3 color = tint.rgb * glow
        + vec3(1.0, 0.96, 0.8) * (core * 0.35 + ring1 * 0.25 + summon * 0.4) * survive;

    gl_FragColor = vec4(color, clamp(glow, 0.0, 1.0) * a * alpha);
}
