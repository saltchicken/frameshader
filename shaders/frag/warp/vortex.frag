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
uniform float audio_volume;
uniform float audio_peak_freq;

uniform float vortex_speed;
uniform float vortex_arms;
uniform float vortex_spin;
uniform float vortex_center_size;
uniform float vortex_fov;
uniform float vortex_color_separation;
uniform float vortex_strobe_size;

#define PI 3.141592653589793

void main() {
    // 1. Center coordinate space and correct aspect ratio
    vec2 p = TexCoord * 2.0 - 1.0;
    p.x *= resolution.x / resolution.y;

    // 2. Polar coordinates
    float r = length(p);
    float theta = atan(p.y, p.x);

    // 3. Logarithmic Tunnel Projection
    // Bass flexes the tunnel perspective / field of view
    float fov_pump = vortex_fov + audio_bass * 0.6 * sensitivity;
    
    // Treble adds a sharp, high-frequency distortion ripple to the tunnel walls
    float treble_jitter = sin(theta * 40.0 + time * 30.0) * (audio_treble * 0.03 * sensitivity);
    
    // Apply jitter to the radius calculation to create a spiky/vibrating geometry
    float depth = (fov_pump / (r + 0.001 + treble_jitter)) + time * vortex_speed;

    // Twist the spiral: rotation accelerates on mids
    float spin = time * vortex_spin + (audio_mid * 0.8);
    float angle = (theta / (2.0 * PI)) * vortex_arms + (depth * 0.2) + spin;

    // 4. Wrap coordinates into repeated texture space (Ping-Pong mirror)
    vec2 tunnel_uv = vec2(angle, depth);
    vec2 wrapped_uv = abs(mod(tunnel_uv, 2.0) - 1.0);

    // 5. Radial RGB Chromatic Dispersion
    float sep = (vortex_color_separation + audio_bass * 0.05) * sensitivity;
    
    // Treble makes the RGB split momentarily sharper and more chaotic
    sep += (audio_treble * 0.02 * sensitivity);
    
    float cr = texture(videoTexture, abs(mod(tunnel_uv + vec2(sep, sep * 0.5), 2.0) - 1.0)).r;
    float cg = texture(videoTexture, wrapped_uv).g;
    float cb = texture(videoTexture, abs(mod(tunnel_uv - vec2(sep, sep * 0.5), 2.0) - 1.0)).b;
    vec3 col = vec3(cr, cg, cb);

    // 6. Tunnel lighting / depth attenuation (dark center, glowing rim)
    // vortex_center_size dictates the black hole size, scales smoothly outwards
    float tunnel_lighting = smoothstep(vortex_center_size, vortex_center_size + 0.4, r);
    col *= tunnel_lighting;

    // Edge flash on intense bass (Deep Blue)
    col += vec3(0.1, 0.4, 0.9) * audio_bass * (1.0 - smoothstep(0.0, 1.2, r));

    // Center strobe / laser flash on intense treble (Bright Cyan/White)
    // Uses the jittered radius so the strobe flashes in a jagged, electric shape
    float treble_strobe = smoothstep(vortex_strobe_size, 0.0, r - (treble_jitter * 2.0));
    col += vec3(0.5, 0.9, 1.0) * audio_treble * treble_strobe * sensitivity;

    FragColor = vec4(col, 1.0);
}
