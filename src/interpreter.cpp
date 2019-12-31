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

bool condition_true(Token left, Token op, Token right)
{
  string left_string;
  string right_string;
  int left_number;
  int right_number;

  if (left.value.substr(0, 1) == "\"" && left.type == Types::lit)
  {
    left_string = left.value.substr(1, left.value.length() - 2);
  }
  else if (!is_number(left.value))
  {
    variables_it = variables.find(left.value);
    left_number = variables_it->second.number_value;
  }
  else
  {
    left_number = std::stoi(left.value);
  }

  if (right.value.substr(0, 1) == "\"" && right.type == Types::lit)
  {
    right_string = right.value.substr(1, right.value.length() - 2);
  }
  else if (!is_number(right.value))
  {
    variables_it = variables.find(right.value);
    //! assumes it's a number
    right_number = variables_it->second.number_value;
  }
  else
  {
    right_number = std::stoi(right.value);
  }

  if (op.value == ">")
  {
    if (left_number > right_number)
    {
      return true;
    }
  }
  else if (op.value == "<")
  {
    if (left_number < right_number)
    {
      return true;
    }
  }
  else if (op.value == "==")
  {
    if (left_string.length() > 0)
    {
      if (left_string == right_string)
      {
        return true;
      }
    }
    else if (left_number == right_number)
    {
      return true;
    }
  }

  return false;
}

void Interpreter::_if(Node node)
{

  if (condition_true(node.condition.left, node.condition.op, node.condition.right))
  {
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      interpret(node.then.nodes[i]);
    }
  }
}

void Interpreter::_while(Node node)
{
  while (condition_true(node.condition.left, node.condition.op, node.condition.right))
  {
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      interpret(node.then.nodes[i]);
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