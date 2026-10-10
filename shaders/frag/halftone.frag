#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float sensitivity;
uniform float audio_bass;

// Base grid size in pixels
const float DOT_GRID_SIZE = 10.0;

void main() {
    // Bass pushes the grid size slightly outwards (zooms the dots)
    float current_grid_size = max(4.0, DOT_GRID_SIZE - (audio_bass * 3.0));

    // Calculate grid layout
    vec2 grid_uv = TexCoord * resolution / current_grid_size;
    vec2 cell_uv = floor(grid_uv) + 0.5; // Center of the current cell
    vec2 local_uv = fract(grid_uv) - 0.5; // Local coordinates (-0.5 to 0.5)

    // Sample the video feed at the center of the cell
    vec2 sample_uv = cell_uv * current_grid_size / resolution;
    vec4 color = texture(videoTexture, sample_uv);

    // Calculate perceived brightness
    float brightness = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    brightness = clamp(brightness * sensitivity, 0.0, 1.0);

    // Determine dot radius based on brightness
    // Using sqrt gives a better visual mapping for area coverage
    float max_radius = 0.707; // sqrt(0.5^2 + 0.5^2), allows dots to touch/merge
    float radius = sqrt(brightness) * max_radius;

    // Draw the circle (anti-aliased using smoothstep)
    float dist = length(local_uv);
    float alpha = 1.0 - smoothstep(radius - 0.05, radius + 0.05, dist);

    // Background color (paper) is dark grey, dots take the camera color
    vec3 paperColor = vec3(0.05, 0.05, 0.05);
    vec3 finalColor = mix(paperColor, color.rgb, alpha);

    FragColor = vec4(finalColor, 1.0);
}
