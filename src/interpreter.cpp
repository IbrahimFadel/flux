#include <iostream>
#include <map>
#include "interpreter.h"
#include "parser.h"

using std::cout;
using std::endl;
using std::string;
using std::vector;

using Parser::Node_Types;

std::map<string, Interpreter::Variable> variables;

void _print(Node node)
{
  for (int i = 0; i < node.parameters.size(); i++)
  {
    if (node.parameters[i].string_value.length() > 0)
    {
      cout << node.parameters[i].string_value.substr(1, node.parameters[i].string_value.length() - 2) << ' ';
    }
    else if (node.parameters[i].number_value != -9999)
    {
      cout << node.parameters[i].number_value << ' ';
    }
    else
    {
      std::map<string, Interpreter::Variable>::iterator it = variables.find(node.parameters[i].id_name);
      if (it->second.string_value.length() > 0)
      {
        cout << it->second.string_value << ' ';
      }
      else
      {
        cout << it->second.number_value << ' ';
      }
    }
  }
  cout << endl;
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

void Interpreter::let(Node node)
{
  Variable var;

  if (node.variable_value_string.value.substr(0, 1) == "\"")
  {
    var.string_value = node.variable_value_string.value.substr(1, node.variable_value_string.value.length() - 2);
  }
  else
  {
    var.number_value = node.variable_value_number.value;
  }

  variables.insert({node.variable_name, var});
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
  case Node_Types::let:
    Interpreter::let(node);
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

  // std::map<string, Interpreter::Variable>::iterator it = variables.find("i");
  // cout << it->first << " = " << it->second.string_value << endl;
  // it = variables.find("x");
  // cout << it->first << " = " << it->second.number_value << endl;
}