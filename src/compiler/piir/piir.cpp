#include "piir.h"

using namespace PIIR;

unique_ptr<IRModule> PIIR::astToIRModule(const unique_ptr<Parser::AST> &ast) {
  auto mod = std::make_unique<IRModule>();

  for (auto const &fn : ast->functions) {
    handleFnDecl(fn, mod);
  }

  return nullptr;
}

void PIIR::handleFnDecl(const unique_ptr<Parser::FnDecl> &fn, const unique_ptr<IRModule> &mod) {
  for (auto const &stmt : fn->getBody()->getList()) {
    auto varDecl = dynamic_cast<Parser::VarDecl *>(stmt.get());
    if (varDecl == nullptr) continue;
  }
}