cmake_minimum_required(VERSION 2.8.4)
project(Sandscript)

set(CMAKE_CXX_FLAGS "-std=c++17")

set(SRC_DIR src)

set(SANDSCRIPT_SOURCE_FILES ${SRC_DIR}/common.cpp ${SRC_DIR}/lexer.cpp ${SRC_DIR}/ast.cpp ${SRC_DIR}/parser.cpp ${SRC_DIR}/code_generation.cpp)
add_library(sandscript ${SANDSCRIPT_SOURCE_FILES})
target_include_directories(sandscript PRIVATE ${SRC_DIR})
target_link_libraries(sandscript LLVM-11)

add_executable(ssc ${SRC_DIR}/main.cpp)
target_link_libraries(ssc sandscript)

set(TESTS_DIR tests)
set(TESTS_SOURCE_FILES ${TESTS_DIR}/tests.cpp)
add_executable(ss-test ${TESTS_SOURCE_FILES})
target_include_directories(ss-test PRIVATE ${SRC_DIR})
target_link_libraries(ss-test sandscript gtest)

set(INSTALL_PREFIX /usr/local/bin)

install(TARGETS ssc DESTINATION /usr/local/bin)