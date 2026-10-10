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
uniform float audio_treble;
uniform float audio_volume;
uniform float audio_peak_freq; // Raw frequency in Hz

// Utility to convert Hue/Saturation/Value to RGB for our pitch-to-color mapping
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    vec2 uv = TexCoord;
    
    // Center coordinates for radial math
    vec2 p = uv * 2.0 - 1.0;
    p.x *= resolution.x / resolution.y;
    float r = length(p);

    // 1. Map raw Hertz to a logarithmic perceived pitch (0.0 to 1.0)
    // Human hearing is logarithmic. log2(20Hz) ~ 4.3, log2(20000Hz) ~ 14.2
    float log_freq = log2(max(audio_peak_freq, 1.0));
    float norm_pitch = clamp((log_freq - 4.3) / 10.0, 0.0, 1.0); 

    // Smooth out the pitch slightly so the visuals don't jitter too violently
    // (In a real scenario you might smooth this in Rust, but we can ease it in GLSL)
    float pitch_factor = norm_pitch;

    // 2. Calculate Cymatic Ripples
    // High pitch = lots of tight rings (e.g. 60 rings). Low pitch = few wide rings (e.g. 5 rings)
    float ripple_count = 5.0 + (pitch_factor * 55.0); 
    
    // Animate the ripples flowing outward. Speed is influenced by volume.
    float phase = (time * 4.0) - (r * ripple_count);
    float ripple = sin(phase) * cos(phase * 0.33);

    // 3. Displace the UV coordinates based on the ripples
    // The louder the volume, the harder the image distorts
    float distortion_mag = (0.005 + (audio_volume * 0.04)) * sensitivity;
    vec2 displaced_uv = uv + (normalize(p) * ripple * distortion_mag);

    // Keep UVs bounded to avoid mirroring artifacts at the edges
    displaced_uv = clamp(displaced_uv, 0.0, 1.0);

    // 4. Base Video Sample
    vec3 col = texture(videoTexture, displaced_uv).rgb;

    // 5. Synesthesia Color Glow
    // Map the pitch to a Hue (Low = Red, Mid = Green, High = Blue/Magenta)
    vec3 pitch_color = hsv2rgb(vec3(1.0 - (pitch_factor * 0.85), 0.85, 1.0));
    
    // Only glow on the crests of the ripples
    float glow_mask = smoothstep(0.7, 1.0, ripple);
    col += pitch_color * glow_mask * (audio_volume * 1.5 * sensitivity);

    // 6. Treble-induced Chromatic Aberration
    // High frequencies cause the RGB channels to split radially
    float split = audio_treble * 0.015 * sensitivity * pitch_factor;
    if (split > 0.0) {
        float cr = texture(videoTexture, clamp(displaced_uv + normalize(p) * split, 0.0, 1.0)).r;
        float cb = texture(videoTexture, clamp(displaced_uv - normalize(p) * split, 0.0, 1.0)).b;
        
        // Blend the split into the edges of the screen
        float edge_blend = smoothstep(0.2, 1.0, r);
        col.r = mix(col.r, cr, edge_blend);
        col.b = mix(col.b, cb, edge_blend);
    }

    FragColor = vec4(col, 1.0);
}
