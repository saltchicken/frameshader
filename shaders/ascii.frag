#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D videoTexture; // Texture unit 0: The camera feed (unused in foreground)
uniform sampler2D fontAtlas;    // Texture unit 1: The font atlas image
uniform sampler2D maskTexture;  // Texture unit 2: The person segmentation mask

uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars = 10.0;
uniform float maskThreshold = 1.0;
uniform float time; // New uniform to drive animation

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

    // If the mask value is below our threshold, it's the background.
    if (maskValue < maskThreshold) {
        // --- BACKGROUND ---
        // For the matrix effect, a plain black background is best.
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return; 
    }

    // --- FOREGROUND (PERSON) ---
    // This is the new Matrix rain logic.
    
    // 1. Define effect parameters (you can tweak these!)
    float rainSpeed = 4.0;
    float tailLength = 15.0;

    // 2. Get a random value for this column to vary speed and starting offset
    float columnRand = random(vec2(charCoord.x, 1.0));

    // 3. Calculate the Y position of the "head" of the stream for this column.
    // It moves down over time and wraps around using fract().
    // The random value makes each column's stream unique.
    float streamProgress = fract((time * 0.1 * rainSpeed * (columnRand * 0.5 + 0.5)) + columnRand);
    float headY = floor(streamProgress * characterGrid.y);

    // 4. Calculate the vertical distance of the current character from the head.
    // We use mod() to handle the wrapping when the head goes off the bottom of the screen.
    float distFromHead = mod(charCoord.y - headY, characterGrid.y);

    // 5. Calculate the character's intensity based on its distance from the head.
    // This creates the fading tail effect.
    float intensity = 1.0 - (distFromHead / tailLength);
    intensity = clamp(intensity, 0.0, 1.0);

    // 6. If the intensity is zero, this character isn't part of a stream, so we make it black.
    if (intensity <= 0.001) {
         FragColor = vec4(0.0, 0.0, 0.0, 1.0);
         return;
    }

    // 7. Select a random character that flickers over time.
    float charIndex = floor(random(charCoord + time * 2.0) * numChars);

    // 8. Sample the font atlas to get the character's shape.
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    vec4 fontColor = texture(fontAtlas, fontUV); // This acts as a mask (e.g., white character on black)

    // 9. Determine the final color for the rain.
    // The head of the stream (intensity=1.0) is a bright color.
    // The tail fades to a darker color.
    vec3 headColor = vec3(0.8, 1.0, 0.8); // Light green/white
    vec3 tailColor = vec3(0.0, 0.5, 0.1); // Dark green
    vec3 finalColor = mix(tailColor, headColor, intensity);

    // 10. Combine the font shape with the calculated rain color.
    FragColor = fontColor * vec4(finalColor, 1.0);
}
