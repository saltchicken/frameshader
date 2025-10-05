#pragma once

#include "Config.h"
#include "Camera.h"
#include "Renderer.h"
#include "Shader.h"
#include "ShaderManager.h"
#include <memory>

// Forward declare GLFWwindow to avoid including glfw3.h in the header
struct GLFWwindow;

class Application {
public:
    Application(int argc, char* argv[]);
    ~Application();

    // The main entry point to run the application
    void run();

private:
    void init();
    void mainLoop();
    void cleanup();

    // Static callbacks for GLFW
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);
    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);

    AppConfig m_config;
    GLFWwindow* m_window;
    std::unique_ptr<Camera> m_camera;
    std::unique_ptr<Renderer> m_renderer;
    std::unique_ptr<Shader> m_shader;
    std::unique_ptr<ShaderEffect> m_effect;
};
