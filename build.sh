#!/bin/bash

# Used for quickly building locally for clang and gcc

set -xeuo pipefail

BUILD=out/build
CONFIGURE_OPTIONS=""
BUILD_OPTIONS="-- -v"
TARGET=all

# rm -rf out/build
# rm -f $BUILD/unixlike-clang-debug/CMakeCache.txt $BUILD/unixlike-gcc-debug/CMakeCache.txt

cmake --preset=unixlike-clang-debug $CONFIGURE_OPTIONS
echo "######### clang configure #########"

cmake --build $BUILD/unixlike-clang-debug --target $TARGET $BUILD_OPTIONS
echo "######### clang build #########"

cmake --preset=unixlike-gcc-debug $CONFIGURE_OPTIONS
echo "######### gcc configure #########"

cmake --build $BUILD/unixlike-gcc-debug --target $TARGET $BUILD_OPTIONS
echo "######### gcc build #########"

$BUILD/unixlike-gcc-debug/test/tests
