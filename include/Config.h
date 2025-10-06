#pragma once
#include <string>

struct AppConfig {
  int cameraDeviceID = 0;
  int cameraWidth = 1920;
  int cameraHeight = 1080;

  // ASCII Effect-specific settings
  float asciiCharWidth = 8.0f;
  float asciiCharHeight = 16.0f;
  float asciiSensitivity = 1.0f;
  float asciiNumChars = 10.0f;

};

AppConfig load_configuration(int argc, char* argv[]);
