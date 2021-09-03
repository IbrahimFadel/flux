set(PI_COMPILER_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

set(PI_COMPILER_SRC_FILES
	${PI_COMPILER_SRC_DIR}/ir/ir.cpp

	${PI_COMPILER_SRC_DIR}/piir/piir.cpp

	${PI_COMPILER_SRC_DIR}/parser/dump.cpp
	${PI_COMPILER_SRC_DIR}/parser/parser.cpp

	${PI_COMPILER_SRC_DIR}/scanner/scanner.cpp
	
	${PI_COMPILER_SRC_DIR}/token/position.cpp
	${PI_COMPILER_SRC_DIR}/token/token.cpp
)

find_package(LLVM REQUIRED CONFIG)
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")
separate_arguments(LLVM_DEFINITIONS_LIST NATIVE_COMMAND ${LLVM_DEFINITIONS})
include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

add_subdirectory(ir/passes/testpass)

add_library(compiler ${PI_COMPILER_SRC_FILES})
target_include_directories (compiler PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

# Find the libraries that correspond to the LLVM components
# that we wish to use
llvm_map_components_to_libnames(llvm_libs support core irreader)

target_link_libraries(compiler ${llvm_libs})