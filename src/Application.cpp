#include "Application.h"
#include <iostream>
#include <stdexcept>
#include <opencv2/imgcodecs.hpp>
#include <opencv2/imgproc.hpp>
#include <algorithm> // <-- Add for std::sort and std::find
#include <iterator>  // <-- Add for std::distance
#include <filesystem>

namespace {
// Helper function for loading textures, moved from main.cpp
void loadTextureFromFile(const char* path, GLuint& textureID, GLenum textureUnit) {
    glGenTextures(1, &textureID);
    glActiveTexture(textureUnit);
    glBindTexture(GL_TEXTURE_2D, textureID);
    
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    
    cv::Mat image = cv::imread(path, cv::IMREAD_UNCHANGED);
    if (image.empty()) {
        std::cerr << "Failed to load texture: " << path << std::endl;
        return;
    }
    GLenum format = GL_RGB;
    if (image.channels() == 4) {
        format = GL_RGBA;
        cv::cvtColor(image, image, cv::COLOR_BGRA2RGBA);
    } else {
        cv::cvtColor(image, image, cv::COLOR_BGR2RGB);
    }
    
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
    glTexImage2D(GL_TEXTURE_2D, 0, format, image.cols, image.rows, 0, format, GL_UNSIGNED_BYTE, image.data);
    glGenerateMipmap(GL_TEXTURE_2D);
}
} // namespace

Application::Application(int argc, char* argv[]) {
    if (!loadConfig(argc, argv)) {
        throw std::runtime_error("Failed to load configuration.");
    }
}

Application::~Application() {
    cleanup();
}

int Application::run() {
    try {
        init();
        mainLoop();
    } catch (const std::exception& e) {
        std::cerr << "An unrecoverable error occurred: " << e.what() << std::endl;
        return -1;
    }
    return 0;
}

void Application::init() {
    if (!initCamera()) throw std::runtime_error("Camera initialization failed");
    if (!initWindow()) throw std::runtime_error("Window initialization failed");
    if (!initGLAD()) throw std::runtime_error("GLAD initialization failed");
    
    initShader();
    initGeometry();
    initTextures();
    // Set initial uniforms for the first shader
    if (!shaders.empty()) {
        updateActiveShaderUniforms();
    }
}

void Application::mainLoop() {
    cv::Mat frame;
    if (!camera->read(frame)) {
        throw std::runtime_error("Could not read the first frame from the camera.");
    }
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, videoTexture);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, frame.cols, frame.rows, 0, GL_BGR, GL_UNSIGNED_BYTE, frame.data);

    while (!glfwWindowShouldClose(window)) {
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, videoTexture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, frame.cols, frame.rows, GL_BGR, GL_UNSIGNED_BYTE, frame.data);
        glClearColor(0.1f, 0.1f, 0.1f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        
        // Use the currently selected shader
        Shader* currentShader = shaders[currentShaderIndex].get();
        currentShader->use();
        currentShader->setFloat("time", (float)glfwGetTime());
        
        glBindVertexArray(VAO);
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
        
        glfwSwapBuffers(window);
        glfwPollEvents();
        
        if (!camera->read(frame)) {
            break;  
        }
    }
}

void Application::cleanup() {
    glDeleteVertexArrays(1, &VAO);
    glDeleteBuffers(1, &VBO);
    glDeleteBuffers(1, &EBO);
    glDeleteTextures(1, &videoTexture);
    glDeleteTextures(1, &fontTexture);

    if (window) {
        glfwDestroyWindow(window);
    }
    glfwTerminate();
}

