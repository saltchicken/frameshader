cmake_minimum_required(VERSION 3.18)
project(frameshader LANGUAGES CXX C CUDA)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# --- Dependencies ---
include(FetchContent)
FetchContent_Declare(
    cxxopts
    GIT_REPOSITORY https://github.com/jarro2783/cxxopts.git
    GIT_TAG        v3.1.1
)
FetchContent_MakeAvailable(cxxopts)

FetchContent_Declare(
    inih
    GIT_REPOSITORY https://github.com/benhoyt/inih.git
    GIT_TAG        r57
)
FetchContent_MakeAvailable(inih)

# --- Find System Packages ---
find_package(OpenCV REQUIRED COMPONENTS core highgui videoio imgproc imgcodecs)
find_package(OpenGL REQUIRED)
find_package(glfw3 3.3 REQUIRED)

# --- Manually Find CUDA Components ---
find_path(MANUAL_CUDA_INCLUDE_DIR cuda_runtime_api.h
    HINTS /opt/cuda
    PATH_SUFFIXES targets/x86_64-linux/include include
)
# --- FIX: Manually find the CUDA runtime library ---
find_library(MANUAL_CUDA_LIBRARY cudart
    HINTS /opt/cuda
    PATH_SUFFIXES lib64 lib
)


# --- Manually Find TensorRT ---
find_path(TensorRT_INCLUDE_DIR NvInfer.h
    HINTS /opt/tensorrt /usr/local/tensorrt /usr/include
    PATH_SUFFIXES include
)
find_library(TensorRT_INFER_LIBRARY nvinfer
    HINTS /opt/tensorrt /usr/local/tensorrt /usr/lib
    PATH_SUFFIXES lib lib64
)
find_library(TensorRT_ONNX_PARSER_LIBRARY nvonnxparser
    HINTS /opt/tensorrt /usr/local/tensorrt /usr/lib
    PATH_SUFFIXES lib lib64
)

# --- Library Setup ---
add_library(OpenCV_lib INTERFACE)
target_include_directories(OpenCV_lib INTERFACE ${OpenCV_INCLUDE_DIRS})
target_link_libraries(OpenCV_lib INTERFACE ${OpenCV_LIBS})

# --- Executable Setup ---
set(SOURCES
    src/main.cpp
    src/Application.cpp
    libs/glad/src/glad.c
    src/Shader.cpp
    src/Config.cpp
    src/Camera.cpp
    src/SegmentationModel.cpp
)
add_executable(${PROJECT_NAME} ${SOURCES})

target_include_directories(${PROJECT_NAME} PRIVATE 
    include
    ${CMAKE_CURRENT_SOURCE_DIR}/libs/glad/include
    ${inih_SOURCE_DIR}
    ${MANUAL_CUDA_INCLUDE_DIR}
)

target_link_libraries(${PROJECT_NAME} PRIVATE
    OpenCV_lib
    OpenGL::GL
    glfw
    cxxopts::cxxopts
    inih
    # --- FIX: Link the manually found CUDA library ---
    ${MANUAL_CUDA_LIBRARY}
)

# --- Link TensorRT Manually ---
if(TensorRT_INCLUDE_DIR AND TensorRT_INFER_LIBRARY AND TensorRT_ONNX_PARSER_LIBRARY)
    message(STATUS "Found TensorRT includes: ${TensorRT_INCLUDE_DIR}")
    
    target_include_directories(${PROJECT_NAME} PRIVATE ${TensorRT_INCLUDE_DIR})
    target_link_libraries(${PROJECT_NAME} PRIVATE 
        ${TensorRT_INFER_LIBRARY}
        ${TensorRT_ONNX_PARSER_LIBRARY}
    )
else()
    message(FATAL_ERROR "Manual search for TensorRT failed. Check HINTS in CMakeLists.txt and your installation.")
endif()

# --- File Copying ---
file(COPY shaders DESTINATION ${CMAKE_BINARY_DIR})
file(COPY font_atlases DESTINATION ${CMAKE_BINARY_DIR})
file(COPY models DESTINATION ${CMAKE_BINARY_DIR})
