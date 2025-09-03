$BUILD = "out/build"

#$PRESET = "windows-msvc-debug-user-mode"
#$PRESET = "windows-clang-release"
$PRESET = "windows-clang-debug"
$CONFIGURE_OPTIONS = "-DBUILD_SHARED_LIBS=OFF"
$BUILD_OPTIONS = "-v"
$TARGET = "all"

#Remove-Item -Path "$BUILD" -Recurse -Force -Confirm
#rm "$BUILD/$PRESET/CMakeCache.txt"

cmake --preset=$PRESET $CONFIGURE_OPTIONS
Write-Output "######### configure #########"

cmake --build "$BUILD/$PRESET" --target "$TARGET" "$BUILD_OPTIONS"
Write-Output "######### build #########"

