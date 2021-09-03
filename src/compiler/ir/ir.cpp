#include "ir.h"

llvm::Module *Codegen::generateLLVMModule(const unique_ptr<Parser::AST> &ast) {
  auto ctx = std::make_unique<llvm::LLVMContext>();
  auto mod = new llvm::Module("Test Module Name", *ctx);
  // auto builder = std::make_unique<llvm::IRBuilder<>>(*ctx);

  std::cout << LLVMModuleToString(mod);

  return mod;
  return nullptr;
}

std::string Codegen::LLVMModuleToString(llvm::Module *mod) {
  std::string repr;
  llvm::raw_string_ostream os(repr);
  os << *mod;
  os.flush();
  return repr;
}