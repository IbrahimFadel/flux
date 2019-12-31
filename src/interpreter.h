#ifndef INTERPRETER_H
#define INTERPRETER_H

#include "parser.h"

using namespace Parser;

namespace Interpreter
{

struct Variable
{
  int number_value;
  string string_value;
};

void _while(Node node);
void _if(Node node);
void let(Node node);
void assign(Node node);
} // namespace Interpreter

// inline bool operator<(const Variable &var) const
// {
// }

void interpret(Node node);
void run(Tree ast);

#endif