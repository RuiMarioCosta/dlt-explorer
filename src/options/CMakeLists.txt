find_package(Boost REQUIRED COMPONENTS program_options)

add_library(options src/options.cpp)
target_include_directories(options PUBLIC include)
target_link_libraries(options PRIVATE Boost::program_options)
