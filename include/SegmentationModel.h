#pragma once

#include <string>
#include <vector>
#include <memory>
#include <opencv2/opencv.hpp>
#include <NvInfer.h>

namespace fs {

// Smart pointer deleter for TensorRT objects
template <typename T>
struct TrtDeleter {
    void operator()(T* obj) const;
};

template <typename T>
using TrtUniquePtr = std::unique_ptr<T, TrtDeleter<T>>;

class Logger : public nvinfer1::ILogger {
    void log(Severity severity, const char* msg) noexcept override;
};

class SegmentationModel {
public:
    explicit SegmentationModel(const std::string& onnxPath);
    ~SegmentationModel();

    bool init();
    cv::Mat infer(const cv::Mat& inputImage);

private:
    bool loadEngine();
    bool buildEngine();
    void preprocess(const cv::Mat& inputImage, std::vector<float>& buffer);

    // --- MODEL PARAMETERS ---
    const int inputWidth = 640;
    const int inputHeight = 640;
    const int inputChannels = 3;

    // YOLOv11-Seg specific outputs
    const int numClasses = 80;
    const int numMaskCoeffs = 32;
    const int outputDetElements = 8400 * (4 + numClasses + numMaskCoeffs); 
    const int outputMaskWidth = 160;
    const int outputMaskHeight = 160;
    const int outputMaskElements = numMaskCoeffs * outputMaskWidth * outputMaskHeight;
    cv::Mat lastGoodMask;
    int framesWithoutDetection = 0;

    // --- CLASS MEMBERS ---
    std::string onnxFilePath;
    std::string engineFilePath;
    Logger logger;
    TrtUniquePtr<nvinfer1::IRuntime> runtime;
    TrtUniquePtr<nvinfer1::ICudaEngine> engine;
    TrtUniquePtr<nvinfer1::IExecutionContext> context;
    cudaStream_t stream = nullptr;
    
    void* buffers[3]{nullptr, nullptr, nullptr}; 
    
    // --- Pre-processing helpers ---
    float scale;
    int padX, padY;
};

} // namespace fs
