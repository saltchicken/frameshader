#version 460 core

out vec4 FragColor;
in vec2 TexCoord;

// We only need the maskTexture for this shader
uniform sampler2D maskTexture;

void main() {
    // 1. Sample the single red channel from the mask texture.
    // The value will be between 0.0 (background) and 1.0 (foreground).
    float maskValue = texture(maskTexture, TexCoord).r;

    // 2. Output this value as a grayscale color.
    // We put the same value in the R, G, and B components.
    FragColor = vec4(vec3(maskValue), 1.0);
}
