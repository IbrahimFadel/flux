cmake_minimum_required(VERSION 2.8.12)

project(Sandscript)

set(CMAKE_CXX_FLAGS "-std=c++17")

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

set(SSC_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/lib/ssc)
set(SSC_SOURCE_FILES
    ${SSC_SRC_DIR}/ssc.cpp

    ${SSC_SRC_DIR}/driver/driver.cpp
    ${SSC_SRC_DIR}/driver/options.cpp

    ${SSC_SRC_DIR}/ast/lexer.cpp
    ${SSC_SRC_DIR}/ast/nodes.cpp
    ${SSC_SRC_DIR}/ast/parser.cpp

    ${SSC_SRC_DIR}/ir/codegen.cpp
)

add_executable(ssc ${SSC_SOURCE_FILES})
target_include_directories(ssc PRIVATE ${SSC_SRC_DIR})

target_link_libraries(ssc LLVMXRay LLVMWindowsManifest LLVMTableGen LLVMSymbolize LLVMDebugInfoPDB LLVMOrcJIT LLVMOrcError LLVMJITLink LLVMObjectYAML LLVMMIRParser LLVMMCA LLVMLTO LLVMPasses LLVMHelloNew LLVMCoroutines LLVMObjCARCOpts LLVMipo LLVMInstrumentation LLVMVectorize LLVMLinker LLVMIRReader LLVMAsmParser LLVMFrontendOpenMP LLVMExtensions LLVMLineEditor LLVMLibDriver LLVMInterpreter LLVMInterfaceStub LLVMFuzzMutate LLVMMCJIT LLVMExecutionEngine LLVMRuntimeDyld LLVMDWARFLinker LLVMDlltoolDriver LLVMOption LLVMDebugInfoGSYM LLVMCoverage LLVMX86Disassembler LLVMX86AsmParser LLVMX86CodeGen LLVMCFGuard LLVMGlobalISel LLVMSelectionDAG LLVMAsmPrinter LLVMDebugInfoDWARF LLVMCodeGen LLVMTarget LLVMScalarOpts LLVMInstCombine LLVMAggressiveInstCombine LLVMTransformUtils LLVMBitWriter LLVMAnalysis LLVMProfileData LLVMObject LLVMTextAPI LLVMMCParser LLVMBitReader LLVMCore LLVMRemarks LLVMBitstreamReader LLVMX86Desc LLVMMCDisassembler LLVMMC LLVMDebugInfoCodeView LLVMDebugInfoMSF LLVMBinaryFormat LLVMX86Info LLVMSupport LLVMDemangle
rt dl pthread m z tinfo xml2)
