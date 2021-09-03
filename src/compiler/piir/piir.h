#ifndef PIIR_H
#define PIIR_H

#include <parser/ast.h>

#include <memory>
#include <vector>

using std::unique_ptr;

namespace PIIR {

class IRModule {
 private:
  std::vector<unique_ptr<Parser::FnDecl>> functions;

 public:
};

unique_ptr<IRModule> astToIRModule(const unique_ptr<Parser::AST> &ast);
void handleFnDecl(const unique_ptr<Parser::FnDecl> &fn, const unique_ptr<IRModule> &mod);

}  // namespace PIIR

#endif