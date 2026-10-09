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
// --- MODIFIED ---
uniform float rain_speed = 0.3;   // Now a uniform
uniform float tail_length = 0.25; // Now a uniform

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);
    vec3 baseColor = videoColor.rgb;

    float col_x = charCoord.x;
    float rand_speed_mult = 0.5 + random(vec2(col_x, 0.0)) * 1.5;
    float rand_offset = random(vec2(col_x, 1.0)) * 10.0;
    
    // UPDATED to use uniform
    float head_y = fract((time * rain_speed * rand_speed_mult) + rand_offset);
    
    float current_y = (charCoord.y + 0.5) / characterGrid.y;
    float dist = mod(current_y - head_y + 1.0, 1.0);
    
    float intensity = 0.0;
    vec3 rainColor = vec3(0.0);
    
    // UPDATED to use uniform
    if (dist < tail_length) {
        // UPDATED to use uniform
        intensity = 1.0 - (dist / tail_length);
        if (intensity > 0.95) {
            rainColor = baseColor * (1.0 + (intensity - 0.95) * 2.0);
        } else {
            rainColor = baseColor * intensity;
        }
    }

    float time_slice = floor(time * 5.0);
    float charIndex = floor(random(charCoord.xy + time_slice) * numChars);

    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;

    FragColor = vec4(rainColor * fontMask, 1.0);
}
