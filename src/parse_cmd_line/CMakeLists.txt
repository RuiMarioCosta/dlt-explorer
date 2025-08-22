add_library(parse_cmd_line
  src/subcommand_gui.cpp
  src/subcommand_term.cpp
)
add_library(dlt_explorer::parse_cmd_line ALIAS parse_cmd_line)

set_target_properties(
  parse_cmd_line
  PROPERTIES
    CXX_VISIBILITY_PRESET hidden
    VISIBILITY_INLINES_HIDDEN TRUE
)

include(GenerateExportHeader)
generate_export_header(parse_cmd_line)
target_include_directories(parse_cmd_line
  PUBLIC
    include
    ${CMAKE_CURRENT_BINARY_DIR}
)
target_link_libraries(parse_cmd_line
  PRIVATE
    dlt_explorer::dlt_explorer_options
    dlt_explorer::dlt_explorer_warnings
)
target_link_system_libraries(parse_cmd_line
  PUBLIC
    CLI11::CLI11
  PRIVATE
    fmt::fmt
)

if(NOT BUILD_SHARED_LIBS)
  target_compile_definitions(parse_cmd_line PUBLIC PARSE_CMD_LINE_STATIC_DEFINE)
endif()
