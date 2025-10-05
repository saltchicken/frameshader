#pragma once
#include <string>

struct AppConfig {
  int cameraDeviceID = 0;
  int cameraWidth = 1920;
  int cameraHeight = 1080;

  std::string shaderName = "pixelate";

};

AppConfig load_configuration(int argc, char* argv[]);
