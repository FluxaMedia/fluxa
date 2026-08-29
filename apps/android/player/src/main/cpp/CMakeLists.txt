cmake_minimum_required(VERSION 3.22.1)

project(fluxa_libass_renderer)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(fluxa_libass_renderer SHARED
    discord_presence_jni.cpp
    fluxa_libass_renderer.cpp
)

find_library(log-lib log)
find_library(android-lib android)
find_library(jnigraphics-lib jnigraphics)
find_library(dl-lib dl)

find_package(discord_partner_sdk REQUIRED CONFIG)

target_link_libraries(fluxa_libass_renderer
    ${android-lib}
    ${jnigraphics-lib}
    ${log-lib}
    ${dl-lib}
    discord_partner_sdk::discord_partner_sdk
)

target_link_options(fluxa_libass_renderer PRIVATE
    "-Wl,-z,max-page-size=16384"
)
