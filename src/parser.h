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
  print = 3,
  _string = 4,
  op = 5,
  sep = 6,
  eol = 7,
  function_call = 8,
  lit = 9
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
  string print_value;
  Parser::Node *parent;
  int skip = 0;
  int number_value;
  string string_value;
  string op;
  string sep;
  vector<Parser::Node> parameters;
  string function_call_name;
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
     << "Condition: " << node.condition.left.value << ' ' << node.condition.op.value << ' ' << node.condition.right.value << std::endl;

  os << "Number value: " << node.number_value << std::endl;
  os << "String value: " << node.string_value << std::endl;
  os << "Operator value: " << node.op << std::endl;
  os << "Seperator value: " << node.sep << std::endl;

  if (node.parameters.size() > 0)
  {
    os << "Parameters: " << std::endl;
    for (int i = 0; i < node.parameters.size(); i++)
    {
      os << node.parameters[i] << std::endl;
    }
  }
  // os << node.parameters->size() << std::endl;
  // if (node.parameters)
  // os << "Print value: " << node.print_value << std::endl;
  // if (node.parameters->size())
  // {
  // os << "Hi" << std::endl;
  // }
  // {
  //   os << "Parameters: " << std::endl;

  //   // for(int *param = node.parameters; param != )
  //   for (int i = 0; i < node.parameters->size(); i++)
  //   {
  //     // os << " " << &node.parameters[i] << std::endl;
  //   }
  // }

  if (node.then.nodes.size() > 0)
  {
    os << "Then: " << std::endl;
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      os << "  " << node.then.nodes[i] << std::endl;
    }
  }
  return os;
}

bool is_number(const std::string &s);
Parser::Tree generate_ast(vector<Token> tokens);
Parser::Node check_token(vector<Token> tokens, int i, Parser::Node *parent);