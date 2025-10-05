#include "ShaderManager.h"
#include "TextureUtils.h" // For loading the font atlas
#include <GLFW/glfw3.h>   // For glfwGetTime()
#include <iostream>

namespace { // Use an anonymous namespace to hide these concrete classes from other files

// A default effect that does nothing. Used as a fallback.
class NullEffect : public ShaderEffect {};

// --- Concrete ShaderEffect Implementations ---

class PixelateEffect : public ShaderEffect {
public:
    void setup(Shader& shader, int frameWidth, int frameHeight) override {
        shader.use();
        shader.setVec2("resolution", (float)frameWidth, (float)frameHeight);
    }
};

class WavyEffect : public ShaderEffect {
public:
    void update(Shader& shader, int frameWidth, int frameHeight) override {
        shader.use();
        shader.setFloat("time", (float)glfwGetTime());
    }
};

class AsciiEffect : public ShaderEffect {
public:
    ~AsciiEffect() override {
        if (fontTexture != 0) {
            glDeleteTextures(1, &fontTexture);
        }
    }

    void loadAssets() override {
        loadTextureFromFile("shaders/font.png", fontTexture, GL_TEXTURE1);
    }

    void setup(Shader& shader, int frameWidth, int frameHeight) override {
        shader.use();
        // This shader needs to know about the second texture unit
        shader.setInt("fontAtlas", 1);
        shader.setVec2("resolution", (float)frameWidth, (float)frameHeight);
        shader.setVec2("charSize", 8.0f, 16.0f);
    }

private:
    GLuint fontTexture = 0;
};

} // end anonymous namespace

// --- ShaderManager Factory Implementation ---

std::unique_ptr<ShaderEffect> ShaderManager::createEffect(const std::string& effectName) {
    if (effectName == "pixelate") {
        return std::make_unique<PixelateEffect>();
    }
    if (effectName == "wavy") {
        return std::make_unique<WavyEffect>();
    }
    if (effectName == "ascii") {
        return std::make_unique<AsciiEffect>();
    }
    
    std::cerr << "Unknown shader effect name: '" << effectName << "'. Using default." << std::endl;
    // Return a default, no-op effect to prevent crashes
    return std::make_unique<NullEffect>();
}
