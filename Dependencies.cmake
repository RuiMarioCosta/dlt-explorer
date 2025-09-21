include(cmake/CPM.cmake)

# Done as a function so that updates to variables like
# CMAKE_CXX_FLAGS don't propagate out to other
# targets
function(dlt_explorer_setup_dependencies)

  # For each dependency, see if it's
  # already been provided to us by a parent project

  if(NOT TARGET fmtlib::fmtlib)
    cpmaddpackage("gh:fmtlib/fmt#11.1.4")
  endif()

  # if(NOT TARGET spdlog::spdlog)
  #   cpmaddpackage(
  #     NAME
  #     spdlog
  #     VERSION
  #     1.15.2
  #     GITHUB_REPOSITORY
  #     "gabime/spdlog"
  #     OPTIONS
  #     "SPDLOG_FMT_EXTERNAL ON")
  # endif()

  if(NOT TARGET Catch2::Catch2WithMain)
    cpmaddpackage("gh:catchorg/Catch2@3.10.0")
  endif()

  if(NOT TARGET CLI11::CLI11)
    cpmaddpackage("gh:CLIUtils/CLI11@2.5.0")
  endif()

  if(NOT TARGET Boost::assert)
    cpmaddpackage("gh:boostorg/assert#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::config)
    cpmaddpackage("gh:boostorg/config#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::container)
    cpmaddpackage("gh:boostorg/container#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::intrusive)
    cpmaddpackage("gh:boostorg/intrusive#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::move)
    cpmaddpackage("gh:boostorg/move#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::winapi)
    cpmaddpackage("gh:boostorg/winapi#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::predef)
    cpmaddpackage("gh:boostorg/predef#boost-1.89.0")
  endif()

  if(NOT TARGET Boost::interprocess)
    cpmaddpackage("gh:boostorg/interprocess#boost-1.89.0")
  endif()

  if(dlt_explorer_ENABLE_DLT_DAEMON)
    cpmaddpackage("gh:COVESA/dlt-daemon@2.18.10")
  endif()

endfunction()
