cmake_minimum_required(VERSION 3.11)
project(frameshader)

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

find_package(OpenCV REQUIRED COMPONENTS core highgui videoio imgproc imgcodecs dnn)
find_package(OpenGL REQUIRED)
find_package(glfw3 3.3 REQUIRED)

add_library(OpenCV_lib INTERFACE)
target_include_directories(OpenCV_lib INTERFACE ${OpenCV_INCLUDE_DIRS})
target_link_libraries(OpenCV_lib INTERFACE ${OpenCV_LIBS})

set(SOURCES
    src/main.cpp
    src/glad.c
    src/Shader.cpp
    src/Config.cpp
    src/Camera.cpp
)

add_executable(${PROJECT_NAME} ${SOURCES})

target_include_directories(${PROJECT_NAME} PRIVATE 
    include
    ${inih_SOURCE_DIR}
)

target_link_libraries(${PROJECT_NAME} PRIVATE
    OpenCV_lib
    OpenGL::GL
    glfw
    cxxopts::cxxopts
    inih
)

file(COPY shaders DESTINATION ${CMAKE_BINARY_DIR})
file(COPY font_atlases DESTINATION ${CMAKE_BINARY_DIR})
file(COPY models DESTINATION ${CMAKE_BINARY_DIR})
