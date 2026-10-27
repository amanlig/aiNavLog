cmake_minimum_required(VERSION 3.22)
project(ainavlog_jni)
set(BRIDGE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../../builds")
add_library(ainavlog_native_bridge SHARED IMPORTED)
set_target_properties(ainavlog_native_bridge PROPERTIES IMPORTED_LOCATION "${BRIDGE_ROOT}/jniLibs/${ANDROID_ABI}/libainavlog_native_bridge.so")
set_target_properties(ainavlog_native_bridge PROPERTIES IMPORTED_SONAME "libainavlog_native_bridge.so")
add_library(ainavlog_jni SHARED bridge.cpp)
target_include_directories(ainavlog_jni PRIVATE "${BRIDGE_ROOT}/include")
target_link_libraries(ainavlog_jni ainavlog_native_bridge)
target_link_options(ainavlog_jni PRIVATE "-Wl,-z,max-page-size=16384")
add_custom_command(TARGET ainavlog_jni POST_BUILD
  COMMAND "${CMAKE_COMMAND}" "-DREADELF=${CMAKE_READELF}"
    "-DLIBRARY=$<TARGET_FILE:ainavlog_jni>"
    -P "${CMAKE_CURRENT_SOURCE_DIR}/check-needed.cmake"
  VERBATIM)
