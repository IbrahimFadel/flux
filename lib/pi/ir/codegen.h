#ifndef SSC_IR_CODEGEN_H
#define SSC_IR_CODEGEN_H

#include "driver/options.h"
#include "ast/parser.h"
#include "context.h"

#include <llvm/IR/Module.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>
#include <llvm/IR/Verifier.h>
#include <llvm/Support/raw_ostream.h>

namespace ssc
{

    void codegenNodes(const Nodes &nodes, unique_ptr<CodegenContext> &codegenContext);
} // namespace ssc

#endif