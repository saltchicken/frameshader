#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity;

uniform float audio_bass;
uniform float audio_mid;

void main() {
    vec2 uv = TexCoord;

    // 1. Subtle rhythmic wave breathing
    uv.y += sin(uv.x * 10.0 + time * 2.0) * (0.005 + audio_bass * 0.015);

    // 2. Sample luminance as elevation/height
    vec4 src = texture(videoTexture, uv);
    float elevation = dot(src.rgb, vec3(0.299, 0.587, 0.114));
    elevation = pow(elevation, 1.2) * sensitivity;

    // 3. Contour lines (Iso-elevation rings)
    float contour_freq = 24.0 + audio_mid * 12.0;
    float contour = sin(elevation * contour_freq - time * 3.0);
    float lines = smoothstep(0.85, 0.98, contour);

    // 4. Perspective grid overlay (horizontal scanning bands)
    float grid_lines = step(0.92, fract(uv.y * 70.0 + elevation * 6.0));

    // 5. Synthetic neon elevation palette (cyan -> magenta -> gold)
    vec3 col_low  = vec3(0.05, 0.1, 0.3);
    vec3 col_mid  = vec3(0.0, 0.9, 0.85);  // Neon Cyan
    vec3 col_high = vec3(1.0, 0.1, 0.6);   // Hot Magenta
    vec3 col_peak = vec3(1.0, 0.9, 0.3);   // Bright Gold

    vec3 terrain_color = mix(col_low, col_mid, smoothstep(0.1, 0.45, elevation));
    terrain_color = mix(terrain_color, col_high, smoothstep(0.45, 0.8, elevation));
    terrain_color = mix(terrain_color, col_peak, smoothstep(0.8, 1.0, elevation));

    // 6. Combine glowing contours and background wireframe
    vec3 final_color = terrain_color * (lines * 2.0 + grid_lines * 0.6);

    // Add faint silhouette boost so dark features still retain context
    final_color += src.rgb * 0.12;

    // Bass punch glow
    final_color += terrain_color * audio_bass * lines * 1.5;

    FragColor = vec4(final_color, 1.0);
}
