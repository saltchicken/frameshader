#pragma once

#include "Shader.h"
#include "Camera.h"
#include "Config.h"
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <memory>
#include <vector> // <-- Add this include

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
    void initGeometry();
    void initTextures();

    // Shader management
    void updateActiveShaderUniforms(); // <-- Add new helper function
    std::vector<std::unique_ptr<Shader>> shaders; // <-- Store multiple shaders
    std::vector<std::string> fragmentShaderPaths; // <-- Store paths for feedback
    std::vector<std::string> shaderNames;
    size_t currentShaderIndex = 0; // <-- Track the current shader

    // Input handling
    void handleKey(int key, int action);
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);
    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);

    // Member variables
    GLFWwindow* window = nullptr;
    AppConfig config;
    FontProfile selectedFont;
    std::unique_ptr<Camera> camera;
    
    // OpenGL handles
    GLuint VAO = 0, VBO = 0, EBO = 0;
    GLuint videoTexture = 0, fontTexture = 0;
};
