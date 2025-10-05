#include "Config.h"
#include <iostream>
#include <string>
#include <cstdlib>
#include <cstdint>
#include <cxxopts.hpp>
#include <ini.h>

// The callback for the INI parser
static int config_handler(void* user, const char* section, const char* name,
                          const char* value) {
    AppConfig* pconfig = (AppConfig*)user;

    // We only care about the [camera] section for now
    if (std::string(section) == "camera") {
        if (std::string(name) == "device") {
            pconfig->cameraDeviceID = std::stoi(value);
        } else if (std::string(name) == "width") {
            pconfig->cameraWidth = std::stoi(value);
        } else if (std::string(name) == "height") {
            pconfig->cameraHeight = std::stoi(value);
        }
    }
    else if (std::string(section) == "shader") {
      if (std::string(name) == "name") {
              pconfig->shaderName = std::string(value);
          }
      }
    return 1;
}

// Internal function to load settings from our config file
static void load_from_ini(AppConfig& config) {
    const char* homeDir = getenv("HOME");
    if (!homeDir) {
        std::cerr << "Could not get HOME directory." << std::endl;
        return;
    }
    std::string configPath = std::string(homeDir) + "/.config/frame_shader/config.ini";

    if (ini_parse(configPath.c_str(), config_handler, &config) < 0) {
    // TODO: Make sure that the absence of a config file is not fatal
        std::cout << "No config file found at " << configPath << ". Using defaults." << std::endl;
    }
}

// Internal function to parse command line arguments and override config
static void parse_from_args(int argc, char* argv[], AppConfig& config) {
    try {
        cxxopts::Options options(argv[0], "A real-time camera shader application");
        options.add_options()
            ("d,device", "Camera device ID", cxxopts::value<int>())
            ("w,width", "Camera frame width", cxxopts::value<int>())
            ("h,height", "Camera frame height", cxxopts::value<int>())
            // TODO: Add all shader options
            ("s,shader", "Shader name (pixelate, wave, ascii", cxxopts::value<std::string>())
            ("help", "Print help");

        auto result = options.parse(argc, argv);

        if (result.count("help")) {
            std::cout << options.help() << std::endl;
            exit(0);
        }
        
        // Only update config if the argument was actually passed
        if (result.count("device")) {
            config.cameraDeviceID = result["device"].as<int>();
        }
        if (result.count("width")) {
            config.cameraWidth = result["width"].as<int>();
        }
        if (result.count("height")) {
            config.cameraHeight = result["height"].as<int>();
        }
        if (result.count("shader")) {
            config.shaderName = result["shader"].as<std::string>();
        }
    } catch (const cxxopts::exceptions::exception& e) {
        std::cerr << "Error parsing options: " << e.what() << std::endl;
        exit(1);
    }
}

// --- Public Interface ---

AppConfig load_configuration(int argc, char* argv[]) {
    AppConfig config; // Start with default values from the struct definition
    load_from_ini(config); // Override with values from config.ini
    parse_from_args(argc, argv, config); // Finally, override with command line arguments
    return config;
}
