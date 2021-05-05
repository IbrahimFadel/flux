cmake_minimum_required(VERSION 3.7...3.18)

project(Sandscript)

set(CMAKE_CXX_FLAGS "-std=c++20")

set(PI_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/lib/pi)
set(PI_SOURCE_FILES
    ${PI_SRC_DIR}/pi.cpp

    ${PI_SRC_DIR}/driver/driver.cpp
    ${PI_SRC_DIR}/driver/options.cpp

    ${PI_SRC_DIR}/ast/lexer.cpp
    ${PI_SRC_DIR}/ast/nodes.cpp
    ${PI_SRC_DIR}/ast/parser.cpp

    ${PI_SRC_DIR}/ir/context.cpp
    ${PI_SRC_DIR}/ir/codegen.cpp

    ${PI_SRC_DIR}/linker/lowering.cpp
)

add_executable(pi ${PI_SOURCE_FILES})
set_target_properties(pi PROPERTIES INTERPROCEDURAL_OPTIMIZATION TRUE)
target_include_directories(pi PRIVATE ${PI_SRC_DIR})

target_link_libraries(pi LLVMXCoreCodeGen LLVMXCoreDesc LLVMXCoreInfo LLVMX86AsmParser LLVMX86CodeGen LLVMX86Desc LLVMX86Info  LLVMWebAssemblyCodeGen LLVMWebAssemblyDesc LLVMWebAssemblyAsmParser LLVMWebAssemblyInfo LLVMSystemZCodeGen LLVMSystemZAsmParser LLVMSystemZDesc LLVMSystemZInfo LLVMSparcCodeGen LLVMSparcAsmParser LLVMSparcDesc LLVMSparcInfo LLVMRISCVCodeGen LLVMRISCVAsmParser LLVMRISCVDesc LLVMRISCVUtils LLVMRISCVInfo LLVMPowerPCCodeGen LLVMPowerPCAsmParser LLVMPowerPCDesc LLVMPowerPCInfo LLVMNVPTXCodeGen LLVMNVPTXDesc LLVMNVPTXInfo LLVMMSP430CodeGen LLVMMSP430AsmParser LLVMMSP430Desc LLVMMSP430Info     LLVMMipsCodeGen LLVMMipsAsmParser LLVMMipsDesc LLVMMipsInfo LLVMLanaiCodeGen LLVMLanaiAsmParser LLVMLanaiDesc LLVMLanaiInfo LLVMHexagonCodeGen LLVMHexagonAsmParser LLVMHexagonDesc LLVMHexagonInfo LLVMBPFCodeGen LLVMBPFAsmParser LLVMBPFDesc LLVMBPFInfo LLVMAVRCodeGen LLVMAVRAsmParser LLVMAVRDesc LLVMAVRInfo LLVMARMCodeGen LLVMARMAsmParser LLVMARMDesc LLVMARMUtils LLVMARMInfo LLVMAMDGPUCodeGen LLVMMIRParser LLVMipo LLVMInstrumentation LLVMVectorize LLVMAsmParser LLVMFrontendOpenMP LLVMAMDGPUAsmParser LLVMAMDGPUDesc LLVMAMDGPUUtils LLVMAMDGPUInfo LLVMMCDisassembler LLVMAArch64CodeGen LLVMCFGuard LLVMGlobalISel LLVMSelectionDAG LLVMAsmPrinter LLVMDebugInfoDWARF LLVMCodeGen LLVMTarget LLVMScalarOpts LLVMInstCombine LLVMAggressiveInstCombine LLVMTransformUtils LLVMBitWriter LLVMAnalysis LLVMProfileData LLVMObject LLVMTextAPI LLVMBitReader LLVMCore LLVMRemarks LLVMBitstreamReader LLVMAArch64AsmParser LLVMMCParser LLVMAArch64Desc LLVMMC LLVMDebugInfoCodeView LLVMBinaryFormat LLVMAArch64Utils LLVMAArch64Info LLVMSupport LLVMDemangle rt dl pthread m z tinfo xml2)