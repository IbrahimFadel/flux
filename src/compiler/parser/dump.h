#ifndef DUMP_H
#define DUMP_H

#include <parser/ast.h>
#include <stdio.h>

namespace Parser {

class ASTDump {
 private:
  std::vector<unique_ptr<Node>> &ast;

 public:
  ASTDump(std::vector<unique_ptr<Node>> &ast) : ast(ast){};

  std::string toString();
};

}  // namespace Parser

#endif