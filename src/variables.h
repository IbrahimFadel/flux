#ifndef VARIABLES_H
#define VARIABLES_H

#include <iostream>
#include <vector>
#include <stack>

namespace Parser
{
struct Node;
}

#include "lexer.h"
#include "parser.h"

using std::cout;
using std::endl;

namespace Variables
{

typedef enum Variable_Types
{
  _void,
  integer
} Variable_Types;

typedef struct Expression
{
  int int_value;
  int skip = 0;
  std::vector<std::string> variable_names;
} Expression;

typedef enum Variable_Scopes
{
  global,
  _if,
  fn
} Variable_Scopes;

typedef struct Variable
{
  int type;
  std::string name;
  Variables::Expression int_value;
  int scope;
  Parser::Node *parent;
} Variable;

inline std::vector<Variables::Variable *> variables;

Variables::Expression evaluate_expression(std::vector<Lexer::Token> tokens, int i);
int apply_operation(int a, int b, char op);
int get_precedence(char op);
Variables::Variable *get_variable(std::string name);
} // namespace Variables

#endif