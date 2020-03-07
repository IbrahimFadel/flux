#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>
#include "lexer.h"
#include "variables.h"

namespace Parser
{

typedef struct Node;
typedef struct Then;
typedef struct Function;

typedef enum Node_Types
{
  var,
  fn
} Node_Types;

struct Node
{
  int type;
  Variables::Variable variable;
  int skip;
  Parser::Function fn;
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

void print_nodes(std::vector<Parser::Node> nodes);
std::vector<Parser::Node> parse_tokens(std::vector<Lexer::Token> tokens);
Parser::Node parse_token(std::vector<Lexer::Token> tokens, int i);
Parser::Node create_int_node(std::vector<Lexer::Token> tokens, int i);
Parser::Node create_fn_node(std::vector<Lexer::Token> tokens, int i);
} // namespace Parser

std::ostream &operator<<(std::ostream &os, const Parser::Node &node);

#endif