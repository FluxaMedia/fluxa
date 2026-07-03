cmake_minimum_required(VERSION 3.22.1)

project(fluxa_libass_renderer)

add_library(fluxa_libass_renderer SHARED
    fluxa_libass_renderer.cpp
)

find_library(log-lib log)
find_library(android-lib android)
find_library(jnigraphics-lib jnigraphics)
find_library(dl-lib dl)

target_link_libraries(fluxa_libass_renderer
    ${android-lib}
    ${jnigraphics-lib}
    ${log-lib}
    ${dl-lib}
)
