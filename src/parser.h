#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"

#include <iostream>
#include <vector>
#include <memory>
#include <variant>
#include <map>
#include <regex>

using std::unique_ptr;

struct Number_Expression_Node;
struct Expression_Node;
struct Variable_Declaration_Node;
struct Call_Node;
struct Binary_Expression_Node;
struct Prototype_Node;
struct Function_Node;
struct Node;
struct RegexMatch;

enum NodeTypes
{
  Error,
  VariableDeclarationNode
};

struct Expression_Node
{
};

struct Number_Expression_Node : public Expression_Node
{
  double value;
  Number_Expression_Node(double value) : value(value){};
};

struct Variable_Declaration_Node : public Expression_Node
{
  std::string name;
  Variable_Declaration_Node(std::string name) : name(name){};
};

struct Call_Node : public Expression_Node
{
  std::string name;
  std::vector<std::unique_ptr<Expression_Node>> args;
  Call_Node(std::string name, std::vector<std::unique_ptr<Expression_Node>> args) : name(name), args(args){};
};

struct Binary_Expression_Node : public Expression_Node
{
  std::string op;
  std::unique_ptr<Expression_Node> lhs;
  std::unique_ptr<Expression_Node> rhs;
  Binary_Expression_Node(std::string op, std::unique_ptr<Expression_Node> lhs, std::unique_ptr<Expression_Node> rhs) : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)){};
};

struct Prototype_Node : public Expression_Node
{
  std::string name;
  std::vector<std::string> arg_names;
  Prototype_Node(std::string name, std::vector<std::string> arg_names) : name(name), arg_names(arg_names){};
};

struct Function_Node : public Expression_Node
{
  std::unique_ptr<Prototype_Node> prototype;
  std::unique_ptr<Expression_Node> expression;
  Function_Node(std::unique_ptr<Prototype_Node> prototype, std::unique_ptr<Expression_Node> expression) : prototype(std::move(prototype)), expression(std::move(expression)){};
};

typedef struct Node
{
  std::variant<Variable_Declaration_Node *> variable_declaration_node;
  int type;
} Node;

struct RegexMatch
{
  bool error;
  std::string result;
};

std::vector<std::unique_ptr<Node>> parse_tokens(std::vector<std::shared_ptr<Token>>);
int match_string(std::string match);
RegexMatch *match_regex(std::regex match);
void consume_token();
int get_precedence();

// std::unique_ptr<Node> variable_declaration();
std::unique_ptr<Expression_Node> parse_expression();
std::unique_ptr<Expression_Node> parse_number_expression();
std::unique_ptr<Expression_Node> parse_paren_expression();
std::unique_ptr<Expression_Node> parse_identifier_expression();
std::unique_ptr<Expression_Node> parse_primary();
std::unique_ptr<Expression_Node> parse_bin_op_rhs(int prec, std::unique_ptr<Expression_Node> lhs);
std::unique_ptr<Expression_Node> parse_prototype();
std::unique_ptr<Expression_Node> parse_definition();

static std::vector<std::shared_ptr<Token>> tokens;
static int pos = 0;
static std::map<std::string, int> bin_op_precedence;

#endif