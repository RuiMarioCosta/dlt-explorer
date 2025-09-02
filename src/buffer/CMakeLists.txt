add_library(buffer src/buffer.cpp)
add_library(dlt_explorer::buffer ALIAS buffer)

include(GenerateExportHeader)
generate_export_header(buffer)

target_include_directories(buffer
  PUBLIC
    include
    ${CMAKE_CURRENT_BINARY_DIR}
)
target_link_libraries(buffer
  PRIVATE
    dlt_explorer::dlt_explorer_options
    dlt_explorer::dlt_explorer_warnings
)

target_link_system_libraries(buffer
  PUBLIC
    fmt::fmt
)

if(NOT BUILD_SHARED_LIBS)
  target_compile_definitions(buffer PUBLIC BUFFER_STATIC_DEFINE)
endif()
