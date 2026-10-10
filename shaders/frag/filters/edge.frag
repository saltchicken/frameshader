#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float sensitivity;
uniform float time;

// --- AUDIO UNIFORMS ---
uniform float audio_bass;
uniform float audio_mid;

void main() {
    // 1. Subtle Edge Thicken: Bass slightly pushes the sampling distance outward.
    // When audio_bass is 0.0, this is exactly 1.0 (the original baseline).
    float bass_pump = 1.0 + (audio_bass * 2.0); 
    vec2 texel = bass_pump / resolution;
    
    // Sample a 3x3 grid around the current pixel
    vec3 c11 = texture(videoTexture, TexCoord + vec2(-texel.x, -texel.y)).rgb;
    vec3 c12 = texture(videoTexture, TexCoord + vec2(0.0,      -texel.y)).rgb;
    vec3 c13 = texture(videoTexture, TexCoord + vec2( texel.x, -texel.y)).rgb;
    vec3 c21 = texture(videoTexture, TexCoord + vec2(-texel.x,  0.0)).rgb;
    vec3 c23 = texture(videoTexture, TexCoord + vec2( texel.x,  0.0)).rgb;
    vec3 c31 = texture(videoTexture, TexCoord + vec2(-texel.x,  texel.y)).rgb;
    vec3 c32 = texture(videoTexture, TexCoord + vec2(0.0,       texel.y)).rgb;
    vec3 c33 = texture(videoTexture, TexCoord + vec2( texel.x,  texel.y)).rgb;
    
    // Calculate gradients on X and Y axes
    vec3 sx = c13 + 2.0 * c23 + c33 - (c11 + 2.0 * c21 + c31);
    vec3 sy = c31 + 2.0 * c32 + c33 - (c11 + 2.0 * c12 + c13);
    
    // Magnitude of the edges
    float edge = length(sx) + length(sy);
    
    // 2. Subtle Color Shimmer: Mids slightly shift the time phase.
    // When audio_mid is 0.0, mid_shift is 0.0, keeping original colors.
    float mid_shift = audio_mid * 1.5;
    
    // Your EXACT original color palette
    vec3 edgeColor = vec3(
        0.5 + 0.5 * sin(time * 2.0 + TexCoord.x * 5.0 + mid_shift), 
        1.0, 
        0.8 + 0.2 * cos(time + TexCoord.y * 3.0 - mid_shift)
    );
    
    // 3. Subtle Glow: A tiny bump to brightness so it doesn't blow out
    float brightness_bump = 1.0 + (audio_bass * 0.3);
    
    vec3 finalColor = edgeColor * edge * sensitivity * brightness_bump;
    
    FragColor = vec4(finalColor, 1.0);
}
