#pragma once
#include "Config.h"
#include "Camera.h"
#include "Shader.h"
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <memory>
#include <vector>
#include <string>
#include <map>

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
    void initFonts(); // <-- NEW
    void initGeometry();
    void initTextures();

    // Input Handling
    void handleKey(int key, int action);

    // Callbacks
    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

    // Helpers
    void updateActiveShaderUniforms();
    void reloadFontTexture();
    const FontProfile& getCurrentFontProfile() const; // <-- NEW HELPER

    // Member Variables
    GLFWwindow* window = nullptr;
    std::unique_ptr<Camera> camera;
    GLuint VAO, VBO, EBO;
    GLuint videoTexture, fontTexture;

    // Shader Management
    std::vector<std::unique_ptr<Shader>> shaders;
    std::vector<std::string> shaderNames;
    size_t currentShaderIndex = 0;

    // Config Management
    AppConfig config;

    // --- NEW: Font Profile Management ---
    std::map<std::string, FontProfile> availableFonts;
    std::vector<std::string> sortedFontNames;
    size_t currentFontIndex = 0;
};
