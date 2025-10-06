#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D videoTexture; // Texture unit 0: The camera feed
uniform sampler2D fontAtlas;    // Texture unit 1: The font atlas image
uniform sampler2D maskTexture;  // Texture unit 2: The person segmentation mask

uniform vec2 resolution;
uniform vec2 charSize;
uniform float sensitivity = 1.0;
uniform float numChars = 10.0;
uniform float maskThreshold = 1.0; // Threshold for the mask (0.0 to 1.0)

void main()
{
    // Calculate the texture coordinate for the center of the character cell
    vec2 characterGrid = resolution / charSize;
    vec2 charCoord = floor(TexCoord * characterGrid);
    vec2 videoUV = (charCoord + 0.5) / characterGrid;

    // Sample the mask to decide if we are in the foreground or background
    float maskValue = texture(maskTexture, videoUV).r; // Sample the person mask

    // Sample the video color for this cell
    vec4 videoColor = texture(videoTexture, videoUV);

    // If the mask value is below our threshold, it's the background.
    // The mask is ~1.0 for the person and ~0.0 for the background.
    if (maskValue < maskThreshold) {
        // --- BACKGROUND ---
        // Apply a simple effect: desaturate the background color.
        // float grayscale = dot(videoColor.rgb, vec3(0.299, 0.587, 0.114));
        // FragColor = vec4(vec3(grayscale), 1.0);
        //
        //
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return; // Exit shader early for the background
    }

    // --- FOREGROUND (PERSON) ---
    // This code only runs if the pixel belongs to the person.
    
    // 1. Calculate the brightness (luminance) of the video color
    float brightness = dot(videoColor.rgb, vec3(0.2126, 0.7152, 0.0722));

    // 2. Boost the brightness using our sensitivity uniform
    float boostedBrightness = brightness * sensitivity;

    // 3. Select a character index, clamping the result to avoid errors
    float clampedBrightness = clamp(boostedBrightness, 0.0, 1.0);
    float charIndex = floor(clampedBrightness * (numChars - 1.0));

    // 4. Sample the font atlas based on the character index
    vec2 intraCharUV = fract(TexCoord * characterGrid);
    float atlasX = (charIndex + intraCharUV.x) / numChars;
    vec2 fontUV = vec2(atlasX, intraCharUV.y);
    vec4 fontColor = texture(fontAtlas, fontUV);
 
    // 5. Combine the font color and video color for the final output
    FragColor = fontColor * videoColor;
}
