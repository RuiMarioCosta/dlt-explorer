find_package(Boost REQUIRED)

add_library(dlt_parser src/dlt_parser.cpp)
target_include_directories(
  dlt_parser
  PUBLIC include
  PRIVATE ${Boost_INCLUDE_DIRS})

target_link_libraries(dlt_parser PRIVATE adapter buffer)

if(ENABLE_DLT_DAEMON)
  target_link_libraries(dlt_parser PUBLIC dlt)
endif()
