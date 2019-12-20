#include <iostream>
#include "lexer.h"

using namespace Lexer;

using std::vector;

namespace Parser
{

struct Position
{
  int line_number;
  int line_position;
};

enum Node_Types
{
  number = 0,
  _while = 1
};

struct Condition
{
  Token left;
  Token op;
  Token right;
};

struct Then
{
  Position start;
  Position end;
  vector<Token> tokens;
};

struct Node
{
  int type;
  Parser::Condition condition;
  Parser::Then then;
};

struct Tree
{
  vector<Parser::Node> ast;
};
} // namespace Parser

void generate_ast(vector<Token> tokens);