#pragma once

#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <memory>
#include "Config.h"
#include "Camera.h"
#include "Shader.h"

class Application {
public:
    Application(int argc, char* argv[]);
    ~Application();
    
    // Disallow copying and assignment
    Application(const Application&) = delete;
    Application& operator=(const Application&) = delete;

    int run();

private:
    void init();
    void mainLoop();
    void cleanup();

    // Initialization helpers
    bool loadConfig(int argc, char* argv[]);
    bool initCamera();
    bool initWindow();
    bool initGLAD();
    void initShader();
    void initGeometry();
    void initTextures();

    // GLFW static callbacks that delegate to member functions
    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);
    
    // Member function to handle input
    void handleKey(int key, int action);

    // Member Variables
    AppConfig config;
    FontProfile selectedFont;
    GLFWwindow* window = nullptr;
    std::unique_ptr<Camera> camera;
    std::unique_ptr<Shader> asciiShader;
    
    GLuint VAO = 0, VBO = 0, EBO = 0;
    GLuint videoTexture = 0, fontTexture = 0;
};
