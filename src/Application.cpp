#include "Application.h"
#include <iostream>
#include <stdexcept>
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <opencv2/core/mat.hpp>

Application::Application(int argc, char* argv[]) : m_window(nullptr) {
    m_config = load_configuration(argc, argv);
}

Application::~Application() {
    cleanup();
}

void Application::run() {
    try {
        init();
        mainLoop();
    } catch (const std::exception& e) {
        std::cerr << "An error occurred: " << e.what() << std::endl;
        cleanup();
    }
}

void Application::init() {
    // 1. Initialize Camera
    m_camera = std::make_unique<Camera>(m_config.cameraDeviceID, m_config.cameraWidth, m_config.cameraHeight);
    if (!m_camera->isOpened()) {
        throw std::runtime_error("Failed to open camera.");
    }
    const int FRAME_WIDTH = m_camera->getWidth();
    const int FRAME_HEIGHT = m_camera->getHeight();

    // 2. Initialize GLFW and create window
    if (!glfwInit()) {
        throw std::runtime_error("Failed to initialize GLFW.");
    }
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    m_window = glfwCreateWindow(FRAME_WIDTH, FRAME_HEIGHT, "FrameShader", NULL, NULL);
    if (!m_window) {
        throw std::runtime_error("Failed to create GLFW window.");
    }
    glfwMakeContextCurrent(m_window);
    glfwSetWindowUserPointer(m_window, this); // Store a pointer to this instance
    glfwSetFramebufferSizeCallback(m_window, framebuffer_size_callback);
    glfwSetKeyCallback(m_window, key_callback);

    // 3. Initialize GLAD
    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        throw std::runtime_error("Failed to initialize GLAD.");
    }

    // 4. Initialize Renderer
    m_renderer = std::make_unique<Renderer>();
    cv::Mat firstFrame;
    if (!m_camera->read(firstFrame)) {
        throw std::runtime_error("Failed to read initial camera frame.");
    }
    m_renderer->initializeTexture(firstFrame);

    // 5. Build and compile our shader program
    std::string fragmentShaderPath = "shaders/" + m_config.fragmentShaderName + ".frag";
    m_shader = std::make_unique<Shader>("shaders/shader.vert", fragmentShaderPath.c_str());
    m_effect = ShaderManager::createEffect(m_config.fragmentShaderName);
    m_effect->loadAssets();

    // 6. Final shader setup
    m_shader->use();
    m_shader->setInt("videoTexture", 0);
    m_effect->setup(*m_shader, FRAME_WIDTH, FRAME_HEIGHT);
}

void Application::mainLoop() {
    cv::Mat frame;
    while (!glfwWindowShouldClose(m_window)) {
        // Read the next frame
        if (!m_camera->read(frame)) {
            break; // End of video stream or camera error
        }
        
        // Update the video texture with the current frame
        m_renderer->updateTexture(frame);

        // Rendering commands
        glClearColor(0.2f, 0.3f, 0.3f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        // Use shader and update uniforms via the effect
        m_shader->use();
        m_effect->update(*m_shader, m_camera->getWidth(), m_camera->getHeight());
        
        // Draw the quad
        m_renderer->draw();

        // Swap buffers and poll for events
        glfwSwapBuffers(m_window);
        glfwPollEvents();
    }
}

void Application::cleanup() {
    // The unique_ptrs will handle deleting the objects.
    // We only need to terminate GLFW.
    if (m_window) {
        glfwDestroyWindow(m_window);
    }
    glfwTerminate();
}

// --- Static Callbacks ---
void Application::framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}

void Application::key_callback(GLFWwindow* window, int key, int scancode, int action, int mods) {
    if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
        glfwSetWindowShouldClose(window, true);
    }
    // You can add more key logic here, retrieving the Application instance:
    // Application* app = static_cast<Application*>(glfwGetWindowUserPointer(window));
    // app->handle_key_event(...)
}
