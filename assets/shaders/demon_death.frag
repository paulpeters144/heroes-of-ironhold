#version 100
precision mediump float;

varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 rect;      // source rect in texture uv space (x, y, w, h)
uniform lowp float progress; // immolation 0..1
uniform lowp float time;     // seconds, drives flame flicker

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123); }

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

float fbm(vec2 p) {
    return noise(p) * 0.55 + noise(p * 2.3 + 7.0) * 0.3 + noise(p * 5.1 + 13.0) * 0.15;
}

void main() {
    // Sprite-local coordinates: (0,0) top-left, (1,1) bottom-right. The source
    // rect may be a sub-frame of a sprite sheet, so normalize first.
    vec2 p = (uv - rect.xy) / rect.zw;

    // The body convulses as the fire takes it.
    float wob = sin(p.y * 22.0 + time * 16.0) * 0.006 * progress;
    vec4 tex = texture2D(Texture, uv + vec2(wob, 0.0));

    // Wavy burn front crawling from the feet (p.y = 1) to the head (p.y = 0).
    float crawl = fbm(vec2(p.x * 5.0, p.y * 3.5 - time * 2.4));
    float front = progress * 1.4 - 0.2;
    float band = (1.0 - p.y) + (crawl - 0.5) * 0.45 - front; // < 0 consumed

    // Fire rim gradient: white-hot core -> orange -> deep red, flickering.
    float rim_t = clamp(band / 0.10, 0.0, 1.0);
    vec3 fire = mix(vec3(1.0, 0.92, 0.45), vec3(1.0, 0.38, 0.05), rim_t);
    fire = mix(fire, vec3(0.55, 0.04, 0.0), smoothstep(0.55, 1.0, rim_t));
    fire *= 0.8 + 0.45 * noise(p * 18.0 + vec2(0.0, -time * 9.0));

    // Scorched flesh just ahead of the front.
    float scorch = 1.0 - smoothstep(0.05, 0.26, band);
    vec3 col = mix(tex.rgb, vec3(0.04, 0.015, 0.01), scorch * 0.88);

    // Embers glow through the char.
    float glow = (1.0 - smoothstep(0.0, 0.16, band))
               * (0.6 + 0.4 * sin(time * 22.0 + p.y * 40.0));
    col += fire * glow * 0.9;

    // The burning rim itself.
    float rim = 1.0 - smoothstep(0.0, 0.075, band);
    col += fire * rim * 1.5;

    // Hellish flicker across the whole body, growing as it burns.
    col *= 1.0 + progress * 0.3 * (noise(vec2(time * 6.0, p.y * 4.0)) - 0.5);

    // Consumed below the front.
    float alpha = tex.a * smoothstep(-0.03, 0.0, band);

    gl_FragColor = vec4(col, alpha);
}
