add_library(buffer src/buffer.cpp)
add_library(dlt_explorer::buffer ALIAS buffer)

include(GenerateExportHeader)
generate_export_header(buffer)

target_sources(
  buffer
  PRIVATE src/buffer.cpp
  PUBLIC
    FILE_SET HEADERS
      FILES include/buffer.h
    FILE_SET generated_headers
      TYPE HEADERS
      BASE_DIRS $<TARGET_PROPERTY:buffer,BINARY_DIR>
      FILES ${CMAKE_CURRENT_BINARY_DIR}/buffer_export.h
)

target_include_directories(buffer
  PUBLIC
    include
    ${CMAKE_CURRENT_BINARY_DIR}
)
target_link_libraries(buffer
  PUBLIC
    fmt::fmt
  PRIVATE
    dlt_explorer::dlt_explorer_options
    dlt_explorer::dlt_explorer_warnings
)
