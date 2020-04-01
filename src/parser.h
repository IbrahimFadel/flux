#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>

namespace Variables
{
struct Variable;
}

#include "lexer.h"
#include "variables.h"

namespace Parser
{

// struct Node;
// struct Then;
// struct Function;

enum Node_Types
{
  null,
  var,
  fn,
  fn_call
};

struct Then
{
  std::vector<Lexer::Token> tokens;
  std::vector<Parser::Node> nodes;
};

struct Function
{
  std::string name;
  std::vector<std::string> parameters;
  int return_type;
  Parser::Then then;
};

struct Function_call
{
  std::string name;
  std::vector<std::string> parameters;
};

struct Node
{
  int type;
  Variables::Variable *variable;
  int skip;
  Parser::Function *fn;
  Parser::Function_call *fn_call;
  Parser::Node *parent;
};

void print_nodes(std::vector<Parser::Node> nodes);
std::vector<Parser::Node> parse_tokens(std::vector<Lexer::Token> tokens);
Parser::Node parse_token(std::vector<Lexer::Token> tokens, int i, Parser::Node *parent);
Parser::Node create_int_node(std::vector<Lexer::Token> tokens, int i, Parser::Node *parent);
Parser::Node create_fn_node(std::vector<Lexer::Token> tokens, int i);
Parser::Node create_fn_call_node(std::vector<Lexer::Token> tokens, int i);
} // namespace Parser

std::ostream &operator<<(std::ostream &os, const Parser::Node &node);

#include "functions.h"

#endif