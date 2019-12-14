#ifndef PARSER_H
#define PARSER_H

#include <vector>
#include "lexer.h"

using namespace Lexer;

using std::string;
using std::vector;

namespace Parser
{

class Position
{
public:
  int line_number;
  int line_position;
};

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
  Position end_position;
};
class WhileNode
{
public:
  Condition condition;
  Then then;
  Position start_position;
};
class PrintNode
{
public:
  string print_value;
};
} // namespace Parser

Parser::PrintNode create_print_node(vector<Token> tokens, int i);
Parser::WhileNode create_while_node(vector<Token> tokens, int i);
Parser::Condition create_condition(vector<Token> tokens, int i);
Parser::Then create_then_Node(vector<Token> tokens, int i);
void check_token(vector<Token> tokens, int i);
void generate_ast(vector<Token> tokens);

#endif