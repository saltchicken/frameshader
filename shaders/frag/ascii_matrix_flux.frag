#version 460 core

out vec4 FragColor;
in vec2 TexCoord;

// --- UNIFORMS ---
uniform sampler2D videoTexture;
uniform sampler2D fontAtlas;
uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars;
uniform float time;

// --- CONFIGURABLE PARAMETERS ---
uniform float rain_speed = 0.4;
uniform float tail_length = 0.3;
uniform float flicker_speed = 15.0; // Controls how fast the tail characters change
uniform float sensitivity = 1.5;    // How much the camera brightness affects the rain

// --- COLORS ---
const vec3 HEAD_COLOR = vec3(0.8, 1.0, 0.8); // Bright white-green
const vec3 TAIL_COLOR = vec3(0.0, 0.9, 0.1); // Classic green

// --- UTILITY FUNCTIONS ---
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // --- STEP 1: Calculate character grid coordinates ---
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);

    // --- STEP 2: Calculate rain properties for this column ---
    float col_x = charCoord.x;
    float rand_speed_mult = 0.6 + random(vec2(col_x, 0.0)) * 1.4;
    float rand_offset = random(vec2(col_x, 1.0)) * 10.0;

    // Calculate the "head" of the raindrop for this column
    float head_y = fract((time * rain_speed * rand_speed_mult) + rand_offset);
    
    // Calculate the current pixel's vertical position relative to the head
    float current_y = (charCoord.y + 0.5) / characterGrid.y;
    float dist = mod(current_y - head_y + 1.0, 1.0);

    // --- STEP 3: Determine rain color and intensity based on distance from head ---
    vec3 rainColor = vec3(0.0);
    float intensity = 0.0;

    if (dist < tail_length) {
        // Intensity is highest at the head (1.0) and fades to 0.0 at the end of the tail
        intensity = 1.0 - (dist / tail_length);
        
        // Use HEAD_COLOR for the very tip of the rain drop, otherwise use TAIL_COLOR
        if (intensity > 0.95) {
            rainColor = HEAD_COLOR;
        } else {
            // Fade the tail color based on its intensity
            rainColor = TAIL_COLOR * intensity;
        }
    }

    // --- STEP 4: Select a "glitching" character ---
    // The flicker rate is higher for characters with lower intensity (further down the tail)
    float flicker_rate = (1.0 - intensity) * flicker_speed;
    float time_slice = floor(time * flicker_rate);
    float charIndex = floor(random(charCoord.xy + time_slice) * numChars);

    // --- STEP 5: Get camera feed brightness ---
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = clamp(brightness * sensitivity, 0.2, 1.0); // Keep a minimum brightness

    // --- STEP 6: Combine everything for the final color ---
    // Get the character shape from the font atlas
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;

    // Final color is the calculated rain color, multiplied by the camera's brightness,
    // and then masked by the character's shape.
    vec3 finalColor = rainColor * boostedBrightness;

    FragColor = vec4(finalColor * fontMask, 1.0);
}
