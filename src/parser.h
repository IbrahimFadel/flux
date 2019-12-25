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

inline std::ostream &operator<<(std::ostream &os, const Parser::Node &node)
{
  os << std::endl
     << "Type: " << node.type << std::endl
     << "Condition: " << node.condition.left.value << ' ' << node.condition.op.value << ' ' << node.condition.right.value << std::endl
     << "Then: " << std::endl;
  for (int i = 0; i < node.then.nodes.size(); i++)
  {
    os << "  " << node.then.nodes[i] << std::endl;
  }
  return os;
}

Parser::Tree generate_ast(vector<Token> tokens);
Parser::Node check_token(vector<Token> tokens, int i, Parser::Node *parent);