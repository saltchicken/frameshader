#include "Config.h"
#include <iostream>
#include <cstdlib>
#include <cstdint>
#include <cxxopts.hpp>
#include <ini.h>

// INI parser callback
static int config_handler(void* user, const char* section, const char* name, const char* value) {
    AppConfig* pconfig = (AppConfig*)user;

    if (std::string(section) == "camera") {
        if (std::string(name) == "device") pconfig->cameraDeviceID = std::stoi(value);
        else if (std::string(name) == "width") pconfig->cameraWidth = std::stoi(value);
        else if (std::string(name) == "height") pconfig->cameraHeight = std::stoi(value);
    } 
    else if (std::string(section) == "ascii_shader") {
        if (std::string(name) == "char_width") pconfig->asciiCharWidth = std::stof(value);
        else if (std::string(name) == "char_height") pconfig->asciiCharHeight = std::stof(value);
    }
    return 1;
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
            ("cw,char-width", "ASCII shader character width", cxxopts::value<float>())
            ("ch,char-height", "ASCII shader character height", cxxopts::value<float>())
            ("help", "Print help");

        auto result = options.parse(argc, argv);

        if (result.count("help")) {
            std::cout << options.help() << std::endl;
            exit(0);
        }
        
        if (result.count("device")) config.cameraDeviceID = result["device"].as<int>();
        if (result.count("width")) config.cameraWidth = result["width"].as<int>();
        if (result.count("height")) config.cameraHeight = result["height"].as<int>();
        if (result.count("char-width")) config.asciiCharWidth = result["char-width"].as<float>();
        if (result.count("char-height")) config.asciiCharHeight = result["char-height"].as<float>();

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
