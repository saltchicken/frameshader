#version 460 core
out vec4 FragColor;

// This line is changed
in vec2 TexCoord; // Was 'texCoord'

uniform sampler2D videoTexture;
uniform vec2 resolution; // The width and height of the camera feed

void main()
{
    // Define the size of the pixelation blocks (e.g., 8x8 pixels)
    float pixelSize = 32.0;

    // 1. Convert normalized texture coords (0-1) to pixel coords (e.g., 0-1920)
    vec2 pixelCoords = TexCoord * resolution;

    // 2. Snap these coordinates to the top-left of the 8x8 grid cell
    vec2 snappedCoords = floor(pixelCoords / pixelSize) * pixelSize;

    // 3. Convert the snapped pixel coords back to normalized texture coords
    vec2 pixelatedTexCoord = snappedCoords / resolution;

    // All pixels within the same 8x8 cell will now sample from the same coordinate
    FragColor = texture(videoTexture, pixelatedTexCoord);
}
