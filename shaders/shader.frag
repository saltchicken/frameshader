#version 330 core
out vec4 FragColor;
in vec2 TexCoord;
uniform sampler2D videoTexture;

// Uniform variable for time, passed from C++
uniform float time;

void main() {
    // --- Wavy Effect Parameters ---
    float amplitude = 0.02; // How much the wave displaces the pixels
    float frequency = 15.0; // How many waves are on the screen
    float speed = 2.0;      // How fast the waves move

    // Calculate the horizontal offset using a sine wave
    float xOffset = amplitude * sin(TexCoord.y * frequency + time * speed);

    // Create new, distorted texture coordinates
    vec2 wavyCoords = vec2(TexCoord.x + xOffset, TexCoord.y);

    // Sample the texture using the new wavy coordinates
    FragColor = texture(videoTexture, wavyCoords);
}
