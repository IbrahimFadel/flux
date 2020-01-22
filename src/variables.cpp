#include "variables.h"

using namespace Variables;

bool global_variable_exists(std::string variable_name)
{
  variables_it = variables.find(variable_name);
  return variables_it != variables.end();
}

bool function_exists(std::string function_name)
{
  functions_it = functions.find(function_name);
  return functions_it != functions.end();
}

bool if_variable_exists(If _if, std::string variable_name)
{
  ifs_variables_it = _if.variables.find(variable_name);
  return ifs_variables_it != _if.variables.end();
}

Variable get_global_variable(std::string variable_name)
{
  variables_it = variables.find(variable_name);
  return variables_it->second;
}

Variable get_function_variable(std::string function_name, std::string variable_name)
{
  functions_it = functions.find(function_name);
  return functions_it->second.variables.find(variable_name)->second;
}

Variable get_if_variable(Variables::If _if, std::string variable_name)
{
  ifs_variables_it = _if.variables.find(variable_name);
  return ifs_variables_it->second;
}