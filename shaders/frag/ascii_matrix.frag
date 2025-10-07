// shaders/ascii.frag
#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

// --- UNIFORMS ---
uniform sampler2D videoTexture; // Still here to tint the rain
uniform sampler2D fontAtlas;    // The font texture
uniform vec2 resolution;        // Screen resolution
uniform vec2 charSize;          // Size of one character
uniform float numChars;         // Number of chars in the atlas
uniform float time;             // The new time uniform

// --- CONFIGURATION ---
const float RAIN_SPEED = 0.3;
const float TAIL_LENGTH = 0.25; // As a fraction of screen height
const vec3 HEAD_COLOR = vec3(0.7, 1.0, 0.7); // Bright light-green
const vec3 TAIL_COLOR = vec3(0.0, 1.0, 0.1); // Matrix green

// --- HELPER FUNCTION ---
// Simple pseudo-random number generator
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // --- 1. Calculate Character Grid Coordinates ---
    // This part is the same as your old shader
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid); // The (column, row) of this character cell

    // --- 2. Create Column-Specific Animation ---
    // Use the column's x-coordinate as a seed for randomness
    float col_x = charCoord.x;
    float rand_speed_mult = 0.5 + random(vec2(col_x, 0.0)) * 1.5; // Each column has a unique speed
    float rand_offset = random(vec2(col_x, 1.0)) * 10.0;         // Each column has a unique start time

    // Calculate the normalized vertical position (0-1) of the rain "head" for this column.
    // It moves from top (0.0) to bottom (1.0) and wraps around using fract().
    float head_y = fract((time * RAIN_SPEED * rand_speed_mult) + rand_offset);
    
    // --- 3. Determine Intensity and Color ---
    // Get the normalized y-position of the current character cell
    float current_y = (charCoord.y + 0.5) / characterGrid.y;

    // Calculate the distance of this character from the rain head.
    // We use mod() to handle the wrapping, ensuring a continuous loop.
    float dist = mod(current_y - head_y + 1.0, 1.0);
    
    float intensity = 0.0;
    vec3 rainColor = vec3(0.0);

    if (dist < TAIL_LENGTH) {
        // This character is part of a rain streak.
        // The intensity fades from 1.0 (at the head) to 0.0 (at the end of the tail).
        intensity = 1.0 - (dist / TAIL_LENGTH);

        // Check if this is the "head" character
        if (intensity > 0.95) {
            rainColor = HEAD_COLOR;
        } else {
            // Otherwise, it's part of the fading tail
            rainColor = TAIL_COLOR * intensity;
        }
    }

    // --- 4. Select a Random Character ---
    // The character changes based on its position and the current time
    float time_slice = floor(time * 5.0); // Characters change ~5 times per second
    float charIndex = floor(random(charCoord.xy + time_slice) * numChars);

    // --- 5. Sample Textures and Combine ---
    // Sample the character shape from the font atlas
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r; // Use red channel as a mask

    // Sample the original video color
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);

    // Final Color:
    // 1. Start with the calculated `rainColor`.
    // 2. Use `fontMask` to "cut out" the character shape.
    // 3. Multiply by the original `videoColor` to tint the rain.
    // The result is black for empty space, and tinted green for characters.
    FragColor = vec4(rainColor, 1.0) * fontMask * videoColor;
}
