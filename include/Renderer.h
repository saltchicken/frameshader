#pragma once

#include <glad/glad.h>
#include <opencv2/core/mat.hpp>

class Renderer {
public:
    Renderer();
    ~Renderer();

    // Deleted copy constructor and assignment operator to prevent copying
    Renderer(const Renderer&) = delete;
    Renderer& operator=(const Renderer&) = delete;

    // Initializes the video texture with the first frame
    void initializeTexture(const cv::Mat& firstFrame);
    
    // Updates the video texture with a new frame
    void updateTexture(const cv::Mat& frame);
    
    // Draws the quad
    void draw() const;

private:
    GLuint m_vao;
    GLuint m_vbo;
    GLuint m_ebo;
    GLuint m_videoTexture;
};

