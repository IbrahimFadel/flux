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
};
class WhileNode
{
public:
  Condition condition;
  Then then;
};
} // namespace Parser

void generate_ast(vector<Token> tokens);

#endif