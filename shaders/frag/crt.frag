#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity; // Reusing this to control RGB shift

// Creates the classic bulging screen effect
vec2 curve(vec2 uv) {
    uv = (uv - 0.5) * 2.0;
    uv *= 1.1; 
    uv.x *= 1.0 + pow((abs(uv.y) / 5.0), 2.0);
    uv.y *= 1.0 + pow((abs(uv.x) / 4.0), 2.0);
    uv  = (uv / 2.0) + 0.5;
    uv =  uv * 0.92 + 0.04;
    return uv;
}

void main() {
    vec2 uv = curve(TexCoord);
    
    // Black out the pixels outside the curved screen
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        FragColor = vec4(0.05, 0.05, 0.05, 1.0);
        return;
    }
    
    // Chromatic aberration (RGB shift)
    float shift = 0.003 * sensitivity;
    float r = texture(videoTexture, uv + vec2(shift, 0.0)).r;
    float g = texture(videoTexture, uv).g;
    float b = texture(videoTexture, uv - vec2(shift, 0.0)).b;
    vec3 col = vec3(r, g, b);

    // Moving Scanlines
    col *= 0.9 + 0.1 * sin(10.0 * time + uv.y * resolution.y * 2.0);

    // Vignette (darken corners)
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    col *= clamp(pow(16.0 * vignette, 0.25), 0.0, 1.0);

    FragColor = vec4(col, 1.0);
}
