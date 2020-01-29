#include "variables.h"
#include <iostream>

using std::cout;
using std::endl;

bool Variables::global_variable_exists(std::string variable_name)
{
  Variables::variables_it = Variables::variables.find(variable_name);
  return Variables::variables_it != Variables::variables.end();
}

bool Variables::function_exists(std::string function_name)
{
  Variables::functions_it = Variables::functions.find(function_name);
  return Variables::functions_it != Variables::functions.end();
}

bool Variables::if_variable_exists(Variables::If _if, std::string variable_name)
{
  Variables::ifs_variables_it = _if.variables.find(variable_name);
  return Variables::ifs_variables_it != _if.variables.end();
}

Variables::Variable Variables::get_global_variable(std::string variable_name)
{
  Variables::variables_it = Variables::variables.find(variable_name);
  return Variables::variables_it->second;
}

Variables::Variable Variables::get_function_variable(std::string function_name, std::string variable_name)
{
  Variables::functions_it = Variables::functions.find(function_name);
  return Variables::functions_it->second.variables.find(variable_name)->second;
}

Variables::Variable Variables::get_if_variable(Variables::If _if, std::string variable_name)
{
  Variables::ifs_variables_it = _if.variables.find(variable_name);
  return Variables::ifs_variables_it->second;
}