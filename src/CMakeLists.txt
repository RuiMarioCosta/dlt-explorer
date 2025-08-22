add_subdirectory(adapter)
add_subdirectory(buffer)
add_subdirectory(dlt_parser)
# add_subdirectory(parse_cmd_line) # ASAN doesn't work when linking

add_executable(main main.cpp)
target_link_libraries(main
  PRIVATE
    dlt_explorer::adapter
    dlt_explorer::config
    dlt_explorer::dlt_explorer_options
    dlt_explorer::dlt_explorer_warnings
    dlt_explorer::dlt_parser
)
target_link_system_libraries(main
  PRIVATE
    CLI11::CLI11
    fmt::fmt
)
