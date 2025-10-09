#version 460 core

out vec4 FragColor;
in vec2 TexCoord;

// --- UNIFORMS ---
uniform sampler2D videoTexture; // Original video
uniform sampler2D fontAtlas;    // Font atlas for characters
uniform sampler2D maskTexture;  // Segmentation mask (person vs background)

uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars;
uniform float time;

// --- CONFIGURABLE PARAMETERS (from your matrix shader) ---
uniform float rain_speed = 0.4;
uniform float tail_length = 0.3;
uniform float flicker_speed = 15.0; 
uniform float sensitivity = 1.5;

// --- COLORS ---
const vec3 HEAD_COLOR = vec3(0.8, 1.0, 0.8); // Bright white-green
const vec3 TAIL_COLOR = vec3(0.0, 0.9, 0.1); // Classic green

// --- UTILITY FUNCTION ---
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // --- STEP 1: Calculate the Matrix digital rain effect ---
    // This section generates the animated rain for the entire screen,
    // using the video's brightness to influence the rain's brightness.

    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);

    float col_x = charCoord.x;
    float rand_speed_mult = 0.6 + random(vec2(col_x, 0.0)) * 1.4;
    float rand_offset = random(vec2(col_x, 1.0)) * 10.0;
    float head_y = fract((time * rain_speed * rand_speed_mult) + rand_offset);
    
    float current_y = (charCoord.y + 0.5) / characterGrid.y;
    float dist = mod(current_y - head_y + 1.0, 1.0);

    vec3 rainColor = vec3(0.0);
    float intensity = 0.0;
    if (dist < tail_length) {
        intensity = 1.0 - (dist / tail_length);
        if (intensity > 0.95) {
            rainColor = HEAD_COLOR;
        } else {
            rainColor = TAIL_COLOR * intensity;
        }
    }

    float flicker_rate = (1.0 - intensity) * flicker_speed;
    float time_slice = floor(time * flicker_rate);
    float charIndex = floor(random(charCoord.xy + time_slice) * numChars);

    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    float brightness = dot(texture(videoTexture, videoUV).rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = clamp(brightness * sensitivity, 0.2, 1.0); 

    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;
    
    vec4 matrixEffectColor = vec4(rainColor * boostedBrightness * fontMask, 1.0);

    // --- STEP 2: Get the original, unmodified video color ---
    vec4 originalVideoColor = texture(videoTexture, TexCoord);

    // --- STEP 3: Get the mask value to differentiate foreground and background ---
    // A value near 1.0 means foreground (the person).
    // A value near 0.0 means background.
    float maskValue = texture(maskTexture, TexCoord).r;

    // --- STEP 4: Mix the two effects using the mask ---
    // The mix() function blends between the matrix effect (for the background)
    // and the original video (for the foreground) based on the mask value.
    FragColor = mix(matrixEffectColor, originalVideoColor, maskValue);
}
