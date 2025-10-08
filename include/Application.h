#pragma once

#include <string>
#include <vector>
#include <memory>
#include <map>
#include <filesystem> // <-- Make sure this is included

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "Config.h"
#include "Camera.h"
#include "Shader.h"

// ... (FontProfile struct)

class Application {
public:
    Application(int argc, char* argv[]);
    ~Application();
    int run();

private:
    void init();
    void mainLoop();
    void cleanup();

    bool loadConfig(int argc, char* argv[]);
    bool initCamera();
    bool initWindow();
    bool initGLAD();

    void initShader();
    void initFonts();
    void initGeometry();
    void initTextures();

    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

    void updateActiveShaderUniforms();
    void reloadFontTexture();
    void handleKey(int key, int action);

    // --- ADD THIS FUNCTION DECLARATION ---
    void reloadConfiguration();
    // ------------------------------------

    const FontProfile& getCurrentFontProfile() const;

    GLFWwindow* window = nullptr;
    std::unique_ptr<Camera> camera;
    std::unique_ptr<Shader> shader;
    AppConfig config;

    GLuint VAO, VBO, EBO;
    GLuint videoTexture, fontTexture;

    std::vector<std::unique_ptr<Shader>> shaders;
    std::vector<std::string> shaderNames;
    int currentShaderIndex = 0;
    
    std::map<std::string, FontProfile> availableFonts;
    std::vector<std::string> sortedFontNames;
    int currentFontIndex = 0;

    // --- AND ADD THESE MEMBER VARIABLES ---
    std::string configFilePath;
    std::filesystem::file_time_type lastConfigWriteTime;
    // ------------------------------------
};
