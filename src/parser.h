#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>
#include <memory>
#include <variant>
#include <map>

#include <llvm/IR/Value.h>

#include "lexer.h"

using std::cout;
using std::endl;

struct Then;
struct Node;
struct Binary_Operation_Node;

enum Node_Types
{
  VariableDeclarationNode,
  ConstantDeclarationNode,
  FunctionDeclarationNode,
  FunctionCallNode,
  NumberExpressionNode,
  StringExpressionNode,
  BinaryOperationNode,
  ReturnNode
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

enum Number_Types
{
  FloatNumber,
  IntNumber
};

struct Number
{
  Number_Types type;
  std::variant<float, int> float_number, int_number;
};

struct Term
{
  std::vector<std::string> numbers;
  std::vector<std::string> ops;
  llvm::Value *code_gen();
};

struct Number_Expression_Node
{
  int type = 0;
  std::vector<std::string> ops;
  std::vector<Term> terms;
  llvm::Value *code_gen();
};

struct String_Expression_Node
{
};

struct Expression_Node
{
  Expression_Types type;
  std::variant<Number_Expression_Node, String_Expression_Node> number_expression, string_expression;
  llvm::Value *code_gen();
};

struct Binary_Operation_Node
{
  Expression_Node *left;
  std::string op;
  Expression_Node *right;
};

struct Constant_Declaration_Node
{
  std::string name;
  Variable_Types type;
  Variable_Scope scope;
  int skip = 0;
  std::unique_ptr<Expression_Node> expression;
  llvm::Constant *code_gen();
};

struct Variable_Declaration_Node
{
  std::string name;
  Variable_Types type;
  Variable_Scope scope;
  int skip = 0;
  std::unique_ptr<Expression_Node> expression;
  llvm::Value *code_gen();
};

struct Then
{
  std::vector<std::shared_ptr<Token>> tokens;
  std::vector<Node *> nodes;
  llvm::Value *code_gen();
};

struct Function_Declaration_Node
{
  std::string name;
  std::map<std::string, Variable_Types> parameters;
  Variable_Types return_type;
  Then then;
  int skip = 0;
  std::map<std::string, llvm::Value *> loaded_variables;
  std::map<std::string, llvm::Value *> loaded_constants;
  llvm::Function *code_gen_prototype();
  llvm::Function *code_gen_function_body(llvm::Function *);
};

struct Return_Node
{
  std::unique_ptr<Expression_Node> return_expression;
  int skip = 0;
  llvm::Value *code_gen();
};

struct Node
{
  Node_Types type;
  Node *parent;
  int skip = 0;
  std::variant<Constant_Declaration_Node *, Variable_Declaration_Node *, Function_Declaration_Node *, Return_Node *> constant_declaration_node, variable_declaration_node, function_declaration_node, return_node;
};

inline std::map<std::string, Constant_Declaration_Node *> constants;
inline std::map<std::string, Variable_Declaration_Node *> variables;

Number evaluate_number_expression(Number_Expression_Node);

std::vector<Node *> parse_tokens(std::vector<std::shared_ptr<Token>>);
Node *parse_token(std::vector<std::shared_ptr<Token>>, int);

Return_Node *create_return_node(std::vector<std::shared_ptr<Token>>, int);

Function_Declaration_Node *create_function_declaration_node(std::vector<std::shared_ptr<Token>>, int);
std::map<std::string, Variable_Types> get_function_declaration_parameters(std::vector<std::shared_ptr<Token>>, int);
Then get_function_declaration_then(std::vector<std::shared_ptr<Token>>, int);

Constant_Declaration_Node *create_constant_declaration_node(std::vector<std::shared_ptr<Token>>, int);
std::unique_ptr<Expression_Node> create_expression_node(std::vector<std::shared_ptr<Token>>, int);

Variable_Declaration_Node *create_variable_declaration_node(std::vector<std::shared_ptr<Token>>, int);

Variable_Types get_variable_type_from_string(std::string);

void print_nodes(std::vector<Node *>);

#endif