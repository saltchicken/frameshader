#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float sensitivity;
uniform float audio_bass;
uniform float audio_mid;

// Kuwahara Filter (Oil Painting effect)
void main() {
    // The brush radius grows violently when the bass kicks
    int radius = int(3.0 + (audio_bass * 6.0 * sensitivity) + (audio_mid * 2.0));
    vec2 texel = 1.0 / resolution;
    
    vec3 m[4];
    vec3 s[4];
    for (int k = 0; k < 4; ++k) {
        m[k] = vec3(0.0);
        s[k] = vec3(0.0);
    }
    
    // Define the 4 quadrants of the Kuwahara filter window
    int x_start[4] = int[](-radius, -radius, 0, 0);
    int y_start[4] = int[](-radius, 0, -radius, 0);
    int x_end[4] = int[](0, 0, radius, radius);
    int y_end[4] = int[](0, radius, 0, radius);
    
    float n = float((radius + 1) * (radius + 1));
    
    // Calculate mean and variance for each quadrant
    for (int k = 0; k < 4; ++k) {
        for (int j = y_start[k]; j <= y_end[k]; ++j) {
            for (int i = x_start[k]; i <= x_end[k]; ++i) {
                vec3 c = texture(videoTexture, TexCoord + vec2(i, j) * texel).rgb;
                m[k] += c;
                s[k] += c * c;
            }
        }
        m[k] /= n;
        s[k] = abs(s[k] / n - m[k] * m[k]);
    }
    
    // Select the quadrant with the lowest variance (preserves edges)
    float min_sigma2 = 1e20;
    vec3 final_color = vec3(0.0);
    for (int k = 0; k < 4; ++k) {
        float sigma2 = s[k].r + s[k].g + s[k].b;
        if (sigma2 < min_sigma2) {
            min_sigma2 = sigma2;
            final_color = m[k];
        }
    }
    
    FragColor = vec4(final_color, 1.0);
}
