#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity;

uniform float audio_bass;
uniform float audio_mid;
uniform float audio_treble;

void main() {
    vec2 uv = TexCoord;

    // 1. Edge Detection (Sobel) for the "Ink" outline
    vec2 texel = 1.0 / resolution;
    
    // Bass makes the ink outlines thicker and bleed out more
    float edge_spread = 1.0 + (audio_bass * 3.0);
    vec2 offset_x = vec2(texel.x * edge_spread, 0.0);
    vec2 offset_y = vec2(0.0, texel.y * edge_spread);

    // Sample the surrounding pixels to find contrast edges
    float s00 = length(texture(videoTexture, uv - offset_x - offset_y).rgb);
    float s01 = length(texture(videoTexture, uv - offset_x).rgb);
    float s02 = length(texture(videoTexture, uv - offset_x + offset_y).rgb);
    float s10 = length(texture(videoTexture, uv - offset_y).rgb);
    float s12 = length(texture(videoTexture, uv + offset_y).rgb);
    float s20 = length(texture(videoTexture, uv + offset_x - offset_y).rgb);
    float s21 = length(texture(videoTexture, uv + offset_x).rgb);
    float s22 = length(texture(videoTexture, uv + offset_x + offset_y).rgb);

    float sx = s00 + 2.0 * s10 + s20 - (s02 + 2.0 * s12 + s22);
    float sy = s00 + 2.0 * s01 + s02 - (s20 + 2.0 * s21 + s22);
    float edge = length(vec2(sx, sy));

    // Threshold the gradient to make it a hard black ink line
    float ink = step(0.6 - (audio_bass * 0.15), edge * sensitivity);

    // 2. Color Posterization (Cel-shading)
    vec3 color = texture(videoTexture, uv).rgb;
    
    // Mids dictate how many color bands there are (lower mids = fewer, blockier colors)
    float bands = 5.0 + floor(audio_mid * 6.0);
    color = floor(color * bands) / bands;

    // Boost the saturation heavily for that pop-art graphic novel feel
    float lum = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(lum), color, 1.8 + (audio_treble * 0.5));

    // 3. Halftone dot pattern in the shadows
    float dot_size = 350.0;
    vec2 dot_uv = uv * dot_size;
    
    // Treble slightly vibrates the halftone dots
    float dot_time = time * 2.0 + (audio_treble * 5.0);
    float dots = sin(dot_uv.x + dot_time) * sin(dot_uv.y + dot_time);
    
    // Mix the dots only into the mid-tones and shadows
    vec3 shaded_color = mix(color * 0.4, color, step(0.0, dots + (lum * 2.5 - 0.5)));

    // 4. Combine the colored fill with the black ink lines
    vec3 final_color = mix(shaded_color, vec3(0.05), ink);

    FragColor = vec4(final_color, 1.0);
}
