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

#define PI 3.141592653589793

void main() {
    // 1. Aspect ratio correction & centering so our kaleidoscope is perfectly round
    vec2 uv = TexCoord * 2.0 - 1.0;
    uv.x *= resolution.x / resolution.y;

    // 2. Audio-reactive world distortion (breathing/pulsing effect on the bass)
    float bass_pump = 1.0 - (audio_bass * 0.15 * sensitivity);
    float time_spin = time * 0.15 + (audio_mid * 0.2);

    // 3. Convert to Polar coordinates for Kaleidoscope mapping
    float radius = length(uv) * bass_pump;
    float angle = atan(uv.y, uv.x) + time_spin;

    // 4. Fold the space (8-sided mirror)
    float segments = 8.0; 
    float slice = 2.0 * PI / segments;
    angle = mod(angle, slice);
    angle = abs(angle - (slice / 2.0)); // Mirror within the slice

    // Rotate the inner projection over time so the texture is constantly flowing
    angle += time * 0.1;

    // 5. Convert back to Cartesian space
    vec2 mapped_uv = vec2(cos(angle), sin(angle)) * radius;

    // 6. Undo aspect ratio and map back to standard 0.0 -> 1.0 UV space
    mapped_uv.x /= (resolution.x / resolution.y);
    mapped_uv = mapped_uv * 0.5 + 0.5;

    // 7. Mirrored wrapping (Ping-Pong) to prevent edge cutoffs when zooming out
    vec2 mirror_uv = abs(mod(mapped_uv, 2.0) - 1.0);

    // 8. Heavy RGB Split / Chromatic Aberration driven by Bass & Mids
    float rgb_split = (audio_bass * 0.08 + audio_mid * 0.02) * sensitivity;

    // Sample the video with radial chromatic displacement
    float r = texture(videoTexture, abs(mod(mirror_uv + vec2(rgb_split, rgb_split), 2.0) - 1.0)).r;
    float g = texture(videoTexture, mirror_uv).g;
    float b = texture(videoTexture, abs(mod(mirror_uv - vec2(rgb_split, rgb_split), 2.0) - 1.0)).b;

    vec3 final_color = vec3(r, g, b);

    // 9. Add a dynamic Vignette (darkens outer edges)
    float vignette = smoothstep(1.0, 0.2, radius);
    final_color *= vignette;

    // 10. Draw glowing neon geometric lines on the structural folds that light up on snare/mids
    float fold_edge = smoothstep(0.02, 0.0, abs(angle - (slice / 2.0)));
    vec3 neon_color = vec3(0.2, 0.8, 1.0); // Cyan glow
    final_color += neon_color * fold_edge * audio_mid * sensitivity;

    FragColor = vec4(final_color, 1.0);
}
