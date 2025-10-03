cmake_minimum_required(VERSION 3.10)
project(frameshader)

cmake_policy(SET CMP0072 NEW)
set(CMAKE_CXX_STANDARD 17)

# Find necessary packages
find_package(OpenCV REQUIRED COMPONENTS core highgui videoio imgproc imgcodecs)
find_package(OpenGL REQUIRED)
find_package(glfw3 3.3 REQUIRED)

include_directories(include)

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
)
