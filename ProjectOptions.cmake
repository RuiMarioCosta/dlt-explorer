include(cmake/SystemLink.cmake)
include(cmake/LibFuzzer.cmake)
include(CMakeDependentOption)
include(CheckCXXCompilerFlag)


include(CheckCXXSourceCompiles)


macro(dlt_explorer_supports_sanitizers)
  if((CMAKE_CXX_COMPILER_ID MATCHES ".*Clang.*" OR CMAKE_CXX_COMPILER_ID MATCHES ".*GNU.*") AND NOT WIN32)

    message(STATUS "Sanity checking UndefinedBehaviorSanitizer, it should be supported on this platform")
    set(TEST_PROGRAM "int main() { return 0; }")

    # Check if UndefinedBehaviorSanitizer works at link time
    set(CMAKE_REQUIRED_FLAGS "-fsanitize=undefined")
    set(CMAKE_REQUIRED_LINK_OPTIONS "-fsanitize=undefined")
    check_cxx_source_compiles("${TEST_PROGRAM}" HAS_UBSAN_LINK_SUPPORT)

    if(HAS_UBSAN_LINK_SUPPORT)
      message(STATUS "UndefinedBehaviorSanitizer is supported at both compile and link time.")
      set(SUPPORTS_UBSAN ON)
    else()
      message(WARNING "UndefinedBehaviorSanitizer is NOT supported at link time.")
      set(SUPPORTS_UBSAN OFF)
    endif()
  else()
    set(SUPPORTS_UBSAN OFF)
  endif()

  if((CMAKE_CXX_COMPILER_ID MATCHES ".*Clang.*" OR CMAKE_CXX_COMPILER_ID MATCHES ".*GNU.*") AND WIN32)
    set(SUPPORTS_ASAN OFF)
  else()
    if (NOT WIN32)
      message(STATUS "Sanity checking AddressSanitizer, it should be supported on this platform")
      set(TEST_PROGRAM "int main() { return 0; }")

      # Check if AddressSanitizer works at link time
      set(CMAKE_REQUIRED_FLAGS "-fsanitize=address")
      set(CMAKE_REQUIRED_LINK_OPTIONS "-fsanitize=address")
      check_cxx_source_compiles("${TEST_PROGRAM}" HAS_ASAN_LINK_SUPPORT)

      if(HAS_ASAN_LINK_SUPPORT)
        message(STATUS "AddressSanitizer is supported at both compile and link time.")
        set(SUPPORTS_ASAN ON)
      else()
        message(WARNING "AddressSanitizer is NOT supported at link time.")
        set(SUPPORTS_ASAN OFF)
      endif()
    else()
      set(SUPPORTS_ASAN ON)
    endif()
  endif()
endmacro()

