#pragma once

#include <glad/glad.h>
#include <string>
#include <unordered_map>

class Shader {
public:
    // The shader program ID
    unsigned int ID;

    // Constructor: reads and builds the shader from source files
    Shader(const char* vertexPath, const char* fragmentPath);
    // Destructor
    ~Shader();

    // Activates the shader program
    void use() const;
    bool usesUniform(const std::string& name) const;

    // Utility uniform functions
    void setBool(const std::string &name, bool value) const;
    void setInt(const std::string &name, int value) const;
    void setFloat(const std::string &name, float value) const;
    void setVec2(const std::string &name, float v1, float v2) const;

private:
    // Caches uniform locations for performance
    mutable std::unordered_map<std::string, GLint> uniformLocationCache;
    GLint getUniformLocation(const std::string &name) const;
    
    // Utility function for checking shader compilation/linking errors.
    void checkCompileErrors(GLuint shader, std::string type);
};
