#ifndef FUNCTIONS_H
#define FUNCTIONS_H

#include <iostream>
#include <vector>
#include "lexer.h"
#include "parser.h"
#include "variables.h"

using std::cout;
using std::endl;

namespace Functions
{
  std::vector<std::string> get_fn_parameters(std::vector<Lexer::Token> tokens, int i);
  int get_fn_return_type(std::vector<Lexer::Token> tokens, int i);
  Parser::Then get_fn_then(std::vector<Lexer::Token> tokens, int i);
}

#endif