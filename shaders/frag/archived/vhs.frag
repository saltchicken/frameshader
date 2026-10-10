#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity;

// --- AUDIO UNIFORMS ---
uniform float audio_bass;
uniform float audio_mid;

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

void main() {
    vec2 uv = TexCoord;

    // 1. Tape wave effect (horizontal wobble)
    float wave = sin(uv.y * 10.0 + time * 5.0) * 0.005 * sensitivity;
    // Audio bass aggressively pumps the wobble
    wave += sin(uv.y * 20.0 - time * 15.0) * 0.015 * audio_bass;
    uv.x += wave;

    // Keep UVs in bounds to avoid wrapping artifacts from the wobble
    uv.x = clamp(uv.x, 0.0, 1.0);

    // 2. Chromatic aberration (RGB shift) pushed by audio_mid
    float shift = 0.003 * sensitivity + (audio_mid * 0.02);
    
    float r = texture(videoTexture, vec2(uv.x + shift, uv.y)).r;
    float g = texture(videoTexture, uv).g;
    float b = texture(videoTexture, vec2(uv.x - shift, uv.y)).b;
    vec3 color = vec3(r, g, b);

    // 3. Scanlines
    float scanline = sin(uv.y * resolution.y * 3.1415) * 0.05;
    color -= scanline;

    // 4. Glitch / Static noise streaks triggered by heavy bass
    float noise = random(uv * vec2(1.0, 0.05) + vec2(time, time * 0.1));
    float glitch_threshold = 0.98 - (audio_bass * 0.15); // Bass makes glitch more likely
    
    if (noise > glitch_threshold) {
        float static_intensity = random(uv + time);
        color = mix(color, vec3(static_intensity), 0.6); // Mix in white/grey noise
    }

    FragColor = vec4(color, 1.0);
}
