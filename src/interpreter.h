#ifndef INTERPRETER_H
#define INTERPRETER_H

#include "parser.h"

using namespace Parser;

namespace Interpreter
{
void _while(Node node);
void _if(Node node);
} // namespace Interpreter

void interpret(Node node);
void run(Tree ast);

#endif