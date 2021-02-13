#include "ast.h"

void Function_Declaration::set_variable(std::string name, llvm::Value *v) { variables[name] = v; }
llvm::Value *Function_Declaration::get_variable(std::string name) { return variables[name]; }
std::string Function_Declaration::get_name() { return name; };
std::map<std::string, std::string> Function_Declaration::get_params() { return params; };
std::string Function_Declaration::get_return_type() { return return_type; };

std::string Import_Statement::get_path() { return path; };

std::map<std::string, unique_ptr<Expression>> Struct_Value_Expression::get_properties() { return std::move(properties); }

std::string Variable_Reference_Expression::get_name() { return name; };

Token_Type Binary_Operation_Expression::get_op() { return op; };