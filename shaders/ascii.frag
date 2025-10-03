#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D videoTexture; // Texture unit 0: The camera feed
uniform sampler2D fontAtlas;    // Texture unit 1: The font atlas image

uniform vec2 resolution;        // Resolution of the camera feed (e.g., 1920x1080)
uniform vec2 charSize;          // Size of one character cell (e.g., 8x16 pixels)

// The characters in our font atlas, ordered by brightness.
// Space, ., :, -, =, +, *, #, %, @
const float numChars = 10.0;

void main()
{
    // 1. Calculate the dimensions of the character grid
    vec2 characterGrid = resolution / charSize;

    // 2. Find the coordinate of the character cell this fragment belongs to
    vec2 charCoord = floor(TexCoord * characterGrid);

    // 3. Calculate the UV coordinate for the center of that cell to sample the video texture
    vec2 videoUV = (charCoord + 0.5) / characterGrid;
    vec4 videoColor = texture(videoTexture, videoUV);

    // 4. Calculate the brightness (luminance) of the video color
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));

    // 5. Select a character index from the atlas based on brightness
    float charIndex = floor(brightness * (numChars - 1.0));

    // 6. Find the UV coordinate *within* the character cell (from 0.0 to 1.0)
    vec2 intraCharUV = fract(TexCoord * characterGrid);

    // 7. Calculate the final UV to sample from the font atlas
    // The X coordinate is based on the character index and the intra-cell X
    // The Y coordinate is just the intra-cell Y
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);

    // 8. Sample the font atlas to get the character shape (black or white)
    vec4 fontColor = texture(fontAtlas, fontUV);

    // 9. Final color is the character shape tinted by the original video color
    FragColor = fontColor * videoColor;
}
