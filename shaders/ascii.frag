#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D videoTexture; // Texture unit 0: The camera feed
uniform sampler2D fontAtlas;    // Texture unit 1: The font atlas image

uniform vec2 resolution;        // Resolution of the camera feed (e.g., 1920x1080)
uniform vec2 charSize;          // Size of one character cell (e.g., 8x16 pixels)

uniform float sensitivity = 1.0; // <-- ADD THIS SENSITIVITY UNIFORM

const float numChars = 10.0;

void main()
{
    // ... (steps 1-3 are the same)
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);

    // 4. Calculate the brightness (luminance) of the video color
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));

    // 5. Boost the brightness using our new sensitivity uniform
    float boostedBrightness = brightness * sensitivity;

    // 6. Select a character index, clamping the result to avoid errors
    float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
    float charIndex = floor(clampedBrightness * (numChars - 1.0));

    // ... (the rest of the shader is the same)
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    vec4 fontColor = texture(fontAtlas, fontUV);
    FragColor = fontColor * videoColor;
}
