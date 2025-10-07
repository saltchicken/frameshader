#pragma once

#include <string>
#include <vector>
#include <map>

// FontProfile remains the same
struct FontProfile {
    std::string path;
    float charWidth = 0.0f;
    float charHeight = 0.0f;
    float numChars = 0.0f;
};

// MODIFIED: ShaderConfig is now a flexible map
using ShaderConfig = std::map<std::string, float>;

// Main application configuration struct
struct AppConfig {
    // Camera settings
    int cameraDeviceID = 0;
    int cameraWidth = 1920;
    int cameraHeight = 1080;

    // Font settings
    std::map<std::string, FontProfile> fontProfiles;
    std::string selectedFontProfile = "default";

    // This map now holds ShaderConfig maps
    std::map<std::string, ShaderConfig> shaderConfigs;
};

AppConfig load_configuration(int argc, char* argv[]);
