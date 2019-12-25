#include <string>
#include <sstream>
#include <fstream>
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
  vector<Token> tokens;
};

struct Node
{
  int type;
  Parser::Condition condition;
  Parser::Then then;
  Token parameter;
  Parser::Node *parent;
  int skip = 0;
};

struct Tree
{
  vector<Parser::Node> nodes;
};
} // namespace Parser

// std::ostream &operator<<(std::ostream &os, const Parser::Node &node)
// {
//   return os << "Type: " << node.type << std::endl;
// }

Parser::Tree generate_ast(vector<Token> tokens);
Parser::Node check_token(vector<Token> tokens, int i, Parser::Node *parent);