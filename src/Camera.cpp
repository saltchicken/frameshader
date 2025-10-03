#include "Camera.h"
#include <iostream>

Camera::Camera(int deviceID, int width, int height) {
    cap.open(deviceID);
    if (!cap.isOpened()) {
        std::cerr << "ERROR: Could not open camera." << std::endl;
        return;
    }

    cap.set(cv::CAP_PROP_FRAME_WIDTH, width);
    cap.set(cv::CAP_PROP_FRAME_HEIGHT, height);

    frameWidth = cap.get(cv::CAP_PROP_FRAME_WIDTH);
    frameHeight = cap.get(cv::CAP_PROP_FRAME_HEIGHT);

    std::cout << "Camera initialized with resolution: " 
              << frameWidth << "x" << frameHeight << std::endl;
}

Camera::~Camera() {
    if (cap.isOpened()) {
        cap.release();
    }
}

bool Camera::isOpened() const {
    return cap.isOpened();
}

bool Camera::read(cv::Mat& frame) {
    return cap.read(frame);
}

int Camera::getWidth() const {
    return frameWidth;
}

int Camera::getHeight() const {
    return frameHeight;
}
