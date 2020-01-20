#ifndef INTERPRETER_H
#define INTERPRETER_H

#include <map>
#include "parser.h"

using namespace Parser;

namespace Interpreter
{

struct Variable
{
  int number_value;
  string string_value;
};

struct Function
{
  vector<Node> parameters;
  Then then;
  std::map<std::string, Interpreter::Variable> variables;
};

void _while(Node node, Node &parent);
void _if(Node node, Node &parent);
void let(Node node, Node &parent);
void assign(Node node, Node &parent);
void _continue(std::vector<Node> nodes, int i, Node &parent);
void _break(std::vector<Node> nodes, int i, Node &parent);
void else_if(vector<Node> nodes, int i, Node &parent);
void _else(vector<Node> nodes, int i, Node &parent);
std::string _input(Node node);
void function(vector<Node> nodes, int i);
void call_function(vector<Node> nodes, int i);
} // namespace Interpreter

void interpret(vector<Node> nodes, int i, Node &parent);
void run(Tree ast);

#endif