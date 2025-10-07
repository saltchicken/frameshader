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
uniform float rain_speed = 0.3;
uniform float tail_length = 0.25;
uniform float sensitivity = 2.0; // NEW: Controls brightness reaction

const vec3 HEAD_COLOR = vec3(0.7, 1.0, 0.7);
const vec3 TAIL_COLOR = vec3(0.0, 1.0, 0.1);

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // ... (Sections 1, 2, and 3 for calculating rainColor are the same) ...
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);

    float col_x = charCoord.x;
    float rand_speed_mult = 0.5 + random(vec2(col_x, 0.0)) * 1.5;
    float rand_offset = random(vec2(col_x, 1.0)) * 10.0;
    float head_y = fract((time * rain_speed * rand_speed_mult) + rand_offset);
    
    float current_y = (charCoord.y + 0.5) / characterGrid.y;
    float dist = mod(current_y - head_y + 1.0, 1.0);
    
    vec3 rainColor = vec3(0.0);
    if (dist < tail_length) {
        float intensity = 1.0 - (dist / tail_length);
        if (intensity > 0.95) {
            rainColor = HEAD_COLOR;
        } else {
            rainColor = TAIL_COLOR * intensity;
        }
    }
    
    // ... (Section 4 for selecting a character is the same) ...
    float time_slice = floor(time * 5.0);
    float charIndex = floor(random(charCoord.xy + time_slice) * numChars);

    // --- MODIFIED FINAL COLOR CALCULATION ---

    // 1. Sample the font character shape
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;

    // 2. Get the camera feed color for this spot
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);

    // 3. Convert the camera color to a single brightness value (0.0 to 1.0)
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = clamp(brightness * sensitivity, 0.0, 1.0);

    // 4. Create the final color
    // We multiply the PURE GREEN rainColor by the camera's BRIGHTNESS
    vec3 finalColor = rainColor * boostedBrightness;

    // 5. Apply the character mask
    FragColor = vec4(finalColor, 1.0) * fontMask;
}
