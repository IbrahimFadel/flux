set(PI_COMPILER_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

set(PI_COMPILER_SRC_FILES
	${PI_COMPILER_SRC_DIR}/parser/dump.cpp
	${PI_COMPILER_SRC_DIR}/parser/parser.cpp

	${PI_COMPILER_SRC_DIR}/scanner/scanner.cpp
	
	${PI_COMPILER_SRC_DIR}/token/position.cpp
	${PI_COMPILER_SRC_DIR}/token/token.cpp
)

add_library(compiler ${PI_COMPILER_SRC_FILES})
target_include_directories (compiler PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})