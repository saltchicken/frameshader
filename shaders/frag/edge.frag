#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float sensitivity;
uniform float time;

void main() {
    vec2 texel = 1.0 / resolution;
    
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
    
    // Create a scrolling gradient using UV coordinates and time
    vec3 edgeColor = vec3(
        0.5 + 0.5 * sin(time * 2.0 + TexCoord.x * 5.0), 
        1.0, 
        0.8 + 0.2 * cos(time + TexCoord.y * 3.0)
    );
    
    vec3 finalColor = edgeColor * edge * sensitivity;
    
    FragColor = vec4(finalColor, 1.0);
}
