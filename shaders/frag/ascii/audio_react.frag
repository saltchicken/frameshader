#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform sampler2D fontAtlas;

uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars;
uniform float time;

// Base sensitivity across the whole shader
uniform float sensitivity = 1.0; 

// --- AUDIO UNIFORMS ---
uniform float audio_bass;
uniform float audio_mid;
uniform float audio_treble;
uniform float audio_volume;

// --- INDEPENDENT GAIN PARAMS ---
// These default to 1.0/0.15 but are overridden by controller.json
uniform float bass_gain = 1.0;
uniform float mid_gain = 1.0;
uniform float treble_gain = 1.0;
uniform float vol_gain = 1.0;
uniform float noise_gate = 0.15;

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // 1. Audio Shaping & Curves
    // Smoothstep acts as a noise gate, Pow shapes the curve to hit sharp and fast
    float clean_bass = smoothstep(noise_gate, 1.0, audio_bass);
    float punchy_bass = clamp(pow(clean_bass, 2.5) * bass_gain * sensitivity, 0.0, 1.0);

    float clean_mid = smoothstep(noise_gate, 1.0, audio_mid);
    float punchy_mid = clamp(pow(clean_mid, 2.0) * mid_gain * sensitivity, 0.0, 1.0);

    float clean_treble = smoothstep(noise_gate, 1.0, audio_treble);
    float crisp_treble = clamp(pow(clean_treble, 3.0) * treble_gain * sensitivity, 0.0, 1.0);

    float clean_vol = smoothstep(noise_gate, 1.0, audio_volume);
    float punchy_vol = clamp(pow(clean_vol, 2.0) * vol_gain * sensitivity, 0.0, 1.0);

    // 2. Bass pumps the scale of the grid
    float pump = clamp(1.0 - (punchy_bass * 0.15), 0.7, 1.0);
    vec2 p = TexCoord - 0.5;
    vec2 pumpedTexCoord = (p * pump) + 0.5;

    // 3. Setup character grid mappings
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(pumpedTexCoord * characterGrid);
    
    // 4. Treble creates horizontal "glitch" displacement
    float glitch = 0.0;
    if (crisp_treble > 0.1) {
        if (random(vec2(charCoord.y, time)) > 0.85) {
            glitch = (random(charCoord) - 0.5) * crisp_treble * 0.1;
        }
    }
    
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    videoUV.x += glitch;
    
    // 5. Sample video color
    vec4 videoColor = texture(videoTexture, videoUV);

    // 6. Calculate base brightness, boosted by overall volume
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = brightness * (1.0 + punchy_vol * 0.8);
    
    // 7. Treble occasionally scrambles the character selection entirely
    if (glitch != 0.0) {
        boostedBrightness = random(charCoord + time);
    }

    // 8. Map brightness to character index
    float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
    float charIndex = floor(clampedBrightness * (numChars - 1.0));

    // 9. Sample the font atlas mask
    vec2 intraCharUV = fract(pumpedTexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;

    // 10. Audio-reactive Color Tinting
    vec3 color = videoColor.rgb;
    
    color.r += punchy_bass * 0.6;   // Bass = Red boost
    color.g += punchy_mid * 0.5;    // Mids = Green boost
    color.b += crisp_treble * 0.7;  // Treble = Blue/electric boost
    
    // Make loud moments glow brighter
    color *= (1.0 + punchy_vol * 0.5);

    FragColor = vec4(color * fontMask, 1.0);
}
