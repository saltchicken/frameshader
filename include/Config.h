#pragma once
#include <string>
#include <vector>
#include <map>

using ShaderConfig = std::map<std::string, float>;
using FontConfig = std::map<std::string, float>;

struct FontProfile {
    std::string path;
    float charWidth = 10.0f;
    float charHeight = 20.0f;
    float numChars = 95.0f;
};

struct AppConfig {
    // Camera settings
    int cameraDeviceID = 0;
    int cameraWidth = 1280;
    int cameraHeight = 720;

    // The name of the font profile to load initially
    // TODO: Make this dynamic or just pick from the first available
    std::string selectedFontProfile = "dejavu_sans_mono-20-8x16";

    // Maps to hold overrides from the .ini file
    std::map<std::string, ShaderConfig> shaderConfigs;
    std::map<std::string, FontConfig> fontConfigs; // <-- MODIFIED
};

// Public interface for loading configuration
AppConfig load_configuration(int argc, char* argv[]);
