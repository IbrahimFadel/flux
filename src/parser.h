#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>
#include <memory>
#include <variant>

#include "lexer.h"

using std::cout;
using std::endl;

enum Node_Types
{
  VariableDeclarationNode,
  ConstantDeclarationNode,
  FunctionDeclarationNode,
  FunctionCallNode,
  NumberExpressionNode,
  StringExpressionNode,
  BinaryOperationNode
};

enum Variable_Scope
{
  Global,
  Function
};

enum Expression_Types
{
  NumberExpression,
  StringExpression
};

enum Variable_Types
{
  IntType,
  StringType,
  VoidType,
  FloatType,
  ArrayType
};

struct Number_Expression_Node
{
};
struct String_Expression_Node
{
};

struct Expression_Node
{
  Expression_Types type;
  std::variant<std::unique_ptr<Number_Expression_Node>, std::unique_ptr<String_Expression_Node>> expression;
};

struct Constant_Declaration_Node
{
  std::string name;
  Variable_Types type;
  Variable_Scope scope;
  std::unique_ptr<Expression_Node> *expression;
};

struct Variable_Declaration_Node
{
  std::string name;
  Variable_Types type;
  Variable_Scope scope;
  std::unique_ptr<Expression_Node> expression;
};

struct Node
{
  Node_Types type;
  std::variant<std::shared_ptr<Constant_Declaration_Node>, std::shared_ptr<Variable_Declaration_Node>> node;
};

void parse_tokens(std::vector<std::shared_ptr<Token>>);

std::shared_ptr<Constant_Declaration_Node> create_constant_declaration_node(std::vector<std::shared_ptr<Token>>, int);
std::unique_ptr<Expression_Node> create_expression_node(std::vector<std::shared_ptr<Token>>, int);
#endif