#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity;

uniform float audio_bass;
uniform float audio_mid;

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

void main() {
    vec2 uv = TexCoord;

    // 1. Horizontal glitch slicers triggered by mids
    float slice_tier = floor(uv.y * 30.0);
    float slice_rand = random(vec2(slice_tier, floor(time * 12.0)));
    if (slice_rand > (0.92 - audio_mid * 0.15)) {
        float offset = (random(vec2(slice_tier, time)) - 0.5) * 0.12 * sensitivity;
        uv.x = fract(uv.x + offset);
    }

    // 2. Pixel sorting / dripping melt simulation
    // Samples brightness slightly above to push streaks downward
    float melt_strength = (audio_bass * 0.25 + 0.02) * sensitivity;
    vec4 top_sample = texture(videoTexture, uv + vec2(0.0, -0.05 * melt_strength));
    float lum = dot(top_sample.rgb, vec3(0.2126, 0.7152, 0.0722));

    // Drip pixels downwards if luminance exceeds a threshold
    if (lum > 0.45) {
        float drip = fract(sin(uv.x * 240.0) * 43758.5) * melt_strength;
        uv.y = clamp(uv.y - drip * (lum - 0.4), 0.0, 1.0);
    }

    // 3. Chromatic RGB offset along the distortion vectors
    float split = (audio_bass * 0.04 + 0.003) * sensitivity;
    float r = texture(videoTexture, vec2(uv.x + split, uv.y)).r;
    float g = texture(videoTexture, uv).g;
    float b = texture(videoTexture, vec2(uv.x - split, uv.y)).b;
    vec3 col = vec3(r, g, b);

    // 4. Color banding / quantization for that harsh digital cyber look
    col = floor(col * 6.0) / 6.0;

    // 5. Electric scanline strobe on the beat
    col *= 0.85 + 0.15 * sin(uv.y * resolution.y * 1.5 + time * 20.0 * (1.0 + audio_bass));

    FragColor = vec4(col, 1.0);
}
