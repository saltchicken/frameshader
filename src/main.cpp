#include <iostream>
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include "Shader.h"
#include "Camera.h"
#include <opencv2/imgcodecs.hpp>

// GLFW Callbacks
void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    glViewport(0, 0, width, height);
}

void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods) {
    if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
        glfwSetWindowShouldClose(window, true);
    }
}

// Helper function to load an image into an OpenGL texture
void loadTexture(const char* path, GLuint& textureID, GLenum textureUnit) {
    glGenTextures(1, &textureID);
    glActiveTexture(textureUnit);
    glBindTexture(GL_TEXTURE_2D, textureID);

    // Set texture wrapping/filtering options
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    // Load image using OpenCV
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

    glTexImage2D(GL_TEXTURE_2D, 0, format, image.cols, image.rows, 0, format, GL_UNSIGNED_BYTE, image.data);
    glGenerateMipmap(GL_TEXTURE_2D);
}

int main() {
    // 1. Initialize Camera
    Camera camera(0, 1920, 1080);
    if (!camera.isOpened()) {
        return -1;
    }
    const int FRAME_WIDTH = camera.getWidth();
    const int FRAME_HEIGHT = camera.getHeight();

    // 2. Initialize GLFW and create window
    glfwInit();
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    GLFWwindow* window = glfwCreateWindow(FRAME_WIDTH, FRAME_HEIGHT, "FrameShader", NULL, NULL);
    if (window == NULL) {
        std::cerr << "Failed to create GLFW window" << std::endl;
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);
    glfwSetKeyCallback(window, key_callback);

    // 3. Initialize GLAD
    if (!gladLoadGLLoader((GLADloadproc)glfwGetProcAddress)) {
        std::cerr << "Failed to initialize GLAD" << std::endl;
        return -1;
    }

    // 4. Build and compile our shader program
    Shader ourShader("shaders/shader.vert", "shaders/ascii.frag");

    // 5. Vertex data for a screen-filling quad
    float vertices[] = {
        // positions          // texture coords (V flipped)
         1.0f,  1.0f, 0.0f,   1.0f, 0.0f, // top right
         1.0f, -1.0f, 0.0f,   1.0f, 1.0f, // bottom right
        -1.0f, -1.0f, 0.0f,   0.0f, 1.0f, // bottom left
        -1.0f,  1.0f, 0.0f,   0.0f, 0.0f  // top left 
    };
    unsigned int indices[] = { 0, 1, 3, 1, 2, 3 };
    GLuint VBO, VAO, EBO;
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

    // 6. Create texture for the video frame and font texture
    GLuint videoTexture;
    glGenTextures(1, &videoTexture);
    glActiveTexture(GL_TEXTURE0); // Activate texture unit 0
    glBindTexture(GL_TEXTURE_2D, videoTexture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    GLuint fontTexture;
    loadTexture("shaders/font.png", fontTexture, GL_TEXTURE1); // Load font into texture unit 1

    // 7. The Render Loop
    cv::Mat frame;
    ourShader.use(); // Activate shader once before the loop
    ourShader.setInt("videoTexture", 0); // Tell shader videoTexture is on unit 0
    ourShader.setInt("fontAtlas", 1);    // Tell shader fontAtlas is on unit
                                         //
    while (!glfwWindowShouldClose(window)) {
        if (!camera.read(frame)) {
            break;
        }
        
        // Update texture data
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, videoTexture);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, frame.cols, frame.rows, 0, GL_BGR, GL_UNSIGNED_BYTE, frame.data);

        // Rendering
        glClearColor(0.2f, 0.3f, 0.3f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        ourShader.use();
        ourShader.setVec2("resolution", (float)frame.cols, (float)frame.rows);
        ourShader.setVec2("charSize", 8.0f, 16.0f); // The size of one character block. Tweak these!

        glBindVertexArray(VAO);
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // 8. Cleanup
    glDeleteVertexArrays(1, &VAO);
    glDeleteBuffers(1, &VBO);
    glDeleteBuffers(1, &EBO);
    glDeleteTextures(1, &videoTexture);
    glDeleteTextures(1, &fontTexture);
    glfwTerminate();
    return 0;
}
