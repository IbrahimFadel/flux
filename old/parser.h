#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>
#include "lexer.h"

using namespace Lexer;
using std::vector;

namespace Parser
{

class Factor
{
public:
  int value;
};

class Term
{
public:
  vector<Parser::Factor> factors;
  vector<Token> operator_tokens;

  int evaluate();
};

class Expression
{
public:
  Parser::Term left;
  Token operator_token;
  Parser::Term right;
};

} // namespace Parser


void generate_ast(std::vector<Lexer::Token> tokens);

#endif