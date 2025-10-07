#include "Config.h"
#include <iostream>
#include <cstdlib>
#include <cstdint>
#include <cxxopts.hpp>
#include <ini.h>

static int config_handler(void* user, const char* section, const char* name, const char* value) {
    AppConfig* pconfig = (AppConfig*)user;
    
    // Handle camera settings
    if (strcmp(section, "camera") == 0) {
        if (strcmp(name, "device") == 0) pconfig->cameraDeviceID = std::stoi(value);
        else if (strcmp(name, "width") == 0) pconfig->cameraWidth = std::stoi(value);
        else if (strcmp(name, "height") == 0) pconfig->cameraHeight = std::stoi(value);
        return 1;
    }
    
    // ## NEW LOGIC ##
    // Handle dynamic font profile sections like [font:default]
    const char* font_prefix = "font:";
    if (strncmp(section, font_prefix, strlen(font_prefix)) == 0) {
        // Extract the profile name (e.g., "default" from "font:default")
        std::string profileName = section + strlen(font_prefix);
        
        // Get or create the FontProfile for this name
        FontProfile& profile = pconfig->fontProfiles[profileName];

        // Set the property on the profile
        if (strcmp(name, "path") == 0) profile.path = value;
        else if (strcmp(name, "char_width") == 0) profile.charWidth = std::stof(value);
        else if (strcmp(name, "char_height") == 0) profile.charHeight = std::stof(value);
        else if (strcmp(name, "num_chars") == 0) profile.numChars = std::stof(value);
        return 1;
    }

    const char* shader_prefix = "shader:";
    if (strncmp(section, shader_prefix, strlen(shader_prefix)) == 0) {
        std::string shaderName = section + strlen(shader_prefix);
        
        // Get the map for this shader
        ShaderConfig& shaderConf = pconfig->shaderConfigs[shaderName];

        // SIMPLIFIED: Store any key-value pair directly into the map
        shaderConf[name] = std::stof(value);
        
        return 1;
    }

    return 1; // Return 1 on success
}

// Load from INI file
static void load_from_ini(AppConfig& config) {
    const char* homeDir = getenv("HOME");
    if (!homeDir) return;
    std::string configPath = std::string(homeDir) + "/.config/frame_shader/config.ini";
    ini_parse(configPath.c_str(), config_handler, &config);
}

// Parse command-line arguments
static void parse_from_args(int argc, char* argv[], AppConfig& config) {
    try {
        cxxopts::Options options(argv[0], "ASCII camera shader");
        options.add_options()
            ("d,device", "Camera device ID", cxxopts::value<int>())
            ("w,width", "Camera frame width", cxxopts::value<int>())
            ("h,height", "Camera frame height", cxxopts::value<int>())
            ("f,font", "Font profile", cxxopts::value<std::string>())
            ("help", "Print help");

        auto result = options.parse(argc, argv);

        if (result.count("help")) {
            std::cout << options.help() << std::endl;
            exit(0);
        }
        
        if (result.count("device")) config.cameraDeviceID = result["device"].as<int>();
        if (result.count("width")) config.cameraWidth = result["width"].as<int>();
        if (result.count("height")) config.cameraHeight = result["height"].as<int>();
        if (result.count("font")) config.selectedFontProfile = result["font"].as<std::string>(); // ## MODIFIED ##


    } catch (const cxxopts::exceptions::exception& e) {
        std::cerr << "Error parsing options: " << e.what() << std::endl;
        exit(1);
    }
}

// Public interface
AppConfig load_configuration(int argc, char* argv[]) {
    AppConfig config; // Start with default values
    load_from_ini(config); // Override with INI file
    parse_from_args(argc, argv, config); // Override with command-line args
    return config;
}
