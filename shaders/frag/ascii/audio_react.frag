#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform sampler2D fontAtlas;

uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars;
uniform float time;
uniform float sensitivity = 1.0; 

// --- AUDIO UNIFORMS ---
uniform float audio_bass;
uniform float audio_mid;
uniform float audio_treble;
uniform float audio_volume;

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    // 1. Bass pumps the scale of the grid
    float pump = 1.0 - (audio_bass * 0.15 * sensitivity);
    vec2 p = TexCoord - 0.5;
    vec2 pumpedTexCoord = (p * pump) + 0.5;

    // 2. Setup character grid mappings
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(pumpedTexCoord * characterGrid);
    
    // 3. Treble creates horizontal "glitch" displacement
    float glitch = 0.0;
    if (audio_treble > 0.3 * sensitivity) {
        // High frequency noise threshold
        if (random(vec2(charCoord.y, time)) > 0.85) {
            glitch = (random(charCoord) - 0.5) * audio_treble * 0.1;
        }
    }
    
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    videoUV.x += glitch;
    
    // 4. Sample video color
    vec4 videoColor = texture(videoTexture, videoUV);

    // 5. Calculate base brightness, boosted by overall volume
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = brightness * sensitivity * (1.0 + audio_volume * 0.8);
    
    // 6. Treble occasionally scrambles the character selection entirely
    if (glitch != 0.0) {
        boostedBrightness = random(charCoord + time);
    }

    // 7. Map brightness to character index
    float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
    float charIndex = floor(clampedBrightness * (numChars - 1.0));

    // 8. Sample the font atlas mask
    vec2 intraCharUV = fract(pumpedTexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    float fontMask = texture(fontAtlas, fontUV).r;

    // 9. Audio-reactive Color Tinting
    vec3 color = videoColor.rgb;
    
    // Push colors dynamically based on frequency bands
    color.r += audio_bass * 0.6 * sensitivity;      // Bass = Red boost
    color.g += audio_mid * 0.5 * sensitivity;       // Mids = Green boost
    color.b += audio_treble * 0.7 * sensitivity;    // Treble = Blue/electric boost
    
    // Make loud moments glow brighter
    color *= (1.0 + audio_volume * 0.5);

    FragColor = vec4(color * fontMask, 1.0);
}
