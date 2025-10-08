#include "Application.h"
#include <iostream>
#include <stdexcept>
#include <opencv2/imgcodecs.hpp>
#include <opencv2/imgproc.hpp>
#include <algorithm>
#include <iterator>
#include <filesystem>

namespace {
// Helper function for loading textures
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
    initFonts();
    initGeometry();
    initTextures();

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
        if (!configFilePath.empty() && std::filesystem::exists(configFilePath)) {
            auto currentWriteTime = std::filesystem::last_write_time(configFilePath);
            if (currentWriteTime > lastConfigWriteTime) {
                reloadConfiguration();
                lastConfigWriteTime = currentWriteTime;
            }
        }
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, videoTexture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, frame.cols, frame.rows, GL_BGR, GL_UNSIGNED_BYTE, frame.data);
        glClearColor(0.1f, 0.1f, 0.1f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        
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

    const char* homeDir = getenv("HOME");
    if (homeDir) {
        configFilePath = std::string(homeDir) + "/.config/frame_shader/config.ini";
        if (std::filesystem::exists(configFilePath)) {
            lastConfigWriteTime = std::filesystem::last_write_time(configFilePath);
        }
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
    glfwSetWindowUserPointer(window, this);
    glfwSetKeyCallback(window, key_callback);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);
    return true;
}

bool Application::initGLAD() {
    return gladLoadGLLoader((GLADloadproc)glfwGetProcAddress);
}

