#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D videoTexture; // This is now used again for the foreground color
uniform sampler2D fontAtlas;    
uniform sampler2D maskTexture;  

uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars = 10.0;
uniform float maskThreshold = 1.0;
uniform float time; 

// A simple pseudo-random number generator
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

void main()
{
    // Calculate the texture coordinate for the center of the character cell
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;

    // Sample the mask to decide if we are in the foreground or background
    float maskValue = texture(maskTexture, videoUV).r; 

    if (maskValue < maskThreshold) {
        // --- BACKGROUND ---
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return; 
    }

    // --- FOREGROUND (PERSON) ---
    // The Matrix rain animation logic remains the same.
    
    // 1. Define effect parameters
    float rainSpeed = 10.0;
    float tailLength = 100.0;

    // 2. Get a random value for this column
    float columnRand = random(vec2(charCoord.x, 1.0));

    // 3. Calculate the Y position of the "head" of the stream
    float streamProgress = fract((time * 0.1 * rainSpeed * (columnRand * 0.5 + 0.5)) + columnRand);
    float headY = floor(streamProgress * characterGrid.y);

    // 4. Calculate the vertical distance of the current character from the head
    float distFromHead = mod(charCoord.y - headY, characterGrid.y);

    // 5. Calculate the character's intensity based on its distance
    float intensity = 1.0 - (distFromHead / tailLength);
    intensity = clamp(intensity, 0.0, 1.0);

    // 6. If intensity is zero, make it black.
    if (intensity <= 0.001) {
         FragColor = vec4(0.0, 0.0, 0.0, 1.0);
         return;
    }

    // 7. Select a random character
    float charIndex = floor(random(charCoord + time * 2.0) * numChars);

    // 8. Sample the font atlas for the character shape.
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    vec4 fontColor = texture(fontAtlas, fontUV); // The character shape mask

    // *** NEW: Get color from the video feed ***
    vec4 videoColor = texture(videoTexture, videoUV);

    // *** NEW: Combine font shape, video color, and rain intensity ***
    // The intensity from the rain effect now controls the brightness of the video color.
    FragColor = fontColor * videoColor * intensity;
}