bool Application::loadConfig(int argc, char* argv[]) {
    config = load_configuration(argc, argv);
    auto it = config.fontProfiles.find(config.selectedFontProfile);
    if (it != config.fontProfiles.end()) {
        selectedFont = it->second;
    } else {
        std::cerr << "ERROR: Font profile '" << config.selectedFontProfile << "' not found." << std::endl;
        auto default_it = config.fontProfiles.find("default");
        if (default_it != config.fontProfiles.end()) {
            std::cerr << "Falling back to 'default' profile." << std::endl;
            selectedFont = default_it->second;
            config.selectedFontProfile = "default"; // Update the name as well
        } else {
            std::cerr << "No 'default' profile found. Exiting." << std::endl;
            return false;
        }
    }

    // --- NEW: Populate and sort font profile names ---
    fontProfileNames.clear();
    for (const auto& pair : config.fontProfiles) {
        fontProfileNames.push_back(pair.first);
    }
    std::sort(fontProfileNames.begin(), fontProfileNames.end());

    // --- NEW: Find the index of the initially selected profile ---
    auto find_it = std::find(fontProfileNames.begin(), fontProfileNames.end(), config.selectedFontProfile);
    if (find_it != fontProfileNames.end()) {
        currentFontProfileIndex = std::distance(fontProfileNames.begin(), find_it);
    } else {
        currentFontProfileIndex = 0; // Should not happen due to fallback logic above
    }

    return true;
}

bool Application::initCamera() {
    camera = std::make_unique<Camera>(config.cameraDeviceID, config.cameraWidth, config.cameraHeight);
    return camera->isOpened();
}

bool Application::initWindow() {
    if (!glfwInit()) return false;
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    
    window = glfwCreateWindow(camera->getWidth(), camera->getHeight(), "ASCII Shader", NULL, NULL);
    if (!window) {
        glfwTerminate();
        return false;
    }
    glfwMakeContextCurrent(window);
    glfwSetWindowUserPointer(window, this); // Store 'this' to retrieve in callbacks
    glfwSetKeyCallback(window, key_callback);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);
    return true;
}

bool Application::initGLAD() {
    return gladLoadGLLoader((GLADloadproc)glfwGetProcAddress);
}

void Application::initShader() {
 std::vector<std::string> fragmentShaderPaths; // Now a local variable
    const std::string shaderDir = "shaders/frag";

    try {
        // Iterate through the directory
        for (const auto& entry : std::filesystem::directory_iterator(shaderDir)) {
            // Check if it's a regular file with the .frag extension
            if (entry.is_regular_file() && entry.path().extension() == ".frag") {
                fragmentShaderPaths.push_back(entry.path().string());
            }
        }
    } catch (const std::filesystem::filesystem_error& e) {
        std::cerr << "Filesystem error while scanning for shaders: " << e.what() << std::endl;
        throw std::runtime_error("Could not read from shader directory: " + shaderDir);
    }

    // Sort the paths alphabetically for a consistent loading order
    std::sort(fragmentShaderPaths.begin(), fragmentShaderPaths.end());

    shaders.clear(); // Clear any previous shaders
    shaderNames.clear();
    for (const auto& path : fragmentShaderPaths) {
        try {
            shaders.push_back(std::make_unique<Shader>("shaders/vert/shader.vert", path.c_str()));
            std::cout << "Loaded shader: " << path << std::endl;

            std::string pathStr = path;
            size_t last_slash = pathStr.find_last_of("/\\");
            last_slash = (last_slash == std::string::npos) ? 0 : last_slash + 1;
            size_t last_dot = pathStr.find_last_of('.');
            std::string shortName = pathStr.substr(last_slash, last_dot - last_slash);
            shaderNames.push_back(shortName);

        } catch (const std::exception& e) {
            std::cerr << "Failed to load shader " << path << ": " << e.what() << std::endl;
        }
    }

    if (shaders.empty()) {
        throw std::runtime_error("No shaders could be loaded. Exiting.");
    }
}

void Application::initGeometry() {
    float vertices[] = {
        // positions          // texture coords (y-axis flipped)
         1.0f,  1.0f, 0.0f,   1.0f, 0.0f,
         1.0f, -1.0f, 0.0f,   1.0f, 1.0f,
        -1.0f, -1.0f, 0.0f,   0.0f, 1.0f,
        -1.0f,  1.0f, 0.0f,   0.0f, 0.0f
    };
    unsigned int indices[] = { 0, 1, 3, 1, 2, 3 };

    glGenVertexArrays(1, &VAO);
    glGenBuffers(1, &VBO);
    glGenBuffers(1, &EBO);
    glBindVertexArray(VAO);
    glBindBuffer(GL_ARRAY_BUFFER, VBO);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, EBO);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices, GL_STATIC_DRAW);
    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 5 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 5 * sizeof(float), (void*)(3 * sizeof(float)));
    glEnableVertexAttribArray(1);
}

