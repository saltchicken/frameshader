#pragma once
#include <string>
#include <vector>
#include <map>

// A map where keys are uniform names (e.g., "sensitivity") and values are floats
using ShaderConfig = std::map<std::string, float>;
// A map where keys are property names (e.g., "char_width") and values are floats
using FontConfig = std::map<std::string, float>;

// ++ ADD THIS STRUCT DEFINITION ++
struct FontProfile {
    std::string path;
    float charWidth  = 8.0f;
    float charHeight = 16.0f;
    float numChars   = 10.0f;
};

struct AppConfig {
    int cameraDeviceID = 0;
    int cameraWidth = 1920;
    int cameraHeight = 1080;
    std::string selectedFontProfile = "dejavu_sans_mono-10-8x16";

    // Maps a font profile name (e.g., "default") to its specific settings
    std::map<std::string, FontConfig> fontConfigs;
    // Maps a shader name (e.g., "ascii_matrix") to its specific settings
    std::map<std::string, ShaderConfig> shaderConfigs;
};

AppConfig load_configuration(int argc, char* argv[]);
void load_from_ini(AppConfig& config);
