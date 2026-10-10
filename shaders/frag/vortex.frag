#version 460 core
out vec4 FragColor;
in vec2 TexCoord;

uniform sampler2D videoTexture;
uniform vec2 resolution;
uniform float time;
uniform float sensitivity;

uniform float audio_bass;
uniform float audio_mid;

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
    float fov_pump = 0.5 + audio_bass * 0.6 * sensitivity;
    float depth = (fov_pump / (r + 0.001)) + time * 1.5;

    // Twist the spiral: rotation accelerates on mids
    float twist_rate = 3.0 / PI;
    float spin = time * 0.4 + (audio_mid * 0.8);
    float angle = (theta / (2.0 * PI)) * 4.0 + (depth * 0.2) + spin;

    // 4. Wrap coordinates into repeated texture space (Ping-Pong mirror)
    vec2 tunnel_uv = vec2(angle, depth);
    vec2 wrapped_uv = abs(mod(tunnel_uv, 2.0) - 1.0);

    // 5. Radial RGB Chromatic Dispersion
    float sep = (0.015 + audio_bass * 0.05) * sensitivity;
    float cr = texture(videoTexture, abs(mod(tunnel_uv + vec2(sep, sep * 0.5), 2.0) - 1.0)).r;
    float cg = texture(videoTexture, wrapped_uv).g;
    float cb = texture(videoTexture, abs(mod(tunnel_uv - vec2(sep, sep * 0.5), 2.0) - 1.0)).b;
    vec3 col = vec3(cr, cg, cb);

    // 6. Tunnel lighting / depth attenuation (dark center, glowing rim)
    float tunnel_lighting = smoothstep(0.05, 0.45, r);
    col *= tunnel_lighting;

    // Edge flash on intense bass
    col += vec3(0.1, 0.4, 0.9) * audio_bass * (1.0 - smoothstep(0.0, 1.2, r));

    FragColor = vec4(col, 1.0);
}
