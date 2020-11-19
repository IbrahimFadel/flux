cmake_minimum_required(VERSION 2.8.4)
project(Sandscript)

set(LLVM_PATH /build/llvm/src/llvm-12.0.0.src)
link_directories(${LLVM_PATH}/lib)
include_directories(${LLVM_PATH}/include)

add_definitions(
-D__STDC_LIMIT_MACROS
-D__STDC_CONSTANT_MACROS
)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++17")

# src/code_generation.cpp
set(SOURCE_FILES src/common.cpp src/main.cpp src/lexer.cpp src/ast.cpp src/parser.cpp src/code_generation.cpp)
add_executable(Sandscript ${SOURCE_FILES})

target_link_libraries(Sandscript LLVM-11)