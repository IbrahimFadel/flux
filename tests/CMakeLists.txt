set(PI_TESTS_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR})

set(PI_TESTS_SRC_FILES
	${PI_TESTS_SRC_DIR}/compiler/parser/parser_test.cpp

	${PI_TESTS_SRC_DIR}/compiler/scanner/scanner_test.cpp

	${PI_TESTS_SRC_DIR}/compiler/token/position_test.cpp
)

include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/609281088cfefc76f9d0ce82e1ff6c30cc3591e5.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)


add_executable(pi_test ${PI_TESTS_SRC_FILES})
target_link_libraries(pi_test gtest_main compiler)

include(GoogleTest)
gtest_discover_tests(pi_test)