void Application::initShader() {
    std::vector<std::string> fragmentShaderPaths;
    const std::string shaderDir = "shaders/frag";

    try {
        for (const auto& entry : std::filesystem::directory_iterator(shaderDir)) {
            if (entry.is_regular_file() && entry.path().extension() == ".frag") {
                fragmentShaderPaths.push_back(entry.path().string());
            }
        }
    } catch (const std::filesystem::filesystem_error& e) {
        throw std::runtime_error("Could not read from shader directory: " + shaderDir);
    }

    std::sort(fragmentShaderPaths.begin(), fragmentShaderPaths.end());

    shaders.clear();
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

void Application::initFonts() {
    const std::string fontDir = "font_atlases";
    availableFonts.clear(); // Clear any pre-existing font data

    try {
        for (const auto& entry : std::filesystem::directory_iterator(fontDir)) {
            if (!entry.is_regular_file() || entry.path().extension() != ".png") {
                continue;
            }

            std::string profileName = entry.path().stem().string();
            FontProfile profile; // Create a new profile with default-constructed values
            profile.path = entry.path().string();

            // 1. Attempt to parse default values from the filename
            // TODO: Make this a function for readability
            std::string stem = profileName;
            size_t last_dash = stem.find_last_of('-');
            if (last_dash != std::string::npos) {
                std::string dimensions_part = stem.substr(last_dash + 1);
                stem = stem.substr(0, last_dash);

                size_t x_pos = dimensions_part.find('x');
                if (x_pos != std::string::npos && x_pos > 0 && x_pos < dimensions_part.length() - 1) {
                    try {
                        profile.charWidth = std::stof(dimensions_part.substr(0, x_pos));
                        profile.charHeight = std::stof(dimensions_part.substr(x_pos + 1));

                        size_t second_last_dash = stem.find_last_of('-');
                        if (second_last_dash != std::string::npos) {
                            std::string numchars_part = stem.substr(second_last_dash + 1);
                            profile.numChars = std::stof(numchars_part);
                        }
                    } catch (const std::exception& e) {
                        std::cerr << "Warning: Could not parse metrics from filename '" << profileName << "'. Using defaults or config." << std::endl;
                    }
                }
            }

            // 2. Check for overrides from the configuration file
            auto it = config.fontConfigs.find(profileName);
            if (it != config.fontConfigs.end()) {
                const FontConfig& fontConf = it->second;
                if (fontConf.count("char_width")) profile.charWidth = fontConf.at("char_width");
                if (fontConf.count("char_height")) profile.charHeight = fontConf.at("char_height");
                if (fontConf.count("num_chars")) profile.numChars = fontConf.at("num_chars");
            }
            
            // 3. Add the fully configured profile to the map
            availableFonts[profileName] = profile;
        }
    } catch (const std::filesystem::filesystem_error& e) {
        throw std::runtime_error("Could not read from font atlas directory: " + fontDir);
    }

    // This part remains the same: sort names and find the selected font
    sortedFontNames.clear();
    for (const auto& pair : availableFonts) {
        sortedFontNames.push_back(pair.first);
    }
    std::sort(sortedFontNames.begin(), sortedFontNames.end());

    auto find_it = std::find(sortedFontNames.begin(), sortedFontNames.end(), config.selectedFontProfile);
    if (find_it != sortedFontNames.end()) {
        currentFontIndex = std::distance(sortedFontNames.begin(), find_it);
    } else if (!sortedFontNames.empty()){
        std::cerr << "Warning: Selected font '" << config.selectedFontProfile << "' not found. Falling back to first available font." << std::endl;
        currentFontIndex = 0;
    } else {
        throw std::runtime_error("No font atlases found in 'font_atlases/' directory.");
    }
}

void Application::initGeometry() {
    float vertices[] = {
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

    const FontProfile& currentFont = getCurrentFontProfile();
    loadTextureFromFile(currentFont.path.c_str(), fontTexture, GL_TEXTURE1);
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
    const FontProfile& currentFont = getCurrentFontProfile();

    currentShader->use();
    currentShader->setInt("videoTexture", 0);
    currentShader->setInt("fontAtlas", 1);
    currentShader->setVec2("resolution", (float)camera->getWidth(), (float)camera->getHeight());
    currentShader->setVec2("charSize", currentFont.charWidth, currentFont.charHeight);
    currentShader->setFloat("numChars", currentFont.numChars);

    auto it = config.shaderConfigs.find(currentShaderName);
    if (it != config.shaderConfigs.end()) {
        const ShaderConfig& shaderConf = it->second;
        for (const auto& pair : shaderConf) {
            currentShader->setFloat(pair.first, pair.second);
        }
    }
}

void Application::reloadFontTexture() {
    if (sortedFontNames.empty()) return;

    const FontProfile& newFont = getCurrentFontProfile();
    std::cout << "Switched to font profile: " << sortedFontNames[currentFontIndex] << std::endl;

    glDeleteTextures(1, &fontTexture);
    loadTextureFromFile(newFont.path.c_str(), fontTexture, GL_TEXTURE1);
    updateActiveShaderUniforms();
}

void Application::handleKey(int key, int action) {
    if (action == GLFW_PRESS) {
        if (key == GLFW_KEY_ESCAPE) {
            glfwSetWindowShouldClose(window, true);
        }

        if (key == GLFW_KEY_R) {
            reloadConfiguration();
        }
        
        if (key == GLFW_KEY_RIGHT) {
            currentShaderIndex = (currentShaderIndex + 1) % shaders.size();
            updateActiveShaderUniforms();
            std::cout << "Switched to shader: " << shaderNames[currentShaderIndex] << std::endl;
        }
        if (key == GLFW_KEY_LEFT) {
            currentShaderIndex = (currentShaderIndex + shaders.size() - 1) % shaders.size();
            updateActiveShaderUniforms();
            std::cout << "Switched to shader: " << shaderNames[currentShaderIndex] << std::endl;
        }
        
        if (key == GLFW_KEY_UP) {
            if (!sortedFontNames.empty()) {
                currentFontIndex = (currentFontIndex + sortedFontNames.size() - 1) % sortedFontNames.size();
                reloadFontTexture();
            }
        }
        if (key == GLFW_KEY_DOWN) {
            if (!sortedFontNames.empty()) {
                currentFontIndex = (currentFontIndex + 1) % sortedFontNames.size();
                reloadFontTexture();
            }
        }
    }
}

const FontProfile& Application::getCurrentFontProfile() const {
    const std::string& currentFontName = sortedFontNames[currentFontIndex];
    return availableFonts.at(currentFontName);
}

void Application::reloadConfiguration() {
    std::cout << "Configuration file changed. Reloading settings..." << std::endl;
    
    // Re-parse the ini file over our existing config struct
    load_from_ini(config);
    
    // Re-apply the new settings to the currently active shader
    updateActiveShaderUniforms();
}
