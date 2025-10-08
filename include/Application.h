#pragma once

#include <string>
#include <vector>
#include <memory>
#include <map>
#include <filesystem>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "Config.h"
#include "Camera.h"
#include "Shader.h"
#include "SegmentationModel.h"

class Application {
public:
    Application(int argc, char* argv[]);
    ~Application();
    int run();

private:
    // Private methods
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
    void handleKey(int key, int action);
    void updateActiveShaderUniforms();
    void reloadConfiguration();
    void reloadFontTexture();
    const FontProfile& getCurrentFontProfile() const;

    static void framebuffer_size_callback(GLFWwindow* window, int width, int height);
    static void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

    // --- Member Variables ---
    GLFWwindow* window = nullptr;
    AppConfig config;
    std::string configFilePath;
    std::filesystem::file_time_type lastConfigWriteTime;

    GLuint VAO = 0, VBO = 0, EBO = 0;
    GLuint videoTexture = 0;
    GLuint fontTexture = 0;
    GLuint maskTexture = 0;

    std::unique_ptr<Camera> camera;
    std::vector<std::unique_ptr<Shader>> shaders;
    std::vector<std::string> shaderNames;
    int currentShaderIndex = 0;

    std::unique_ptr<fs::SegmentationModel> segmentationModel;

    // Font-related members
    std::map<std::string, FontProfile> availableFonts;
    std::vector<std::string> sortedFontNames;
    
    // --- THIS IS THE FIX ---
    // The duplicate 'currentShaderIndex' has been corrected to 'currentFontIndex'.
    int currentFontIndex = 0; 
};
