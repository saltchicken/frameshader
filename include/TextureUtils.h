#pragma once
#include <glad/glad.h>

// A utility function to load an image file into an OpenGL texture
void loadTextureFromFile(const char* path, GLuint& textureID, GLenum textureUnit);
