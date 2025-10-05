cmake_minimum_required(VERSION 3.11)
project(frameshader)

cmake_policy(SET CMP0072 NEW)
set(CMAKE_CXX_STANDARD 17)

# Find necessary packages
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

find_package(OpenCV REQUIRED COMPONENTS core highgui videoio imgproc imgcodecs)
find_package(OpenGL REQUIRED)
find_package(glfw3 3.3 REQUIRED)

include_directories(
  include
  ${inih_SOURCE_DIR}
)

# Add the executable with all its source files
add_executable(${PROJECT_NAME} 
    src/main.cpp 
    src/glad.c
    src/Shader.cpp
    src/Camera.cpp
)

file(COPY shaders DESTINATION ${CMAKE_BINARY_DIR})

# Link all the required libraries
target_link_libraries(${PROJECT_NAME} PRIVATE
    ${OpenCV_LIBS}
    ${OPENGL_LIBRARIES}
    glfw
    cxxopts::cxxopts
    inih
)
