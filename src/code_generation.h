#ifndef CODE_GENERATION_H
#define CODE_GENERATION_H

#include "parser.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Verifier.h>
#include <llvm/Support/raw_ostream.h>

static llvm::LLVMContext context;
static std::unique_ptr<llvm::Module> module = std::make_unique<llvm::Module>("Module", context);
static llvm::IRBuilder<> builder(context);

void code_gen(std::vector<std::unique_ptr<Node>> nodes);
void code_gen_node(std::unique_ptr<Node> node, bool function_body = false, llvm::BasicBlock *bb = 0);
llvm::Type *ss_type_to_llvm_type(Variable_Types type);

static llvm::Value *error_v(const char *Str);

#endif