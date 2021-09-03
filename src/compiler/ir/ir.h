#ifndef IR_H
#define IR_H

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Value.h>
#include <parser/ast.h>

#include <iostream>
#include <memory>

using std::unique_ptr;

namespace Codegen {

llvm::Module *generateLLVMModule(const unique_ptr<Parser::AST> &ast);
std::string LLVMModuleToString(llvm::Module *mod);

llvm::Value *typeDecl();
llvm::Function *fnDecl();

}  // namespace Codegen

#endif