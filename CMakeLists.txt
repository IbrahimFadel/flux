cmake_minimum_required(VERSION 3.8)

project(pi-lang)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

enable_testing()

add_subdirectory(src/compiler)
add_subdirectory(src/pi)
add_subdirectory(tests)
