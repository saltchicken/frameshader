#include <iostream>
#include <string>
#include <vector>
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <opencv2/core.hpp>
#include <opencv2/imgcodecs.hpp>
#include <opencv2/imgproc.hpp>
#include <opencv2/dnn.hpp>
#include "Shader.h"
#include "Camera.h"
#include "Config.h" // Use the configuration system

// --- Helper function for loading textures ---
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

// --- GLFW Callbacks ---
void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}

void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods) {
    if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
        glfwSetWindowShouldClose(window, true);
    }
}

int main(int argc, char* argv[]) {
    // 1. Load Configuration
    AppConfig config = load_configuration(argc, argv);

    FontProfile selectedFont;
    auto it = config.fontProfiles.find(config.selectedFontProfile);
    if (it != config.fontProfiles.end()) {
        selectedFont = it->second;
    } else {
        std::cerr << "ERROR: Font profile '" << config.selectedFontProfile << "' not found in config.ini." << std::endl;
        // Attempt to fall back to a "default" profile if it exists
        auto default_it = config.fontProfiles.find("default");
        if (default_it != config.fontProfiles.end()) {
            std::cerr << "Falling back to 'default' profile." << std::endl;
            selectedFont = default_it->second;
        } else {
            std::cerr << "No 'default' profile found. Exiting." << std::endl;
            return -1;
        }
    }

    // 2. Initialize Camera
    Camera camera(config.cameraDeviceID, config.cameraWidth, config.cameraHeight);
    if (!camera.isOpened()) {
        std::cerr << "ERROR: Could not open camera. Check device ID." << std::endl;
        return -1;
    }

    // 3. Initialize GLFW and Window
    if (!glfwInit()) {
        std::cerr << "Failed to initialize GLFW" << std::endl;
        return -1;
    }
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    GLFWwindow* window = glfwCreateWindow(camera.getWidth(), camera.getHeight(), "ASCII Shader", NULL, NULL);
    if (!window) {
        std::cerr << "Failed to create GLFW window" << std::endl;
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetKeyCallback(window, key_callback);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);

    // 4. Initialize GLAD
    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        std::cerr << "Failed to initialize GLAD" << std::endl;
        return -1;
    }

    cv::dnn::Net depthNet;
    try {
        depthNet = cv::dnn::readNet("models/model-small.onnx");
        std::cout << "Successfully loaded depth estimation model." << std::endl;
        depthNet.setPreferableBackend(cv::dnn::DNN_BACKEND_OPENCV);
        depthNet.setPreferableTarget(cv::dnn::DNN_TARGET_CPU);
    } catch (const cv::Exception& e) {
        std::cerr << "ERROR: Could not load the depth model: " << e.what() << std::endl;
        return -1;
    }

    // 5. Build shader program
    Shader asciiShader("shaders/shader.vert", "shaders/ascii.frag");

    // 6. Setup screen quad VAO
    // ## FIX for upside-down video: Flipped texture coordinates ##
    float vertices[] = {
        // positions         // texture coords (note y-axis is flipped)
         1.0f,  1.0f, 0.0f,   1.0f, 0.0f, // top right
         1.0f, -1.0f, 0.0f,   1.0f, 1.0f, // bottom right
        -1.0f, -1.0f, 0.0f,   0.0f, 1.0f, // bottom left
        -1.0f,  1.0f, 0.0f,   0.0f, 0.0f  // top left
    };
    unsigned int indices[] = { 0, 1, 3, 1, 2, 3 };
    GLuint VAO, VBO, EBO;
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

    // 7. Setup Textures
    GLuint videoTexture;
    glGenTextures(1, &videoTexture);
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, videoTexture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    GLuint fontTexture;
    loadTextureFromFile(selectedFont.path.c_str(), fontTexture, GL_TEXTURE1);

    GLuint depthTexture;
    glGenTextures(1, &depthTexture);
    glActiveTexture(GL_TEXTURE2);
    glBindTexture(GL_TEXTURE_2D, depthTexture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    // 8. Set initial shader uniforms
    asciiShader.use();
    asciiShader.setInt("videoTexture", 0);
    asciiShader.setInt("fontAtlas", 1);
    asciiShader.setVec2("resolution", (float)camera.getWidth(), (float)camera.getHeight());
    asciiShader.setVec2("charSize", selectedFont.charWidth, selectedFont.charHeight);
    asciiShader.setFloat("numChars", selectedFont.numChars);

    asciiShader.setFloat("sensitivity", config.asciiSensitivity);

    // 9. Render Loop
    cv::Mat frame, depthMap;
    if (!camera.read(frame)) {
         std::cerr << "Could not read the first frame from the camera." << std::endl;
         return -1;
    }
    
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, videoTexture);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, frame.cols, frame.rows, 0, GL_BGR, GL_UNSIGNED_BYTE, frame.data);

    while (!glfwWindowShouldClose(window)) {
        if (!camera.read(frame)) break;

        // --- NEW: Depth Estimation Step ---
        cv::Mat blob;
        cv::dnn::blobFromImage(frame, blob, 1/255.f, cv::Size(256, 256), cv::Scalar(123.675, 116.28, 103.53), true, false);
        depthNet.setInput(blob);
        cv::Mat depthOutput = depthNet.forward();

        cv::Mat depthResult = depthOutput.reshape(1, 256);

        // Normalize the depth map to be in the 0-1 range for visualization
        cv::normalize(depthResult, depthMap, 1, 0, cv::NORM_MINMAX);
        
        // --- Update Textures ---
        // Update video texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, videoTexture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, frame.cols, frame.rows, GL_BGR, GL_UNSIGNED_BYTE, frame.data);

        // Update depth texture (convert to a displayable format)
        cv::Mat depthVis;
        depthMap.convertTo(depthVis, CV_8UC1, 255.0); // Convert float map to 8-bit grayscale
        glActiveTexture(GL_TEXTURE2);
        glBindTexture(GL_TEXTURE_2D, depthTexture);
        // We need to re-initialize if the size changes, but for now we'll update it
        // Note: The depth map is low-res (256x256), but the GPU will interpolate it.
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RED, depthVis.cols, depthVis.rows, 0, GL_RED, GL_UNSIGNED_BYTE, depthVis.data);

        // --- Rendering commands ---
        // ... (glClearColor, glClear, etc.)
        asciiShader.use();
        glBindVertexArray(VAO);
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // 10. Cleanup
    glDeleteVertexArrays(1, &VAO);
    glDeleteBuffers(1, &VBO);
    glDeleteBuffers(1, &EBO);
    glDeleteTextures(1, &videoTexture);
    glDeleteTextures(1, &fontTexture);
    glDeleteTextures(1, &depthTexture);
    glfwDestroyWindow(window);
    glfwTerminate();
    
    return 0;
}
