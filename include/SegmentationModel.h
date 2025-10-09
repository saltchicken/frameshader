#ifndef SEGMENTATION_MODEL_H
#define SEGMENTATION_MODEL_H

#include <string>
#include <vector>
#include <memory>
#include <opencv2/opencv.hpp>
#include <NvInfer.h>
#include <cuda_runtime_api.h>

namespace fs {

// Logger for TensorRT info/warning/errors
class Logger : public nvinfer1::ILogger {
    void log(Severity severity, const char* msg) noexcept override;
};

// RAII wrapper for TensorRT objects
template<typename T>
struct TrtDeleter {
    void operator()(T* obj) const;
};

template<typename T>
using TrtUniquePtr = std::unique_ptr<T, TrtDeleter<T>>;


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

    // --- Model Parameters for selfie_segmenter_landscape ---
    const int inputWidth = 256;
    const int inputHeight = 144;
    const int inputChannels = 3;
    const int outputWidth = 256;
    const int outputHeight = 144;
    const int64_t inputElements = inputWidth * inputHeight * inputChannels;
    const int64_t outputElements = outputWidth * outputHeight;
    // ---

    std::string onnxFilePath;
    std::string engineFilePath;
    Logger logger;
    
    TrtUniquePtr<nvinfer1::IRuntime> runtime;
    TrtUniquePtr<nvinfer1::ICudaEngine> engine;
    TrtUniquePtr<nvinfer1::IExecutionContext> context;

    void* buffers[2]{}; // Input, Output
    cudaStream_t stream{};
};

} // namespace fs

#endif // SEGMENTATION_MODEL_H
