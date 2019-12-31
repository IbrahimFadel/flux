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
std::map<string, Interpreter::Variable>::iterator variables_it;

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
      variables_it = variables.find(node.parameters[i].id_name);
      if (variables_it->second.string_value.length() > 0)
      {
        cout << variables_it->second.string_value << ' ';
      }
      else
      {
        cout << variables_it->second.number_value << ' ';
      }
    }
  }
  cout << endl;
};

void Interpreter::_if(Node node)
{
  string left_string;
  string right_string;
  int left;
  int right;

  if (node.condition.left.value.substr(0, 1) == "\"" && node.condition.left.type == Types::lit)
  {
    left_string = node.condition.left.value.substr(1, node.condition.left.value.length() - 2);
  }
  else if (!is_number(node.condition.left.value))
  {
    variables_it = variables.find(node.condition.left.value);
    left = variables_it->second.number_value;
  }
  else
  {
    left = std::stoi(node.condition.left.value);
  }

  if (node.condition.right.value.substr(0, 1) == "\"" && node.condition.right.type == Types::lit)
  {
    right_string = node.condition.right.value.substr(1, node.condition.right.value.length() - 2);
  }
  else if (!is_number(node.condition.right.value))
  {
    variables_it = variables.find(node.condition.right.value);
    right = variables_it->second.number_value;
  }
  else
  {
    right = std::stoi(node.condition.right.value);
  }

  string op = node.condition.op.value;

  if (op == ">")
  {
    if (left > right)
    {
      for (int i = 0; i < node.then.nodes.size(); i++)
      {
        interpret(node.then.nodes[i]);
      }
    }
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
  else if (op == "==")
  {
    if (left_string.length() > 0)
    {
      if (left_string == right_string)
      {
        for (int i = 0; i < node.then.nodes.size(); i++)
        {
          interpret(node.then.nodes[i]);
        }
      }
    }
    else if (left == right)
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