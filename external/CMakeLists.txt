add_library(imgui
  imgui/imgui.cpp
  imgui/imgui.h
  imgui/imgui_demo.cpp
  imgui/imgui_draw.cpp
  imgui/imgui_internal.h
  imgui/imgui_tables.cpp
  imgui/imgui_widgets.cpp
  imgui/imstb_rectpack.h
  imgui/imstb_textedit.h
  imgui/imstb_truetype.h
)
add_library(imgui::imgui ALIAS imgui)

target_include_directories(imgui PUBLIC imgui/)

target_link_libraries(imgui
  PRIVATE
    # dlt_explorer::dlt_explorer_options
    # dlt_explorer::dlt_explorer_warnings
)


add_library(imgui-opengl
  imgui/backends/imgui_impl_opengl3.h
  imgui/backends/imgui_impl_opengl3.cpp
  imgui/backends/imgui_impl_opengl3_loader.h
)
add_library(imgui::imgui-opengl ALIAS imgui-opengl)

target_include_directories(imgui-opengl PUBLIC imgui/backends)

target_link_libraries(imgui-opengl
  PUBLIC
    imgui::imgui
  PRIVATE
    # dlt_explorer::dlt_explorer_options
    # dlt_explorer::dlt_explorer_warnings
)


