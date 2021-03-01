#ifndef SSC_LINKER_LOWERING_H
#define SSC_LINKER_LOWERING_H

#include "llvm/IR/Module.h"
#include "llvm/IR/LegacyPassManager.h"

#include "llvm/Transforms/InstCombine/InstCombine.h"
#include "llvm/Transforms/Scalar.h"
#include "llvm/Transforms/Scalar/GVN.h"
#include "llvm/Transforms/Utils.h"

#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/TargetRegistry.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Support/Host.h"
#include "llvm/Support/raw_ostream.h"

#include "llvm/Target/TargetOptions.h"
#include "llvm/Target/TargetMachine.h"

#include "llvm/IR/AssemblyAnnotationWriter.h"

#include "ir/context.h"
#include <memory>
#include <string>
#include <sys/unistd.h>

using std::unique_ptr;

namespace ssc
{
    void writeModuleToObjectFile(unique_ptr<CodegenContext> &ctx, std::string outputFilePath);
    void executeCommand(const std::string cmd, int &exitCode);
} // namespace ssc

#endif