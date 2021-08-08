set(PI_EXEC_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

set(PI_EXEC_SRC_FILES
	${PI_EXEC_SRC_DIR}/pi.cpp
)

add_executable(pi ${PI_EXEC_SRC_FILES})
target_link_libraries(pi LINK_PUBLIC compiler)
