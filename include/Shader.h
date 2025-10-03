#ifndef SHADER_H
#define SHADER_H

#include <glad/glad.h>
#include <string>

class Shader {
public:
    // The program ID
    unsigned int ID;

    // Constructor reads and builds the shader
    Shader(const char* vertexPath, const char* fragmentPath);

    // Use/activate the shader
    void use();
    
    // Utility uniform functions
    void setBool(const std::string &name, bool value) const;
    void setInt(const std::string &name, int value) const;
    void setFloat(const std::string &name, float value) const;
    void setVec2(const std::string &name, float v1, float v2) const;

private:
    // Utility function for checking shader compilation/linking errors.
    void checkCompileErrors(unsigned int shader, std::string type);
};

#endif
