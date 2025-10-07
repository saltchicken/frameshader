// shaders/ascii.frag - MODIFIED FOR VIDEO COLORS
#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

// --- UNIFORMS ---
uniform sampler2D videoTexture; // The camera feed - now crucial for color
uniform sampler2D fontAtlas;    // The font texture
uniform vec2 resolution;        // Screen resolution
uniform vec2 charSize;          // Size of one character
uniform float numChars;         // Number of chars in the atlas
uniform float time;             // The time uniform for animation

// --- CONFIGURATION ---
const float RAIN_SPEED = 0.3;
const float TAIL_LENGTH = 0.25; // As a fraction of screen height
// We'll calculate head and tail colors based on videoColor, so these are less critical
// const vec3 HEAD_COLOR = vec3(0.7, 1.0, 0.7); 
// const vec3 TAIL_COLOR = vec3(0.0, 1.0, 0.1); 

// --- HELPER FUNCTION ---
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // --- 1. Calculate Character Grid Coordinates ---
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);

    // --- NEW: Sample video color early for this character cell ---
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);
    vec3 baseColor = videoColor.rgb; // Use this as the base for our rain colors

    // --- 2. Create Column-Specific Animation ---
    float col_x = charCoord.x;
    float rand_speed_mult = 0.5 + random(vec2(col_x, 0.0)) * 1.5;
    float rand_offset = random(vec2(col_x, 1.0)) * 10.0;

    float head_y = fract((time * RAIN_SPEED * rand_speed_mult) + rand_offset);
    
    // --- 3. Determine Intensity and Color ---
    float current_y = (charCoord.y + 0.5) / characterGrid.y;
    float dist = mod(current_y - head_y + 1.0, 1.0);
    
    float intensity = 0.0;
    vec3 rainColor = vec3(0.0);

    if (dist < TAIL_LENGTH) {
        intensity = 1.0 - (dist / TAIL_LENGTH);

        // Apply video color, but make the head brighter
        if (intensity > 0.95) {
            // Head: Slightly boosted brightness of the video color
            rainColor = baseColor * (1.0 + (intensity - 0.95) * 2.0); // Boost brightness
        } else {
            // Tail: Fading video color
            rainColor = baseColor * intensity;
        }
    }

    // --- 4. Select a Random Character ---
    float time_slice = floor(time * 5.0);
    float charIndex = floor(random(charCoord.xy + time_slice) * numChars);

    // --- 5. Sample Textures and Combine ---
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;

    // Final Color:
    // Use the `rainColor` (which is now derived from `baseColor` / `videoColor`)
    // multiplied by `fontMask` to apply the character shape.
    // The alpha component should be 1.0 for opaque characters.
    FragColor = vec4(rainColor * fontMask, 1.0);
}
