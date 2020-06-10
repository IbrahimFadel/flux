#include <iostream>
#include <vector>
#include <map>
#include <string>
#include <variant>

#include "parser.h"

void generate_llvm_ir(std::vector<Node *>);
float evaluate_float_expression(std::unique_ptr<Expression_Node>);
llvm::Type *get_llvm_variable_type(int type);
bool is_float(std::string myString);