#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float audio_bass;
uniform float audio_treble;

// Standard 8x8 Bayer Dither Matrix
const int dither[64] = int[64](
    0, 32, 8, 40, 2, 34, 10, 42,
    48, 16, 56, 24, 50, 18, 58, 26,
    12, 44, 4, 36, 14, 46, 6, 38,
    60, 28, 52, 20, 62, 30, 54, 22,
    3, 35, 11, 43, 1, 33, 9, 41,
    51, 19, 59, 27, 49, 17, 57, 25,
    15, 47, 7, 39, 13, 45, 5, 37,
    63, 31, 55, 23, 61, 29, 53, 21
);

void main() {
    vec2 uv = TexCoord;
    
    // Bass punches the camera inward slightly
    float zoom = 1.0 - (audio_bass * 0.08);
    uv = (uv - 0.5) * zoom + 0.5;

    vec3 col = texture(videoTexture, uv).rgb;
    float lum = dot(col, vec3(0.299, 0.587, 0.114));

    // Treble creates bright threshold flashes
    lum += (audio_treble * 0.15);
    
    // "Pixelate" the screen (down-res for that retro console look)
    // The pixel size increases during heavy bass hits
    float pixelSize = max(2.0, floor(2.0 + audio_bass * 4.0));
    vec2 pixelCoord = floor(uv * resolution / pixelSize);

    // Look up the Bayer matrix threshold
    int x = int(mod(pixelCoord.x, 8.0));
    int y = int(mod(pixelCoord.y, 8.0));
    float limit = (float(dither[x + y * 8]) + 1.0) / 65.0;

    // Apply the 1-bit threshold
    float res = step(limit, lum);

    // Classic Gameboy Green Palette
    vec3 dark = vec3(0.06, 0.22, 0.06);   // #0f380f
    vec3 light = vec3(0.54, 0.67, 0.06);  // #8bac0f

    FragColor = vec4(mix(dark, light, res), 1.0);
}
