#include <iostream>
#include "lexer.h"

using namespace Lexer;

using std::vector;

namespace Parser
{

struct Node;
struct Then;

struct Position
{
  int line_number;
  int line_position;
};

enum Node_Types
{
  number = 0,
  _while = 1,
  _if = 2,
  print = 3
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
  vector<Parser::Node> nodes;
  // vector<Token> tokens;
};

struct Node
{
  int type;
  Parser::Condition condition;
  Parser::Then then;
  Token parameter;
};

struct Tree
{
  vector<Parser::Node> nodes;
};
} // namespace Parser

void generate_ast(vector<Token> tokens);
Parser::Node check_token(vector<Token> tokens, int i);