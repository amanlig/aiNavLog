# Android must resolve the Rust dependency by SONAME, never a worker path.
execute_process(COMMAND "${READELF}" -d "${LIBRARY}"
  RESULT_VARIABLE result OUTPUT_VARIABLE dynamic ERROR_VARIABLE diagnostic)
if(NOT result EQUAL 0)
  message(FATAL_ERROR "Cannot inspect JNI dependencies: ${diagnostic}")
endif()
if(NOT dynamic MATCHES "\\(NEEDED\\)[^\n]*\\[libainavlog_native_bridge\\.so\\]")
  message(FATAL_ERROR "JNI library must depend on libainavlog_native_bridge.so by name: ${dynamic}")
endif()
if(dynamic MATCHES "\\(NEEDED\\)[^\n]*\\[[^]]*/[^]]*\\]")
  message(FATAL_ERROR "JNI library contains a build-machine dependency path: ${dynamic}")
endif()
message(STATUS "Verified Android JNI resolves Rust by library name")
