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

void _while(Node node, Node &parent);
void _if(Node node, Node &parent);
void let(Node node);
void assign(Node node);
void _continue(std::vector<Node> nodes, int i, Node &parent);
void _break(std::vector<Node> nodes, int i, Node &parent);
void else_if(vector<Node> nodes, int i, Node &parent);
void _else(vector<Node> nodes, int i, Node &parent);
std::string _input(Node node);
} // namespace Interpreter

void interpret(vector<Node> nodes, int i, Node &parent);
void run(Tree ast);

#endif