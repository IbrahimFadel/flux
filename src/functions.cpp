#include "functions.h"


std::vector<Variables::If> t_ifs;

void Print::print(Parser::Node node, Parser::Node &parent)
{
  for (int i = 0; i < node.parameters.size(); i++)
  {
    if (node.parameters[i].string_value.length() > 0)
    {
      Print::print_string(node.parameters[i].string_value);
    }
    else if (node.parameters[i].number_value != -9999)
    {
      Print::print_number(node.parameters[i].number_value);
    }
    else
    {
      Print::print_variable(node, parent, i);
    }
  }

  std::cout << std::endl;
}

void Print::print_variable(Parser::Node node, Parser::Node &parent, int i)
{
  if (parent.type == Parser::Node_Types::function_call)
  {
    Print::print_function_variable(node, parent, i);
  }
  else if (parent.type == Parser::Node_Types::_if)
  {
    Print::print_if_variable(node, parent);
  }
  else
  {
    Print::print_global_variable(node.parameters[i].id_name);
  }
}

void Print::print_global_variable(std::string variable_name)
{
  bool global_variable_exists = Variables::global_variable_exists(variable_name);
  if (!global_variable_exists)
  {
    Print::print_undefined_variable(variable_name);
    return;
  }
  Variables::Variable var = Variables::get_global_variable(variable_name);
  if (var.string_value.length() > 0)
  {
    Print::print_string(var.string_value);
  }
  else
  {
    Print::print_number(var.number_value);
  }
}

void Print::print_if_variable(Parser::Node node, Parser::Node &parent)
{
  //! This does not look at global variables, or outer scopes(functions, or if statements, etc. ) hmmmm something to think about
  for (int i = 0; i < t_ifs.size(); i++)
  {
    if (t_ifs[i].id == parent.if_id)
    {
      bool if_variable_exists = Variables::if_variable_exists(t_ifs[i], node.variable_name);
      if (if_variable_exists)
      {
        Variables::Variable var = get_if_variable(t_ifs[i], node.variable_name);
        if (var.string_value.length() > 0)
        {
          Print::print_string(var.string_value);
        }
        else
        {
          Print::print_number(var.number_value);
        }
      }
    }
  }
}

void Print::print_function_variable(Parser::Node node, Parser::Node &parent, int i)
{
  bool global_variable_exists = Variables::global_variable_exists(node.parameters[i].id_name);
  bool function_exists = Variables::function_exists(parent.function_name);
  if (function_exists)
  {
    Variables::Variable var = Variables::get_function_variable(parent.function_name, node.parameters[i].id_name);
    if (var.string_value.length() > 0)
    {
      Print::print_string(var.string_value);
    }
    else
    {
      Print::print_number(var.number_value);
    }
  }
  else if (global_variable_exists)
  {
    Variables::Variable var = Variables::get_global_variable(node.parameters[i].id_name);
    if (var.string_value.length() > 0)
    {
      Print::print_string(var.string_value);
    }
    else
    {
      Print::print_number(var.number_value);
    }
  }
  else
  {
    Print::print_undefined_variable(node.parameters[i].id_name);
  }
}

void Print::print_string(std::string string)
{
  std::cout << string.substr(1, string.length() - 2) << ' ';
}

void Print::print_number(int number)
{
  std::cout << number << ' ';
}

void Print::print_undefined_variable(std::string variable_name)
{
  std::cerr << "Cannot print undefined variable: " << variable_name << std::endl;
  return;
}