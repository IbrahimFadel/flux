#include "variables.h"

std::map<std::string, Variables::Variable> _variables;
std::map<std::string, Variables::Variable>::iterator _variables_it;

std::map<std::string, Variables::Function> _functions;
std::map<std::string, Variables::Function>::iterator _functions_it;
std::map<std::string, Variables::Variable>::iterator _function_variables_it;

std::vector<Variables::If> _ifs;
std::map<std::string, Variables::Variable>::iterator _ifs_variables_it;

bool Variables::global_variable_exists(std::string variable_name)
{
  _variables_it = _variables.find(variable_name);
  return _variables_it != _variables.end();
}

bool Variables::function_exists(std::string function_name)
{
  _functions_it = _functions.find(function_name);
  return _functions_it != _functions.end();
}

bool Variables::if_variable_exists(Variables::If _if, std::string variable_name)
{
  _ifs_variables_it = _if.variables.find(variable_name);
  return _ifs_variables_it != _if.variables.end();
}

Variables::Variable Variables::get_global_variable(std::string variable_name)
{
  _variables_it = _variables.find(variable_name);
  return _variables_it->second;
}

Variables::Variable Variables::get_function_variable(std::string function_name, std::string variable_name)
{
  _functions_it = _functions.find(function_name);
  return _functions_it->second.variables.find(variable_name)->second;
}

Variables::Variable Variables::get_if_variable(Variables::If _if, std::string variable_name)
{
  _ifs_variables_it = _if.variables.find(variable_name);
  return _ifs_variables_it->second;
}