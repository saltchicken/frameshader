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
#include "Config.h"

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
        std::cerr << "ERROR: Font profile '" << config.selectedFontProfile << "' not found. Falling back to 'default'." << std::endl;
        auto default_it = config.fontProfiles.find("default");
        if (default_it != config.fontProfiles.end()) {
            selectedFont = default_it->second;
        } else {
            std::cerr << "No 'default' profile found. Exiting." << std::endl;
            return -1;
        }
    }

    // 2. Initialize Camera
    Camera camera(config.cameraDeviceID, config.cameraWidth, config.cameraHeight);
    if (!camera.isOpened()) {
        return -1;
    }

    // 3. Load the segmentation model
    cv::dnn::Net segNet;
    try {
        segNet = cv::dnn::readNet("models/selfie_segmentation.onnx");
        std::cout << "Successfully loaded ONNX person segmentation model." << std::endl;
    } catch (const cv::Exception& e) {
        std::cerr << "ERROR: Could not load the segmentation model: " << e.what() << std::endl;
        return -1;
    }

    // 4. Initialize GLFW and Window
    if (!glfwInit()) {
        return -1;
    }
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    GLFWwindow* window = glfwCreateWindow(camera.getWidth(), camera.getHeight(), "ASCII Shader", NULL, NULL);
    if (!window) {
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetKeyCallback(window, key_callback);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);

    // 5. Initialize GLAD
    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        return -1;
    }

    // 6. Build shader program
    Shader asciiShader("shaders/shader.vert", "shaders/ascii.frag");

    // 7. Setup screen quad VAO
    float vertices[] = {
        // positions         // texture coords (y-axis flipped)
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

    // 8. Setup Textures
    GLuint videoTexture;
    glGenTextures(1, &videoTexture);
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, videoTexture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    // <<< FIX: Allocate storage for the texture
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, camera.getWidth(), camera.getHeight(), 0, GL_RGB, GL_UNSIGNED_BYTE, NULL);

    GLuint fontTexture;
    loadTextureFromFile(selectedFont.path.c_str(), fontTexture, GL_TEXTURE1);

    GLuint maskTexture;
    glGenTextures(1, &maskTexture);
    glActiveTexture(GL_TEXTURE2);
    glBindTexture(GL_TEXTURE_2D, maskTexture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    // <<< FIX: Allocate storage for the texture
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RED, camera.getWidth(), camera.getHeight(), 0, GL_RED, GL_UNSIGNED_BYTE, NULL);

    // 9. Set initial shader uniforms
    asciiShader.use();
    // asciiShader.setInt("videoTexture", 0);
    asciiShader.setInt("fontAtlas", 1);
    asciiShader.setInt("maskTexture", 2);
    asciiShader.setVec2("resolution", (float)camera.getWidth(), (float)camera.getHeight());
    asciiShader.setVec2("charSize", selectedFont.charWidth, selectedFont.charHeight);
    asciiShader.setFloat("numChars", selectedFont.numChars);
    // asciiShader.setFloat("sensitivity", config.asciiSensitivity);

    // 10. Render Loop
    cv::Mat frame;
    while (!glfwWindowShouldClose(window)) {
        if (!camera.read(frame)) {
            break;
        }

        // Person Segmentation Step
        cv::Mat blob = cv::dnn::blobFromImage(frame, 1.0 / 255.0, cv::Size(256, 256), cv::Scalar(), true, false);
        segNet.setInput(blob);
        cv::Mat output = segNet.forward();
        
        // Post-process the mask
        cv::Mat mask(256, 256, CV_32F, output.ptr<float>());
        cv::resize(mask, mask, frame.size());
        
        cv::Mat mask8u;
        mask.convertTo(mask8u, CV_8U, 255);
        
        // Update video texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, videoTexture);
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, frame.cols, frame.rows, GL_BGR, GL_UNSIGNED_BYTE, frame.data);

        // Update mask texture
        glActiveTexture(GL_TEXTURE2);
        glBindTexture(GL_TEXTURE_2D, maskTexture);
        // <<< FIX: Use glTexSubImage2D here for performance
        glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, mask8u.cols, mask8u.rows, GL_RED, GL_UNSIGNED_BYTE, mask8u.data);

        // Rendering commands
        glClearColor(0.1f, 0.1f, 0.1f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);
        asciiShader.use();

        asciiShader.setFloat("time", (float)glfwGetTime());
    
        glBindVertexArray(VAO);
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // 11. Cleanup
    glDeleteVertexArrays(1, &VAO);
    glDeleteBuffers(1, &VBO);
    glDeleteBuffers(1, &EBO);
    glDeleteTextures(1, &videoTexture);
    glDeleteTextures(1, &fontTexture);
    glDeleteTextures(1, &maskTexture);
    glfwDestroyWindow(window);
    glfwTerminate();
    
    return 0;
}
