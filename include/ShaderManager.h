#pragma once

#include "Shader.h"
#include <string>
#include <memory>

/**
 * @class ShaderEffect
 * @brief The public interface for all shader effects.
 * This defines the "contract" that any new shader effect must follow.
 */
class ShaderEffect {
public:
    virtual ~ShaderEffect() = default;

    // Called once after OpenGL context is created to load assets like textures.
    virtual void loadAssets() {}

    // Called once after the shader is linked to set initial, non-changing uniforms.
    virtual void setup(Shader& shader, int frameWidth, int frameHeight) {}

    // Called every frame to update dynamic uniforms like time.
    virtual void update(Shader& shader, int frameWidth, int frameHeight) {}
};


/**
 * @class ShaderManager
 * @brief A factory class to create the appropriate shader effect based on its name.
 */
class ShaderManager {
public:
    // Creates and returns a shader effect object based on the name.
    // Returns a "noop" (no-operation) effect if the name is not found.
    static std::unique_ptr<ShaderEffect> createEffect(const std::string& effectName);
};
