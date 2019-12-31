#ifndef PARSER_H
#define PARSER_H

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
  int line_number = -1;
  int line_position = -1;
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
  lit = 9,
  let = 10,
  id = 11
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

struct Number
{
  int value;
};

struct String
{
  string value;
};

struct Node
{
  int type;
  Parser::Condition condition;
  Parser::Then then;
  string print_value;
  Parser::Node *parent;
  int skip = 0;
  int number_value = -9999;
  string string_value;
  string op;
  string sep;
  vector<Parser::Node> parameters;
  string function_call_name;
  string variable_name;
  Parser::Number variable_value_number;
  Parser::String variable_value_string;
  string id_name;
};

struct Tree
{
  vector<Parser::Node> nodes;
};
} // namespace Parser

inline std::ostream &operator<<(std::ostream &os, const Parser::Node &node)
{

  if (node.type == Parser::Node_Types::eol)
  {
    os << "EOL";
  }
  else if (node.type == Parser::Node_Types::function_call)
  {
    os << "FUNCTION_CALL: " << node.function_call_name << std::endl;
    os << "-- PARAMETERS --" << std::endl;
    for (int i = 0; i < node.parameters.size(); i++)
    {
      os << node.parameters[i] << std::endl;
    }
    os << "-- END PARAMETERS --" << std::endl;
  }
  else if (node.type == Parser::Node_Types::lit)
  {
    os << "LITERAL: ";
    if (node.string_value.length() > 0)
    {
      os << node.string_value << std::endl;
    }
    else
    {
      os << node.number_value << std::endl;
    }
  }
  else if (node.type == Parser::Node_Types::op)
  {
    os << "OPERATOR: " << node.op << std::endl;
  }
  else if (node.type == Parser::Node_Types::sep)
  {
    os << "SEPERATOR: " << node.sep << std::endl;
  }
  else if (node.type == Parser::Node_Types::_if)
  {
    os << "IF STATEMENT: " << std::endl;
    os << "CONDITION: " << node.condition.left.value << ' ' << node.condition.op.value << ' ' << node.condition.right.value << std::endl;
    os << "THEN: " << std::endl;
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      os << node.then.nodes[i] << std::endl;
    }
    os << "-- END THEN --" << std::endl;
  }
  else if (node.type == Parser::Node_Types::_while)
  {
    os << "WHILE LOOP: " << std::endl;
    os << "CONDITION: " << node.condition.left.value << ' ' << node.condition.op.value << ' ' << node.condition.right.value << std::endl;
    os << "THEN: " << std::endl;
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      os << node.then.nodes[i] << std::endl;
    }
    os << "-- END THEN --" << std::endl;
  }
  else if (node.type == Parser::Node_Types::let)
  {
    os << "LET: " << node.variable_name << " = ";
    if (node.variable_value_string.value.substr(0, 1) == "\"")
    {
      os << node.variable_value_string.value << std::endl;
    }
    else
    {
      os << node.variable_value_number.value << std::endl;
    }
  }
  else if (node.type == Parser::Node_Types::id)
  {
    os << "IDENTIFIER: " << node.id_name << std::endl;
  }

  return os;
}

bool is_number(const std::string &s);
Parser::Tree generate_ast(vector<Token> tokens);
Parser::Node check_token(vector<Token> tokens, int i, Parser::Node *parent);

#endif