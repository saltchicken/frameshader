#version 460 core

out vec4 FragColor;
in vec2 TexCoord;

// --- UNIFORMS (used by one or both effects) ---
uniform sampler2D videoTexture; 
uniform sampler2D fontAtlas;    
uniform sampler2D maskTexture;  
uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars;
uniform float time;

// --- CONFIGURABLE PARAMETERS ---
uniform float rain_speed = 0.4;
uniform float tail_length = 0.3;
uniform float flicker_speed = 15.0; 
uniform float sensitivity = 1.5; // Affects brightness for both effects

// --- COLORS (for Matrix effect) ---
const vec3 HEAD_COLOR = vec3(0.8, 1.0, 0.8);
const vec3 TAIL_COLOR = vec3(0.0, 0.9, 0.1);

// --- UTILITY FUNCTION ---
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // --- Common calculations for both ASCII effects ---
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColorForCell = texture(videoTexture, videoUV);
    float brightness = dot(videoColorForCell.rgb, vec3(0.2126, 0.7152, 0.0722));


    // --- STEP 1: Calculate the Background (Matrix digital rain effect) ---
    vec4 matrixEffectColor;
    {
        // Rain properties for this column
        float col_x = charCoord.x;
        float rand_speed_mult = 0.6 + random(vec2(col_x, 0.0)) * 1.4;
        float rand_offset = random(vec2(col_x, 1.0)) * 10.0;
        float head_y = fract((time * rain_speed * rand_speed_mult) + rand_offset);
        float current_y = (charCoord.y + 0.5) / characterGrid.y;
        float dist = mod(current_y - head_y + 1.0, 1.0);

        // Rain color and intensity based on distance from head
        vec3 rainColor = vec3(0.0);
        float intensity = 0.0;
        if (dist < tail_length) {
            intensity = 1.0 - (dist / tail_length);
            rainColor = (intensity > 0.95) ? HEAD_COLOR : (TAIL_COLOR * intensity);
        }

        // Select a flickering character for the rain
        float flicker_rate = (1.0 - intensity) * flicker_speed;
        float time_slice = floor(time * flicker_rate);
        float charIndex = floor(random(charCoord.xy + time_slice) * numChars);
        
        // Boost brightness based on video, get font mask
        float boostedBrightness = clamp(brightness * sensitivity, 0.2, 1.0); 
        vec2 intraCharUV = fract(TexCoord * characterGrid);
        float atlasX = (charIndex + intraCharUV.x) / numChars;
        vec2 fontUV = vec2(atlasX, intraCharUV.y);
        float fontMask = texture(fontAtlas, fontUV).r;
        
        matrixEffectColor = vec4(rainColor * boostedBrightness * fontMask, 1.0);
    }


    // --- STEP 2: Calculate the Foreground (Standard ASCII effect) ---
    vec4 asciiEffectColor;
    {
        // Select character based on cell brightness
        float boostedBrightness = brightness * sensitivity;
        float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
        float charIndex = floor(clampedBrightness * (numChars - 1.0));

        // Sample the character from the font atlas
        vec2 intraCharUV = fract(TexCoord * characterGrid);
        float atlasX = (charIndex + intraCharUV.x) / numChars;
        vec2 fontUV = vec2(atlasX, intraCharUV.y);
        vec4 fontShape = texture(fontAtlas, fontUV);

        // Final color is the character shape multiplied by the cell's original color
        asciiEffectColor = fontShape * videoColorForCell;
    }


    // --- STEP 3: Get the mask value ---
    float maskValue = texture(maskTexture, TexCoord).r;

    // --- STEP 4: Mix the two effects using the mask ---
    // Background is matrix rain, Foreground is standard ASCII
    FragColor = mix(matrixEffectColor, asciiEffectColor, maskValue);
}
