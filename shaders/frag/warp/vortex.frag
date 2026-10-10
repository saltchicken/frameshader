#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity = 1.0;

// --- AUDIO UNIFORMS ---
uniform float audio_bass;
uniform float audio_mid;
uniform float audio_treble;
uniform float audio_volume;

// --- INDEPENDENT GAIN PARAMS ---
uniform float bass_gain = 1.0;
uniform float mid_gain = 1.0;
uniform float treble_gain = 1.0;
uniform float vol_gain = 1.0;
uniform float noise_gate = 0.15;

// --- CONFIGURABLE PARAMS ---
uniform float swirl_strength = 6.0;
uniform float vortex_radius = 1.2;
uniform float spin_speed = 0.5;

void main() {
    // 1. Audio Shaping & Curves
    float clean_bass = smoothstep(noise_gate, 1.0, audio_bass);
    float punchy_bass = clamp(pow(clean_bass, 2.0) * bass_gain, 0.0, 1.0);

    float clean_mid = smoothstep(noise_gate, 1.0, audio_mid);
    float punchy_mid = clamp(pow(clean_mid, 2.0) * mid_gain, 0.0, 1.0);

    float clean_treble = smoothstep(noise_gate, 1.0, audio_treble);
    float crisp_treble = clamp(pow(clean_treble, 2.0) * treble_gain, 0.0, 1.0);

    float clean_vol = smoothstep(noise_gate, 1.0, audio_volume);
    float punchy_vol = clamp(pow(clean_vol, 2.0) * vol_gain, 0.0, 1.0);

    // 2. Center coordinate space and correct aspect ratio
    vec2 p = TexCoord * 2.0 - 1.0;
    float aspect = resolution.x / resolution.y;
    p.x *= aspect;

    // 3. Convert to polar coordinates
    float r = length(p);
    float theta = atan(p.y, p.x);

    // 4. Audio-reactive vortex strength
    // Bass makes the twist coil tighter/deeper
    float dynamic_swirl = swirl_strength * sensitivity;
    dynamic_swirl += (punchy_bass * 5.0 * sensitivity);

    // Mids drive the base continuous rotation speed
    float current_spin = (time * spin_speed) + (punchy_mid * sensitivity * 0.5);

    // 5. Calculate the twist (exponential falloff so the edges stay relatively intact)
    float twist = dynamic_swirl * exp(-r * (3.0 / vortex_radius));
    theta += twist + current_spin;

    // 6. Convert back to Cartesian coordinates
    vec2 warped_p = vec2(cos(theta), sin(theta)) * r;

    // Un-fix aspect ratio and map back to 0.0 -> 1.0 UV space
    warped_p.x /= aspect;
    vec2 uv = warped_p * 0.5 + 0.5;

    // 7. Treble-induced Radial Chromatic Aberration
    // High frequencies split the RGB channels apart inside the vortex
    float split = crisp_treble * 0.03 * sensitivity * twist * 0.1;
    
    // Use fract() to repeat the texture playfully as it spirals out of bounds
    vec3 col;
    if (split > 0.0) {
        col.r = texture(videoTexture, fract(uv + normalize(warped_p) * split)).r;
        col.g = texture(videoTexture, fract(uv)).g;
        col.b = texture(videoTexture, fract(uv - normalize(warped_p) * split)).b;
    } else {
        col = texture(videoTexture, fract(uv)).rgb;
    }

    // 8. Add an "Event Horizon" center void
    // Sucks the light out of the exact center, glowing brightly on the edge when the volume hits
    float void_mask = smoothstep(0.02, 0.15, r);
    col *= void_mask; 
    
    // Synesthesia glow near the void edge pushed by volume
    float accretion_disk = smoothstep(0.2, 0.05, r) * smoothstep(0.0, 0.08, r);
    vec3 glow_color = vec3(0.2, 0.6, 1.0); // Electric blue
    col += glow_color * accretion_disk * punchy_vol * 3.0 * sensitivity;

    FragColor = vec4(col, 1.0);
}
