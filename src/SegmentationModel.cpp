#include "SegmentationModel.h"
#include <fstream>
#include <iostream>
#include <vector>
#include <memory>
#include <cuda_runtime_api.h>
#include <NvOnnxParser.h>
#include <numeric>

// --- TRT Deleter and Logger Implementations (Unchanged) ---
template<typename T>
void fs::TrtDeleter<T>::operator()(T* obj) const {
    if (obj) {
        delete obj;
    }
}
template struct fs::TrtDeleter<nvinfer1::IBuilder>;
template struct fs::TrtDeleter<nvinfer1::IBuilderConfig>;
template struct fs::TrtDeleter<nvinfer1::ICudaEngine>;
template struct fs::TrtDeleter<nvinfer1::IExecutionContext>;
template struct fs::TrtDeleter<nvinfer1::IHostMemory>;
template struct fs::TrtDeleter<nvinfer1::INetworkDefinition>;
template struct fs::TrtDeleter<nvinfer1::IRuntime>;
template struct fs::TrtDeleter<nvonnxparser::IParser>;
void fs::Logger::log(Severity severity, const char* msg) noexcept {
    if (severity <= Severity::kINFO) { // Changed to kINFO for more verbose build output
        std::cout << msg << std::endl;
    }
}
// ---

fs::SegmentationModel::SegmentationModel(const std::string& onnxPath)
    : onnxFilePath(onnxPath) {
    size_t last_dot = onnxFilePath.find_last_of('.');
    engineFilePath = onnxFilePath.substr(0, last_dot) + ".trt";
}

fs::SegmentationModel::~SegmentationModel() {
    if (buffers[0]) cudaFree(buffers[0]);
    if (buffers[1]) cudaFree(buffers[1]);
    if (stream) cudaStreamDestroy(stream);
}

bool fs::SegmentationModel::init() {
    if (!loadEngine()) {
        std::cout << "Could not load engine. Building from ONNX file..." << std::endl;
        if (!buildEngine()) {
            std::cerr << "Failed to build engine." << std::endl;
            return false;
        }
    }
    context = TrtUniquePtr<nvinfer1::IExecutionContext>(engine->createExecutionContext());
    if (!context) {
        std::cerr << "Failed to create execution context." << std::endl;
        return false;
    }
    
    // Allocate buffers for input and output mask
    cudaMalloc(&buffers[0], inputElements * sizeof(float));
    cudaMalloc(&buffers[1], outputElements * sizeof(float));

    // Set tensor addresses based on the model's expected I/O tensor names
    context->setTensorAddress(engine->getIOTensorName(0), buffers[0]); // input
    context->setTensorAddress(engine->getIOTensorName(1), buffers[1]); // output
    
    cudaStreamCreate(&stream);
    std::cout << "Segmentation model initialized successfully." << std::endl;
    return true;
}

bool fs::SegmentationModel::loadEngine() {
    std::ifstream engineFile(engineFilePath, std::ios::binary);
    if (!engineFile) return false;
    engineFile.seekg(0, std::ios::end);
    long int fsize = engineFile.tellg();
    engineFile.seekg(0, std::ios::beg);
    std::vector<char> engineData(fsize);
    engineFile.read(engineData.data(), fsize);
    runtime = TrtUniquePtr<nvinfer1::IRuntime>(nvinfer1::createInferRuntime(logger));
    engine = TrtUniquePtr<nvinfer1::ICudaEngine>(runtime->deserializeCudaEngine(engineData.data(), fsize));
    return engine != nullptr;
}

