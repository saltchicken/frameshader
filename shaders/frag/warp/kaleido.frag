#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity = 1.0;

// --- AUDIO UNIFORMS ---
uniform float audio_bass;
uniform float audio_mid;
uniform float audio_treble;

// --- CONFIGURABLE PARAMS ---
// Defaults set directly in the shader!
uniform float segments = 6.0;
uniform float spin_speed = 0.2;

#define PI 3.141592653589793

void main() {
    // 1. Center coordinate space and correct aspect ratio
    vec2 p = TexCoord * 2.0 - 1.0;
    p.x *= resolution.x / resolution.y;

    // 2. Convert to polar coordinates
    float r = length(p);
    float theta = atan(p.y, p.x);

    // 3. Audio-reactive spin and zoom
    // Mids drive the rotation speed, bass pumps the zoom
    float spin = time * spin_speed + (audio_mid * 0.5 * sensitivity);
    theta += spin;

    float zoom = 1.0 - (audio_bass * 0.15 * sensitivity);
    r *= zoom;

    // 4. Kaleidoscope folding math
    float slice = (PI * 2.0) / segments;
    
    // Wrap the angle into a single slice, then mirror it so edges align seamlessly
    theta = mod(theta, slice);
    theta = abs(theta - (slice / 2.0));

    // 5. Convert back to Cartesian coordinates
    vec2 folded_p = vec2(cos(theta), sin(theta)) * r;
    
    // Un-fix aspect ratio and map back to 0.0 -> 1.0 UV space
    folded_p.x *= resolution.y / resolution.x;
    vec2 uv = folded_p * 0.5 + 0.5;

    // 6. Treble-induced Radial Chromatic Aberration
    // High frequencies split the RGB channels apart towards the edges of the screen
    float split = audio_treble * 0.03 * sensitivity * r;
    
    // Use fract() to repeat the texture if the zoom pulls UVs out of bounds
    float cr = texture(videoTexture, fract(uv + normalize(folded_p) * split)).r;
    float cg = texture(videoTexture, fract(uv)).g;
    float cb = texture(videoTexture, fract(uv - normalize(folded_p) * split)).b;
    
    vec3 col = vec3(cr, cg, cb);

    // 7. Add a subtle vignette based on radius to deepen the folds
    col *= smoothstep(1.5, 0.2, r);

    FragColor = vec4(col, 1.0);
}
