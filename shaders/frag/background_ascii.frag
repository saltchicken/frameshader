#version 460 core

out vec4 FragColor;
in vec2 TexCoord;

// --- UNIFORMS ---
uniform sampler2D videoTexture; // Texture unit 0: Camera feed
uniform sampler2D fontAtlas;    // Texture unit 1: Font atlas
uniform sampler2D maskTexture;  // Texture unit 2: Segmentation mask

uniform vec2 resolution;
uniform vec2 charSize;
uniform float numChars;
uniform float sensitivity = 1.0;

void main() {
    // --- STEP 1: Get the mask value ---
    // The mask is a single-channel texture, so we only need the .r component.
    // A high value (near 1.0) means foreground, a low value (near 0.0) means background.
    float maskValue = texture(maskTexture, TexCoord).r;

    // --- STEP 2: Calculate the ASCII effect color (same as ascii.frag) ---
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColorForAscii = texture(videoTexture, videoUV);

    float brightness = dot(videoColorForAscii.rgb, vec3(0.2126, 0.7152, 0.0722));
    float boostedBrightness = brightness * sensitivity;
    float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
    float charIndex = floor(clampedBrightness * (numChars - 1.0));

    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    vec4 fontColor = texture(fontAtlas, fontUV);

    vec4 asciiEffectColor = fontColor * videoColorForAscii;

    // --- STEP 3: Get the original video color ---
    vec4 originalVideoColor = texture(videoTexture, TexCoord);

    // --- STEP 4: Mix the two colors using the mask ---
    // The mix() function blends between two values.
    // mix(background, foreground, foreground_alpha)
    // When maskValue is 0.0, the result is asciiEffectColor.
    // When maskValue is 1.0, the result is originalVideoColor.
    FragColor = mix(asciiEffectColor, originalVideoColor, maskValue);
}
