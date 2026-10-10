#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float audio_bass;
uniform float audio_treble;

float rand(vec2 n) { 
    return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}

void main() {
    vec2 uv = TexCoord;
    
    // Binocular / Pincushion lens distortion
    vec2 centered = uv - 0.5;
    float r = length(centered);
    
    // Bass flexes the lens curvature
    float lens_pump = 0.2 + (audio_bass * 0.15);
    uv = uv + centered * r * lens_pump;
    
    // Out of bounds safety
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    
    vec3 col = texture(videoTexture, uv).rgb;
    
    // Calculate Luminance and blow out the exposure
    float lum = dot(col, vec3(0.299, 0.587, 0.114));
    lum *= 2.5 + (audio_bass * 0.8); 
    
    // Apply classic NVG phosphor green
    vec3 nvColor = vec3(0.1, 1.0, 0.2) * lum;
    
    // Scanlines
    nvColor *= 0.85 + 0.15 * sin(uv.y * resolution.y * 2.0 + time * 10.0);
    
    // Treble triggers static noise bursts
    float noise = rand(uv * time);
    float noiseAmount = 0.1 + (audio_treble * 0.35);
    nvColor += noise * noiseAmount * vec3(0.1, 1.0, 0.2);
    
    // Heavy circular vignette
    float vignette = smoothstep(0.8, 0.3, r * 1.6);
    nvColor *= vignette;
    
    FragColor = vec4(nvColor, 1.0);
}
