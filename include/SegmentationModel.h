#pragma once

#include <string>
#include <memory>
#include <vector>
#include <opencv2/opencv.hpp>
#include <NvInfer.h>

namespace fs {

// Helper to manage TensorRT object lifetime
template<typename T>
struct TrtDeleter {
    void operator()(T* obj) const;
};

// Define a unique_ptr type with our custom deleter
template<typename T>
using TrtUniquePtr = std::unique_ptr<T, TrtDeleter<T>>;

class Logger : public nvinfer1::ILogger {
    void log(Severity severity, const char* msg) noexcept override;
};

class SegmentationModel {
public:
    SegmentationModel(const std::string& onnxPath);
    ~SegmentationModel();

    bool init();
    cv::Mat infer(const cv::Mat& inputImage);

private:
    bool buildEngine();
    bool loadEngine();
    void preprocess(const cv::Mat& inputImage, std::vector<float>& buffer);

    std::string onnxFilePath;
    std::string engineFilePath;

    Logger logger;
    
    // --- THIS IS THE FIX ---
    // Use our custom TrtUniquePtr for all TensorRT objects
    TrtUniquePtr<nvinfer1::IRuntime> runtime;
    TrtUniquePtr<nvinfer1::ICudaEngine> engine;
    TrtUniquePtr<nvinfer1::IExecutionContext> context;

    void* buffers[2]; // 0 for input, 1 for output
    cudaStream_t stream;

    // Model input dimensions
    const int inputWidth = 256;
    const int inputHeight = 256;
    const int inputChannels = 3;
    
    // Model output dimensions
    const int outputWidth = 256;
    const int outputHeight = 256;
    const int outputChannels = 1;
};

} // namespace fs
