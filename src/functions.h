#ifndef FUNCTIONS_H
#define FUNCTIONS_H

#include <iostream>
#include "parser.h"

using namespace Parser;

namespace Print
{
void print(Node node, Node &parent);

void print_variable(Node node, Node &parent, int i);
void print_function_variable(Node node, Node &parent, int i);
void print_if_variable(Node node, Node &parent);
void print_global_variable(std::string variable_name);

void print_number(int number);
void print_string(std::string string);
void print_undefined_variable(std::string variable_name);
} // namespace Print

#endif