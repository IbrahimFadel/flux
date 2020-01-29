#ifndef VARIABLES_H
#define VARIABLES_H

#include <iostream>
#include <map>
#include <vector>

#include "parser.h"

namespace Variables
{
struct Variable
{
  int number_value;
  std::string string_value;
};

struct Function
{
  std::vector<Parser::Node> parameters;
  Parser::Then then;
  std::map<std::string, Variables::Variable> variables;
};

struct If
{
  Parser::Condition condition;
  Parser::Then then;
  std::map<std::string, Variables::Variable> variables;
  int id;
};

extern std::map<std::string, Variables::Variable> variables;
extern std::map<std::string, Variables::Variable>::iterator variables_it;

extern std::map<std::string, Variables::Function> functions;
extern std::map<std::string, Variables::Function>::iterator functions_it;
extern std::map<std::string, Variables::Variable>::iterator function_variables_it;

extern std::vector<Variables::If> ifs;
extern std::map<std::string, Variables::Variable>::iterator ifs_variables_it;

bool global_variable_exists(std::string variable_name);
bool function_exists(std::string function_name);
bool if_variable_exists(Variables::If _if, std::string variable_name);

Variables::Variable get_global_variable(std::string variable_name);
Variables::Variable get_function_variable(std::string function_name, std::string variable_name);
Variables::Variable get_if_variable(Variables::If _if, std::string variable_name);
} // namespace Variables

#endif