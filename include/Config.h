#pragma once
#include <string>
#include <vector>
#include <map>

struct FontProfile {
    std::string path;
    float charWidth = 8.0f;
    float charHeight = 16.0f;
    float numChars = 10.0f;
};

struct AppConfig {
  int cameraDeviceID = 0;
  int cameraWidth = 1920;
  int cameraHeight = 1080;

  float asciiSensitivity = 1.0f;

  // A map of available font profiles, keyed by name (e.g., "default").
  std::map<std::string, FontProfile> fontProfiles;
  // The name of the font profile to use, chosen via command line.
  std::string selectedFontProfile = "default";

};

AppConfig load_configuration(int argc, char* argv[]);
