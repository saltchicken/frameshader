#!/bin/bash

set -e

BUILD_DIR="build"

echo "--- Cleaning and preparing build directory ---"
rm -rf "$BUILD_DIR"
mkdir "$BUILD_DIR"

echo "--- Configuring with CMake ---"
cmake -S . -B $BUILD_DIR -D CMAKE_BUILD_TYPE=Debug

echo "--- Building project ---"
cmake --build $BUILD_DIR

echo "--- Build complete! ---"