macro(dlt_explorer_setup_options)
  option(dlt_explorer_ENABLE_HARDENING "Enable hardening" ON)
  option(dlt_explorer_ENABLE_COVERAGE "Enable coverage reporting" OFF)
  cmake_dependent_option(
    dlt_explorer_ENABLE_GLOBAL_HARDENING
    "Attempt to push hardening options to built dependencies"
    ON
    dlt_explorer_ENABLE_HARDENING
    OFF)

  dlt_explorer_supports_sanitizers()

  if(NOT PROJECT_IS_TOP_LEVEL OR dlt_explorer_PACKAGING_MAINTAINER_MODE)
    option(dlt_explorer_ENABLE_IPO "Enable IPO/LTO" OFF)
    option(dlt_explorer_WARNINGS_AS_ERRORS "Treat Warnings As Errors" OFF)
    option(dlt_explorer_ENABLE_USER_LINKER "Enable user-selected linker" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_ADDRESS "Enable address sanitizer" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_LEAK "Enable leak sanitizer" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_UNDEFINED "Enable undefined sanitizer" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_THREAD "Enable thread sanitizer" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_MEMORY "Enable memory sanitizer" OFF)
    option(dlt_explorer_ENABLE_UNITY_BUILD "Enable unity builds" OFF)
    option(dlt_explorer_ENABLE_CLANG_TIDY "Enable clang-tidy" OFF)
    option(dlt_explorer_ENABLE_CPPCHECK "Enable cpp-check analysis" OFF)
    option(dlt_explorer_ENABLE_PCH "Enable precompiled headers" OFF)
    option(dlt_explorer_ENABLE_CACHE "Enable ccache" OFF)
  else()
    option(dlt_explorer_ENABLE_IPO "Enable IPO/LTO" ON)
    option(dlt_explorer_WARNINGS_AS_ERRORS "Treat Warnings As Errors" ON)
    option(dlt_explorer_ENABLE_USER_LINKER "Enable user-selected linker" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_ADDRESS "Enable address sanitizer" ${SUPPORTS_ASAN})
    option(dlt_explorer_ENABLE_SANITIZER_LEAK "Enable leak sanitizer" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_UNDEFINED "Enable undefined sanitizer" ${SUPPORTS_UBSAN})
    option(dlt_explorer_ENABLE_SANITIZER_THREAD "Enable thread sanitizer" OFF)
    option(dlt_explorer_ENABLE_SANITIZER_MEMORY "Enable memory sanitizer" OFF)
    option(dlt_explorer_ENABLE_UNITY_BUILD "Enable unity builds" OFF)
    option(dlt_explorer_ENABLE_CLANG_TIDY "Enable clang-tidy" ON)
    option(dlt_explorer_ENABLE_CPPCHECK "Enable cpp-check analysis" ON)
    option(dlt_explorer_ENABLE_PCH "Enable precompiled headers" OFF)
    option(dlt_explorer_ENABLE_CACHE "Enable ccache" ON)
  endif()

  if(NOT PROJECT_IS_TOP_LEVEL)
    mark_as_advanced(
      dlt_explorer_ENABLE_IPO
      dlt_explorer_WARNINGS_AS_ERRORS
      dlt_explorer_ENABLE_USER_LINKER
      dlt_explorer_ENABLE_SANITIZER_ADDRESS
      dlt_explorer_ENABLE_SANITIZER_LEAK
      dlt_explorer_ENABLE_SANITIZER_UNDEFINED
      dlt_explorer_ENABLE_SANITIZER_THREAD
      dlt_explorer_ENABLE_SANITIZER_MEMORY
      dlt_explorer_ENABLE_UNITY_BUILD
      dlt_explorer_ENABLE_CLANG_TIDY
      dlt_explorer_ENABLE_CPPCHECK
      dlt_explorer_ENABLE_COVERAGE
      dlt_explorer_ENABLE_PCH
      dlt_explorer_ENABLE_CACHE)
  endif()

  dlt_explorer_check_libfuzzer_support(LIBFUZZER_SUPPORTED)
  if(LIBFUZZER_SUPPORTED AND (dlt_explorer_ENABLE_SANITIZER_ADDRESS OR dlt_explorer_ENABLE_SANITIZER_THREAD OR dlt_explorer_ENABLE_SANITIZER_UNDEFINED))
    set(DEFAULT_FUZZER ON)
  else()
    set(DEFAULT_FUZZER OFF)
  endif()

  option(dlt_explorer_BUILD_FUZZ_TESTS "Enable fuzz testing executable" ${DEFAULT_FUZZER})

endmacro()

macro(dlt_explorer_global_options)
  if(dlt_explorer_ENABLE_IPO)
    include(cmake/InterproceduralOptimization.cmake)
    dlt_explorer_enable_ipo()
  endif()

  dlt_explorer_supports_sanitizers()

  if(dlt_explorer_ENABLE_HARDENING AND dlt_explorer_ENABLE_GLOBAL_HARDENING)
    include(cmake/Hardening.cmake)
    if(NOT SUPPORTS_UBSAN 
       OR dlt_explorer_ENABLE_SANITIZER_UNDEFINED
       OR dlt_explorer_ENABLE_SANITIZER_ADDRESS
       OR dlt_explorer_ENABLE_SANITIZER_THREAD
       OR dlt_explorer_ENABLE_SANITIZER_LEAK)
      set(ENABLE_UBSAN_MINIMAL_RUNTIME FALSE)
    else()
      set(ENABLE_UBSAN_MINIMAL_RUNTIME TRUE)
    endif()
    message("${dlt_explorer_ENABLE_HARDENING} ${ENABLE_UBSAN_MINIMAL_RUNTIME} ${dlt_explorer_ENABLE_SANITIZER_UNDEFINED}")
    dlt_explorer_enable_hardening(dlt_explorer_options ON ${ENABLE_UBSAN_MINIMAL_RUNTIME})
  endif()
