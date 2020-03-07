#ifndef VARIABLES_H
#define VARIABLES_H

#include <iostream>
#include <vector>
#include <stack>
#include "lexer.h"

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
} Expression;

typedef struct Variable
{
  int type;
  std::string name;
  Variables::Expression int_value;
} Variable;

Variables::Expression evaluate_expression(std::vector<Lexer::Token> tokens, int i);
int apply_operation(int a, int b, char op);
int get_precedence(char op);
} // namespace Variables

#endif