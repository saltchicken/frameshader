// shaders/ascii.frag
#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D videoTexture; // Texture unit 0: The camera feed
uniform sampler2D fontAtlas;    // Texture unit 1: The font atlas image
uniform sampler2D depthTexture; // Texture unit 2: The depth map

uniform vec2 resolution;
uniform vec2 charSize;
uniform float sensitivity = 1.0;
uniform float numChars = 10.0;
uniform float depthThreshold = 0.5;

void main()
{
    // Sample the depth map first to decide what to do
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    float depth = texture(depthTexture, videoUV).r;

    // Check if the pixel is in the background
    if (depth > depthThreshold) {
        // --- BACKGROUND ---
        // Set the output color directly to bright pink (R, G, B, A)
        FragColor = vec4(1.0, 0.0, 1.0, 1.0);
        // Exit the shader immediately for this pixel
        return;
    }

    // --- FOREGROUND ---
    // This code only runs if the 'if' condition above was false.
    // Sample the video color for the character cell
    vec4 videoColor = texture(videoTexture, videoUV);

    // Calculate brightness to select a character
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = brightness * sensitivity;
    float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
    float charIndex = floor(clampedBrightness * (numChars - 1.0));
 
    // Use the character index to find the right part of the font atlas
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    vec4 fontColor = texture(fontAtlas, fontUV);
 
    // Combine the font color and video color for the final output
    FragColor = fontColor * videoColor;
}
