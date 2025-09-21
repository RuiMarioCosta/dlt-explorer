#!/bin/bash

# Used for quickly building locally for clang and gcc

set -xeuo pipefail

BUILD=out/build

PRESET=unixlike-gcc-debug
# PRESET=unixlike-clang-debug
CONFIGURE_OPTIONS=""
BUILD_OPTIONS="-- -v"
TARGET=all

# rm -rf out/build
# rm -f $BUILD/unixlike-clang-debug/CMakeCache.txt $BUILD/unixlike-gcc-debug/CMakeCache.txt

echo "######### configure #########" && cmake --preset=$PRESET $CONFIGURE_OPTIONS

echo "######### build #########" && cmake --build $BUILD/$PRESET --target $TARGET $BUILD_OPTIONS

$BUILD/$PRESET/test/tests
