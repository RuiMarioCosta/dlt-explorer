if(dlt_explorer_ENABLE_DLT_DAEMON)
  add_library(adapter with_dlt_daemon/src/adapter.cpp)
  target_include_directories(
    adapter
    PUBLIC with_dlt_daemon/include)
  target_link_libraries(adapter PUBLIC dlt)
  target_compile_definitions(adapter PUBLIC $<$<BOOL:${ENABLE_DLT_DAEMON}>:ENABLE_DLT_DAEMON>)
else()
  add_library(adapter without_dlt_daemon/src/adapter.cpp)
  target_include_directories(
    adapter
    PUBLIC without_dlt_daemon/include)
endif()