endmacro()

macro(dlt_explorer_local_options)
  if(PROJECT_IS_TOP_LEVEL)
    include(cmake/StandardProjectSettings.cmake)
  endif()

  add_library(dlt_explorer_warnings INTERFACE)
  add_library(dlt_explorer_options INTERFACE)

  include(cmake/CompilerWarnings.cmake)
  dlt_explorer_set_project_warnings(
    dlt_explorer_warnings
    ${dlt_explorer_WARNINGS_AS_ERRORS}
    ""
    ""
    ""
    "")

  if(dlt_explorer_ENABLE_USER_LINKER)
    include(cmake/Linker.cmake)
    dlt_explorer_configure_linker(dlt_explorer_options)
  endif()

  include(cmake/Sanitizers.cmake)
  dlt_explorer_enable_sanitizers(
    dlt_explorer_options
    ${dlt_explorer_ENABLE_SANITIZER_ADDRESS}
    ${dlt_explorer_ENABLE_SANITIZER_LEAK}
    ${dlt_explorer_ENABLE_SANITIZER_UNDEFINED}
    ${dlt_explorer_ENABLE_SANITIZER_THREAD}
    ${dlt_explorer_ENABLE_SANITIZER_MEMORY})

  set_target_properties(dlt_explorer_options PROPERTIES UNITY_BUILD ${dlt_explorer_ENABLE_UNITY_BUILD})

  if(dlt_explorer_ENABLE_PCH)
    target_precompile_headers(
      dlt_explorer_options
      INTERFACE
      <vector>
      <string>
      <utility>)
  endif()

  if(dlt_explorer_ENABLE_CACHE)
    include(cmake/Cache.cmake)
    dlt_explorer_enable_cache()
  endif()

  include(cmake/StaticAnalyzers.cmake)
  if(dlt_explorer_ENABLE_CLANG_TIDY)
    dlt_explorer_enable_clang_tidy(dlt_explorer_options ${dlt_explorer_WARNINGS_AS_ERRORS})
  endif()

  if(dlt_explorer_ENABLE_CPPCHECK)
    dlt_explorer_enable_cppcheck(${dlt_explorer_WARNINGS_AS_ERRORS} "" # override cppcheck options
    )
  endif()

  if(dlt_explorer_ENABLE_COVERAGE)
    include(cmake/Tests.cmake)
    dlt_explorer_enable_coverage(dlt_explorer_options)
  endif()

  if(dlt_explorer_WARNINGS_AS_ERRORS)
    check_cxx_compiler_flag("-Wl,--fatal-warnings" LINKER_FATAL_WARNINGS)
    if(LINKER_FATAL_WARNINGS)
      # This is not working consistently, so disabling for now
      # target_link_options(dlt_explorer_options INTERFACE -Wl,--fatal-warnings)
    endif()
  endif()

  if(dlt_explorer_ENABLE_HARDENING AND NOT dlt_explorer_ENABLE_GLOBAL_HARDENING)
    include(cmake/Hardening.cmake)
    if(NOT SUPPORTS_UBSAN 
       OR dlt_explorer_ENABLE_SANITIZER_UNDEFINED
       OR dlt_explorer_ENABLE_SANITIZER_ADDRESS
       OR dlt_explorer_ENABLE_SANITIZER_THREAD
       OR dlt_explorer_ENABLE_SANITIZER_LEAK)
      set(ENABLE_UBSAN_MINIMAL_RUNTIME FALSE)
    else()
      set(ENABLE_UBSAN_MINIMAL_RUNTIME TRUE)
    endif()
    dlt_explorer_enable_hardening(dlt_explorer_options OFF ${ENABLE_UBSAN_MINIMAL_RUNTIME})
  endif()

endmacro()
