#ifndef DUMP_H
#define DUMP_H

#include <parser/ast.h>
#include <stdio.h>

namespace Parser {

std::string astToString(const unique_ptr<AST> &ast);

}  // namespace Parser

#endif