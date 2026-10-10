#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform float sensitivity;

// Audio uniforms
uniform float audio_bass;

// Maps a 0.0 -> 1.0 heat value to a Thermal color palette
vec3 getThermalColor(float t) {
    vec3 c1 = vec3(0.0, 0.0, 0.4); // Dark blue
    vec3 c2 = vec3(0.5, 0.0, 0.5); // Purple
    vec3 c3 = vec3(1.0, 0.1, 0.0); // Red
    vec3 c4 = vec3(1.0, 0.8, 0.0); // Yellow/Orange
    vec3 c5 = vec3(1.0, 1.0, 1.0); // White

    if (t < 0.25) return mix(c1, c2, t / 0.25);
    if (t < 0.50) return mix(c2, c3, (t - 0.25) / 0.25);
    if (t < 0.75) return mix(c3, c4, (t - 0.50) / 0.25);
    return mix(c4, c5, (t - 0.75) / 0.25);
}

void main() {
    vec3 col = texture(videoTexture, TexCoord).rgb;
    
    // Calculate raw brightness
    float lum = dot(col, vec3(0.299, 0.587, 0.114));
    
    // Bass spikes the heat signature, sensitivity acts as baseline exposure
    float heat = (lum * sensitivity) + (audio_bass * 0.3);
    
    FragColor = vec4(getThermalColor(clamp(heat, 0.0, 1.0)), 1.0);
}
