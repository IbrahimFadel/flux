cmake_minimum_required(VERSION 2.8.12)

project(Sandscript)

set(CMAKE_CXX_FLAGS "-std=c++17 -fno-exceptions -fno-rtti")
set(SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)

set(SANDSCRIPT_SOURCE_FILES
    ${SRC_DIR}/common.cpp
    ${SRC_DIR}/lexer.cpp
    ${SRC_DIR}/ast.cpp
    ${SRC_DIR}/parser.cpp
    ${SRC_DIR}/code_generation.cpp
    ${SRC_DIR}/main.cpp
)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)
add_executable(ssc ${SANDSCRIPT_SOURCE_FILES})
# target_link_libraries(ssc LLVMSupport LLVMExecutionEngine LLVMMIRParser LLVMIRReader LLVMCore LLVMAnalysis LLVMInstCombine LLVMObject LLVMScalarOpts LLVMRuntimeDyld)
target_link_libraries(ssc LLVMXRay LLVMWindowsManifest LLVMTableGen LLVMSymbolize LLVMDebugInfoPDB LLVMOrcJIT LLVMOrcError LLVMJITLink LLVMObjectYAML LLVMMIRParser LLVMMCA LLVMLTO LLVMPasses LLVMHelloNew LLVMCoroutines LLVMObjCARCOpts LLVMipo LLVMInstrumentation LLVMVectorize LLVMLinker LLVMIRReader LLVMAsmParser LLVMFrontendOpenMP LLVMExtensions LLVMLineEditor LLVMLibDriver LLVMInterpreter LLVMInterfaceStub LLVMFuzzMutate LLVMMCJIT LLVMExecutionEngine LLVMRuntimeDyld LLVMDWARFLinker LLVMDlltoolDriver LLVMOption LLVMDebugInfoGSYM LLVMCoverage LLVMX86Disassembler LLVMX86AsmParser LLVMX86CodeGen LLVMCFGuard LLVMGlobalISel LLVMSelectionDAG LLVMAsmPrinter LLVMDebugInfoDWARF LLVMCodeGen LLVMTarget LLVMScalarOpts LLVMInstCombine LLVMAggressiveInstCombine LLVMTransformUtils LLVMBitWriter LLVMAnalysis LLVMProfileData LLVMObject LLVMTextAPI LLVMMCParser LLVMBitReader LLVMCore LLVMRemarks LLVMBitstreamReader LLVMX86Desc LLVMMCDisassembler LLVMMC LLVMDebugInfoCodeView LLVMDebugInfoMSF LLVMBinaryFormat LLVMX86Info LLVMSupport LLVMDemangle
rt dl pthread m z tinfo xml2)

# include_directories(include)

# add_library(ssc STATIC ${SSC_CPP_SOURCES})
# set(SSC_CPP_SOURCES
    # ""
# )

# add_executable(ssc ${SSC_CPP_SOURCES})