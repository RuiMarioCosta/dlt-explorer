if(dlt_explorer_ENABLE_DLT_DAEMON)
  add_library(adapter with_dlt_daemon/src/adapter.cpp)
  target_include_directories(
    adapter
    PUBLIC with_dlt_daemon/include)
  target_link_system_libraries(adapter PUBLIC dlt)
  target_compile_definitions(adapter PUBLIC $<$<BOOL:${ENABLE_DLT_DAEMON}>:ENABLE_DLT_DAEMON>)
else()
  add_library(adapter without_dlt_daemon/src/adapter.cpp)
  target_include_directories(
    adapter
    PUBLIC without_dlt_daemon/include)
endif()

add_library(dlt_explorer::adapter ALIAS adapter)

include(GenerateExportHeader)
generate_export_header(adapter)

target_include_directories(
    adapter
    PUBLIC
    ${CMAKE_CURRENT_BINARY_DIR}
)
target_link_libraries(adapter
  PRIVATE
    dlt_explorer::dlt_explorer_options
    dlt_explorer::dlt_explorer_warnings
)

if(NOT BUILD_SHARED_LIBS)
  target_compile_definitions(adapter PUBLIC ADAPTER_STATIC_DEFINE)
endif()
