#ifndef PARSER_H
#define PARSER_H

#include <vector>
#include "lexer.h"

using namespace Lexer;

using std::string;
using std::vector;

namespace Parser
{

class OperatorNode
{
public:
  string value;
};

class NumberNode
{
public:
  int value;
};

class Condition
{
public:
  NumberNode left;
  OperatorNode operator_node;
  NumberNode right;
};
class Then
{
public:
  vector<Token> tokens;
};
class WhileNode
{
public:
  Condition condition;
  Then then;
};
} // namespace Parser

void check_tokens(vector<Token> tokens, int i);
void generate_ast(vector<Token> tokens);

#endif