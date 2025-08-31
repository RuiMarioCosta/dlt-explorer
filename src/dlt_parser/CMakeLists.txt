add_library(dlt_parser src/dlt_parser.cpp)
add_library(dlt_explorer::dlt_parser ALIAS dlt_parser)

include(GenerateExportHeader)
generate_export_header(dlt_parser)

target_include_directories(
  dlt_parser
  PUBLIC
    include
    ${CMAKE_CURRENT_BINARY_DIR}
)

target_link_libraries(dlt_parser
  PUBLIC
    buffer
  PRIVATE
    adapter
    dlt_explorer::dlt_explorer_options
    dlt_explorer::dlt_explorer_warnings
)

target_link_system_libraries(dlt_parser
  PUBLIC
    Boost::interprocess
)

if(ENABLE_DLT_DAEMON)
  target_link_libraries(dlt_parser PUBLIC dlt)
endif()

if(NOT BUILD_SHARED_LIBS)
  target_compile_definitions(dlt_parser PUBLIC DLT_PARSER_STATIC_DEFINE)
endif()
