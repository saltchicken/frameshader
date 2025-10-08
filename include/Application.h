#pragma once
#include "Config.h"
#include "Camera.h"
#include "Shader.h"
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <memory>
#include <vector>
#include <string>

class Application {
public:
    Application(int argc, char* argv[]);
    ~Application();
    int run();

private:
    void init();
    void mainLoop();
    void cleanup();

    // Initialization
    bool loadConfig(int argc, char* argv[]);
    bool initCamera();
    bool initWindow();
    bool initGLAD();
    void initShader();
    void initGeometry();
    void initTextures();

    // Input Handling
    void handleKey(int key, int action);

    // Callbacks
    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

    // Helpers
    void updateActiveShaderUniforms();
    void reloadFontTexture(); // <-- ADD THIS FUNCTION DECLARATION

    // Member Variables
    GLFWwindow* window = nullptr;
    std::unique_ptr<Camera> camera;
    GLuint VAO, VBO, EBO;
    GLuint videoTexture, fontTexture;

    // Shader Management
    std::vector<std::unique_ptr<Shader>> shaders;
    std::vector<std::string> fragmentShaderPaths;
    std::vector<std::string> shaderNames;
    size_t currentShaderIndex = 0;

    // Config Management
    AppConfig config;
    FontProfile selectedFont;

    // --- NEW FONT PROFILE MANAGEMENT ---
    std::vector<std::string> fontProfileNames;
    size_t currentFontProfileIndex = 0;
};
