#include "SegmentationModel.h"
#include <fstream>
#include <iostream>
#include <vector>
#include <memory>
#include <cuda_runtime_api.h>
#include <NvOnnxParser.h>
#include <numeric>

// TrtDeleter and Logger implementations remain the same...
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
    if (severity <= Severity::kWARNING) {
        std::cout << msg << std::endl;
    }
}


fs::SegmentationModel::SegmentationModel(const std::string& onnxPath)
    : onnxFilePath(onnxPath), scale(1.0f), padX(0), padY(0) {
    size_t last_dot = onnxFilePath.find_last_of('.');
    engineFilePath = onnxFilePath.substr(0, last_dot) + ".trt";
}

fs::SegmentationModel::~SegmentationModel() {
    if (buffers[0]) cudaFree(buffers[0]);
    if (buffers[1]) cudaFree(buffers[1]);
    if (buffers[2]) cudaFree(buffers[2]);
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
    
    // Allocate buffers for input, detection output, and mask output
    cudaMalloc(&buffers[0], inputWidth * inputHeight * inputChannels * sizeof(float));
    cudaMalloc(&buffers[1], outputDetElements * sizeof(float));
    cudaMalloc(&buffers[2], outputMaskElements * sizeof(float));

    // Set tensor addresses
    context->setTensorAddress(engine->getIOTensorName(0), buffers[0]); // input
    context->setTensorAddress(engine->getIOTensorName(1), buffers[1]); // detections
    context->setTensorAddress(engine->getIOTensorName(2), buffers[2]); // mask prototypes

    cudaStreamCreate(&stream);
    std::cout << "Segmentation model initialized successfully." << std::endl;
    return true;
}

bool fs::SegmentationModel::loadEngine() {
    // This function remains the same as before
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
    // This function is updated for YOLO's input dimensions
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
    nvinfer1::Dims dims = {4, {1, 3, inputHeight, inputWidth}}; // BCHW
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
    // Pre-process with letterboxing to maintain aspect ratio
    int originalWidth = inputImage.cols;
    int originalHeight = inputImage.rows;

    scale = std::min(static_cast<float>(inputWidth) / originalWidth, static_cast<float>(inputHeight) / originalHeight);
    int newWidth = static_cast<int>(originalWidth * scale);
    int newHeight = static_cast<int>(originalHeight * scale);

    cv::Mat resizedImage;
    cv::resize(inputImage, resizedImage, cv::Size(newWidth, newHeight));

    padX = (inputWidth - newWidth) / 2;
    padY = (inputHeight - newHeight) / 2;
    
    cv::Mat paddedImage(inputHeight, inputWidth, CV_8UC3, cv::Scalar(114, 114, 114));
    resizedImage.copyTo(paddedImage(cv::Rect(padX, padY, newWidth, newHeight)));
    
    cv::Mat floatImage;
    paddedImage.convertTo(floatImage, CV_32F, 1.0 / 255.0);
    
    const int imageSize = inputWidth * inputHeight;
    buffer.resize(inputChannels * imageSize);
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
    
    std::vector<float> outputDetBuffer(outputDetElements);
    std::vector<float> outputMaskBuffer(outputMaskElements);
    cudaMemcpyAsync(outputDetBuffer.data(), buffers[1], outputDetBuffer.size() * sizeof(float), cudaMemcpyDeviceToHost, stream);
    cudaMemcpyAsync(outputMaskBuffer.data(), buffers[2], outputMaskBuffer.size() * sizeof(float), cudaMemcpyDeviceToHost, stream);
    
    cudaStreamSynchronize(stream);

    // --- Start Post-processing ---
    
    // MOVED: Declarations and transposition now happen before the main logic
    std::vector<float> transposedDetBuffer(outputDetElements);
    int n_rows = 4 + numClasses + numMaskCoeffs;
    int n_cols = 8400;
    for(int i = 0; i < n_rows; ++i) {
        for(int j = 0; j < n_cols; ++j) {
            transposedDetBuffer[j * n_rows + i] = outputDetBuffer[i * n_cols + j];
        }
    }
    
    int bestDetIndex = -1;
    float maxConf = 0.5f; // Confidence threshold

    // Find the detection with the highest confidence
    for (int i = 0; i < n_cols; ++i) {
        float* det = transposedDetBuffer.data() + i * n_rows;
        if (det[4] > maxConf) {
            maxConf = det[4];
            bestDetIndex = i;
        }
    }
    
    if (bestDetIndex != -1) {
        // --- A detection was successful ---
        framesWithoutDetection = 0; // Reset the counter

        float* bestDet = transposedDetBuffer.data() + bestDetIndex * n_rows;
        
        cv::Mat maskCoeffs(1, numMaskCoeffs, CV_32F, bestDet + 4 + numClasses);
        cv::Mat proto(numMaskCoeffs, outputMaskWidth * outputMaskHeight, CV_32F, outputMaskBuffer.data());
        cv::Mat matmulResult = maskCoeffs * proto;
        cv::Mat sigmoidResult(outputMaskHeight, outputMaskWidth, CV_32F, matmulResult.data);
        
        cv::exp(-sigmoidResult, sigmoidResult);
        sigmoidResult = 1.0 / (1.0 + sigmoidResult);
        
        int unpaddedWidth = inputWidth - 2 * padX;
        int unpaddedHeight = inputHeight - 2 * padY;
        cv::Rect roi(padX / 4, padY / 4, unpaddedWidth / 4, unpaddedHeight / 4);
        cv::Mat unpaddedMask = sigmoidResult(roi);

        cv::Mat resizedMask;
        cv::resize(unpaddedMask, resizedMask, inputImage.size());
        
        lastGoodMask = resizedMask > 0.5;
        return lastGoodMask;

    } else {
        // --- Detection failed, use the smoothing logic ---
        framesWithoutDetection++;

        if (framesWithoutDetection <= 5 && !lastGoodMask.empty()) {
            return lastGoodMask;
        }
    }

    // If detection fails for too long, return an empty mask
    return cv::Mat::zeros(inputImage.rows, inputImage.cols, CV_8UC1);
}
