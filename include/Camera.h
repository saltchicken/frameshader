#pragma once

#include <opencv2/opencv.hpp>

class Camera {
public:
    Camera(int deviceID = 0, int width = 1920, int height = 1080);
    ~Camera();

    bool isOpened() const;
    bool read(cv::Mat& frame);
    
    int getWidth() const;
    int getHeight() const;

private:
    cv::VideoCapture cap;
    int frameWidth;
    int frameHeight;
};
