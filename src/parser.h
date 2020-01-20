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
  number,
  _while,
  _if,
  print,
  _string,
  op,
  sep,
  eol,
  function_call,
  lit,
  let,
  id,
  assign,
  _continue,
  _break,
  _else,
  else_if,
  _for,
  function
};

struct Number
{
  int value;
};

struct String
{
  string value;
};

struct Condition
{
  vector<Token> lefts;
  vector<Token> ops;
  vector<Token> rights;
  vector<Token> results;
  vector<Token> results_operators;
  vector<Token> condition_seperators;
};

struct Then
{
  Position start;
  Position end;
  vector<Parser::Node> nodes;
  vector<Token> tokens;
};

struct Action
{
  vector<Parser::Node> nodes;
  vector<Token> tokens;
};

struct For
{
  string variable_name;
  Parser::Number variable_value_number;
  Parser::Condition condition;
  Parser::Action action;
  Parser::Then then;
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
  vector<Parser::Node> assignment_values;
  bool should_continue = false;
  int line_number;
  int line_position;
  bool should_break = false;
  string function_name;
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
    os << "CONDITION(s): " << std::endl;
    for (int i = 0; i < node.condition.lefts.size(); i++)
    {
      os << node.condition.lefts[i].value << ' ' << node.condition.ops[i].value << ' ' << node.condition.rights[i].value << std::endl;
    }
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
    os << "CONDITION(s): " << std::endl;
    for (int i = 0; i < node.condition.lefts.size(); i++)
    {
      os << node.condition.lefts[i].value << ' ' << node.condition.ops[i].value << ' ' << node.condition.rights[i].value << std::endl;
    }
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
  else if (node.type == Parser::Node_Types::assign)
  {
    os << "ASSIGN: " << node.id_name;
  }
  else if (node.type == Parser::Node_Types::_continue)
  {
    os << "CONTINUE" << std::endl;
  }
  else if (node.type == Parser::Node_Types::_break)
  {
    os << "BREAK" << std::endl;
  }
  else if (node.type == Parser::Node_Types::_else)
  {
    os << "ELSE: " << std::endl;
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      os << node.then.nodes[i];
    }
  }
  else if (node.type == Parser::Node_Types::else_if)
  {
    os << "ELSE IF: " << std::endl;
    os << "CONDITION(s): " << std::endl;
    for (int i = 0; i < node.condition.lefts.size(); i++)
    {
      os << node.condition.lefts[i].value << ' ' << node.condition.ops[i].value << ' ' << node.condition.rights[i].value << std::endl;
    }
    os << "THEN: " << std::endl;
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      os << node.then.nodes[i];
    }
  }
  else if (node.type == Parser::Node_Types::function)
  {
    os << "FUNCTION: " << node.function_name << std::endl << "THEN: " << std::endl;
    for(int i = 0; i < node.then.nodes.size(); i++)
    {
      os << node.then.nodes[i] << std::endl;
    }
  }

  return os;
}

bool is_number(const std::string &s);
Parser::Tree generate_ast(vector<Token> tokens);
Parser::Node check_token(vector<Token> tokens, int i, Parser::Node *parent);

#endif