void Application::initTextures() {
    glGenTextures(1, &videoTexture);
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, videoTexture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    loadTextureFromFile(selectedFont.path.c_str(), fontTexture, GL_TEXTURE1);
}

void Application::framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}

void Application::key_callback(GLFWwindow* window, int key, int scancode, int action, int mods) {
    Application* app = static_cast<Application*>(glfwGetWindowUserPointer(window));
    if (app) {
        app->handleKey(key, action);
    }
}

void Application::updateActiveShaderUniforms() {
    if (shaders.empty() || shaderNames.empty()) return;

    Shader* currentShader = shaders[currentShaderIndex].get();
    const std::string& currentShaderName = shaderNames[currentShaderIndex];

    currentShader->use();
    // Set common uniforms for all shaders
    currentShader->setInt("videoTexture", 0);
    currentShader->setInt("fontAtlas", 1);
    currentShader->setVec2("resolution", (float)camera->getWidth(), (float)camera->getHeight());
    currentShader->setVec2("charSize", selectedFont.charWidth, selectedFont.charHeight);
    currentShader->setFloat("numChars", selectedFont.numChars);

    // Find the specific configuration for the current shader
    auto it = config.shaderConfigs.find(currentShaderName);
    if (it != config.shaderConfigs.end()) {
        const ShaderConfig& shaderConf = it->second;
        // NEW: Loop through all parameters in the config map and set them as uniforms
        for (const auto& pair : shaderConf) {
            const std::string& uniformName = pair.first;
            float uniformValue = pair.second;
            currentShader->setFloat(uniformName, uniformValue);
        }
    }
}

// --- NEW FUNCTION IMPLEMENTATION ---
void Application::reloadFontTexture() {
    if (fontProfileNames.empty()) return;

    // Get the name of the new profile
    const std::string& profileName = fontProfileNames[currentFontProfileIndex];

    // Find the profile data in the config map
    auto it = config.fontProfiles.find(profileName);
    if (it != config.fontProfiles.end()) {
        selectedFont = it->second; // Update the selectedFont struct

        std::cout << "Switched to font profile: " << profileName << std::endl;

        // Delete the old texture to prevent memory leaks
        glDeleteTextures(1, &fontTexture);

        // Load the new texture
        loadTextureFromFile(selectedFont.path.c_str(), fontTexture, GL_TEXTURE1);

        // After changing the font, we must update the shader uniforms
        updateActiveShaderUniforms();
    } else {
        std::cerr << "Could not find font profile: " << profileName << std::endl;
    }
}

void Application::handleKey(int key, int action) {
    if (action == GLFW_PRESS) {
        if (key == GLFW_KEY_ESCAPE) {
            glfwSetWindowShouldClose(window, true);
        }
        
        if (key == GLFW_KEY_RIGHT) {
            // Cycle to the next shader, wrapping around
            currentShaderIndex = (currentShaderIndex + 1) % shaders.size();
            updateActiveShaderUniforms();
            std::cout << "Switched to shader: " << shaderNames[currentShaderIndex] << std::endl;
        }
        if (key == GLFW_KEY_LEFT) {
            // Cycle to the previous shader, wrapping around
            currentShaderIndex = (currentShaderIndex + shaders.size() - 1) % shaders.size();
            updateActiveShaderUniforms();
            std::cout << "Switched to shader: " << shaderNames[currentShaderIndex] << std::endl;
        }

        // --- NEW KEY HANDLERS FOR FONT TOGGLING ---
        if (key == GLFW_KEY_UP) {
            if (!fontProfileNames.empty()) {
                currentFontProfileIndex = (currentFontProfileIndex + fontProfileNames.size() - 1) % fontProfileNames.size();
                reloadFontTexture();
            }
        }
        if (key == GLFW_KEY_DOWN) {
            if (!fontProfileNames.empty()) {
                currentFontProfileIndex = (currentFontProfileIndex + 1) % fontProfileNames.size();
                reloadFontTexture();
            }
        }
    }
}
