macro(dlt_explorer_enable_ipo)
  include(CheckIPOSupported)
  check_ipo_supported(RESULT result OUTPUT output)
  if(result)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION ON)

    # Force build of libs as shared when LTO is enabled. Since static libs
    # include the object files of its dependencies this means that all the
    # dependencies need to be compiled with LTO as well which cannot be
    # guaranteed for external dependencies
    option(BUILD_SHARED_LIBS "Build using shared libraries" ON)
  else()
    message(SEND_ERROR "IPO is not supported: ${output}")
  endif()
endmacro()
