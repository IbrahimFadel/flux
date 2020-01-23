#ifndef FUNCTIONS_H
#define FUNCTIONS_H

#include <iostream>
#include "parser.h"
#include "variables.h"

// using namespace Parser;

namespace Print
{
void print(Parser::Node node, Parser::Node &parent);

void print_variable(Parser::Node node, Parser::Node &parent, int i);
void print_function_variable(Parser::Node node, Parser::Node &parent, int i);
void print_if_variable(Parser::Node node, Parser::Node &parent);
void print_global_variable(std::string variable_name);

void print_number(int number);
void print_string(std::string string);
void print_undefined_variable(std::string variable_name);
} // namespace Print

#endif