bool fs::SegmentationModel::buildEngine() {
    auto builder = TrtUniquePtr<nvinfer1::IBuilder>(nvinfer1::createInferBuilder(logger));
    if (!builder) return false;
    auto network = TrtUniquePtr<nvinfer1::INetworkDefinition>(builder->createNetworkV2(0U));
    if (!network) return false;
    auto config = TrtUniquePtr<nvinfer1::IBuilderConfig>(builder->createBuilderConfig());
    if (!config) return false;
    auto parser = TrtUniquePtr<nvonnxparser::IParser>(nvonnxparser::createParser(*network, logger));
    if (!parser) return false;
    if (!parser->parseFromFile(onnxFilePath.c_str(), static_cast<int>(nvinfer1::ILogger::Severity::kWARNING))) {
        std::cerr << "Failed to parse ONNX file." << std::endl;
        return false;
    }
    
    auto profile = builder->createOptimizationProfile();
    auto input = network->getInput(0);
    const char* inputName = input->getName();
    
    nvinfer1::Dims dims = {4, {1, inputChannels, inputHeight, inputWidth}}; // BCHW
    
    profile->setDimensions(inputName, nvinfer1::OptProfileSelector::kMIN, dims);
    profile->setDimensions(inputName, nvinfer1::OptProfileSelector::kOPT, dims);
    profile->setDimensions(inputName, nvinfer1::OptProfileSelector::kMAX, dims);
    config->addOptimizationProfile(profile);
    config->setMemoryPoolLimit(nvinfer1::MemoryPoolType::kWORKSPACE, 1ULL << 30); // 1 GB
    
    auto serializedEngine = TrtUniquePtr<nvinfer1::IHostMemory>(builder->buildSerializedNetwork(*network, *config));
    if (!serializedEngine) {
        std::cerr << "Failed to build serialized engine." << std::endl;
        return false;
    }
    
    std::ofstream engineFile(engineFilePath, std::ios::binary);
    engineFile.write(reinterpret_cast<const char*>(serializedEngine->data()), serializedEngine->size());
    
    return loadEngine();
}

void fs::SegmentationModel::preprocess(const cv::Mat& inputImage, std::vector<float>& buffer) {
    // 1. Resize the image to the model's input size
    cv::Mat resizedImage;
    cv::resize(inputImage, resizedImage, cv::Size(inputWidth, inputHeight));

    // 2. Convert to float and normalize to [0, 1]
    cv::Mat floatImage;
    resizedImage.convertTo(floatImage, CV_32F, 1.0 / 255.0);

    // 3. Convert from HWC (interleaved BGR) to CHW (planar RGB)
    buffer.resize(inputElements);
    const int imageSize = inputWidth * inputHeight;
    // This loop rearranges the data. OpenCV reads as BGR, but many models expect RGB.
    // The original code was fine as it didn't swap channels, let's keep it that way
    // unless the model requires it. Assuming the model is fine with BGR planar.
    for (int c = 0; c < inputChannels; ++c) {
        for (int i = 0; i < imageSize; ++i) {
            buffer[c * imageSize + i] = floatImage.at<cv::Vec3f>(i)[c];
        }
    }
}

cv::Mat fs::SegmentationModel::infer(const cv::Mat& inputImage) {
    std::vector<float> inputBuffer;
    preprocess(inputImage, inputBuffer);
    cudaMemcpyAsync(buffers[0], inputBuffer.data(), inputBuffer.size() * sizeof(float), cudaMemcpyHostToDevice, stream);
    
    context->enqueueV3(stream);
    
    // Retrieve single output mask from the GPU
    std::vector<float> outputBuffer(outputElements);
    cudaMemcpyAsync(outputBuffer.data(), buffers[1], outputBuffer.size() * sizeof(float), cudaMemcpyDeviceToHost, stream);
    
    cudaStreamSynchronize(stream);

    // --- Post-processing for selfie_segmenter ---
    // 1. Create a Mat from the raw output buffer (which contains logits)
    cv::Mat outputMask(outputHeight, outputWidth, CV_32F, outputBuffer.data());
    
    // 2. Apply a sigmoid function to convert logits to probabilities (0.0 to 1.0)
    cv::Mat probabilityMask;
    cv::exp(-outputMask, probabilityMask);
    probabilityMask = 1.0 / (1.0 + probabilityMask);

    // 3. Resize the small probability mask to the original camera frame size
    cv::Mat resizedMask;
    cv::resize(probabilityMask, resizedMask, inputImage.size());

    // 4. Threshold the probabilities to get a binary mask and convert to CV_8UC1
    return resizedMask > 0.5;
}
