#include <iostream>
#include "interpreter.h"
#include "parser.h"

using std::cout;
using std::endl;
using std::string;
using std::vector;

using Parser::Node_Types;

void _print(Node node)
{
  for (int i = 0; i < node.parameters.size(); i++)
  {
    if (node.parameters[i].string_value.length() > 0)
    {
      cout << node.parameters[i].string_value << endl;
    }
    else
    {
      cout << node.parameters[i].number_value << endl;
    }

    // cout << node.parameters[i].string_value
  }
};

void Interpreter::_if(Node node)
{
  int left = std::stoi(node.condition.left.value);
  int right = std::stoi(node.condition.right.value);
  string op = node.condition.op.value;

  if (op == ">")
  {
  }
  else if (op == "<")
  {
    if (left < right)
    {
      for (int i = 0; i < node.then.nodes.size(); i++)
      {
        interpret(node.then.nodes[i]);
      }
    }
  }
}

void Interpreter::_while(Node node)
{
  int left = std::stoi(node.condition.left.value);
  int right = std::stoi(node.condition.right.value);
  string op = node.condition.op.value;

  if (op == ">")
  {
  }
  else if (op == "<")
  {
    if (left < right)
    {
      for (int i = 0; i < node.then.nodes.size(); i++)
      {
        interpret(node.then.nodes[i]);
      }
    }
  }
};

void interpret(Node node)
{
  switch (node.type)
  {
  case Node_Types::function_call:
    if (node.function_call_name == "print")
    {
      _print(node);
    }
    break;
  case Node_Types::_while:
    Interpreter::_while(node);
    break;
  case Node_Types::_if:
    Interpreter::_if(node);
    break;
  default:
    break;
  }
}
void run(Tree ast)
{
  for (int i = 0; i < ast.nodes.size(); i++)
  {
    interpret(ast.nodes[i]);
  }
}