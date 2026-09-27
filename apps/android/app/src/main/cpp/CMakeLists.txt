cmake_minimum_required(VERSION 3.22)
project(fluxa_mpv C)

set(LIBMPV_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../../libmpv)

add_library(mpv SHARED IMPORTED)
set_target_properties(mpv PROPERTIES IMPORTED_LOCATION ${LIBMPV_DIR}/jni/${ANDROID_ABI}/libmpv.so)

add_library(fluxa_mpv SHARED mpv.c)
target_include_directories(fluxa_mpv PRIVATE ${LIBMPV_DIR}/include)
target_link_libraries(fluxa_mpv mpv log)
